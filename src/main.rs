mod date;
mod html;
mod icon;
mod lang;

use {
    crate::{date::Date, html::Make, icon::Icon, lang::Lang},
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, HashSet, hash_map::Entry},
        fs,
        io::{Error, ErrorKind},
        process::ExitCode,
    },
};

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run() -> Result<(), Error> {
    let conf = read_conf()?;
    let mut gener = Generator::new(&conf.social)?;
    for (name, article) in conf.articles {
        let mut generate = gener.generate(&name, &article);
        for lang in Lang::ENUM {
            generate(lang)?;
        }
    }

    gener.save()?;
    Ok(())
}

struct Generator<'soc> {
    meta: Meta,
    rerender: bool,
    social: &'soc [Social],
    deps: HashSet<Box<str>>,
}

impl<'soc> Generator<'soc> {
    const DIST_PATH: &'static str = "dist";

    fn new(social: &'soc [Social]) -> Result<Self, Error> {
        create_dir_all(Self::DIST_PATH)?;

        for lang in Lang::ENUM {
            create_dir_all(&format!("{}/{lang}", Self::DIST_PATH))?;
        }

        let mut meta = Meta::read()?;
        let rerender = meta.version != Meta::VERSION;
        if rerender {
            meta.version = Meta::VERSION;
        }

        Ok(Self {
            meta,
            rerender,
            social,
            deps: HashSet::new(),
        })
    }

    fn generate(&mut self, name: &str, article: &Article) -> impl FnMut(Lang) -> Result<(), Error> {
        let article_meta = self.meta.articles.entry(name.to_owned());
        let skip = matches!(article_meta, Entry::Occupied(_)) && !self.rerender;

        let article_meta = article_meta.or_insert_with(|| ArticleMeta { date: date::now() });
        let social = self.social;
        let deps = &mut self.deps;

        move |lang| {
            if skip {
                return Ok(());
            }

            let article_path = format!("{lang}/{name}.md");
            let page_path = format!("{}/{lang}/{name}.html", Self::DIST_PATH);
            println!("generate {page_path}");

            let md = match read(&article_path) {
                Read::Content(s) => s,
                Read::NotFound => {
                    eprintln!("{article_path} not found!");
                    return Ok(());
                }
                Read::Failed(e) => return Err(e),
            };

            let page = html::make(Make {
                md: &md,
                title: &article.title,
                date: article_meta.date,
                social,
                deps,
            });

            write(&page_path, &page)?;

            Ok(())
        }
    }

    fn save(self) -> Result<(), Error> {
        for dep in self.deps {
            let to = format!("{}/{dep}", Self::DIST_PATH);
            println!("save {to}");
            copy(&dep, &to)?;
        }

        let style_path = "dist/style.css";
        if self.rerender || !exists(style_path)? {
            println!("save style.css");
            write(style_path, include_str!("../assets/style.css"))?;
        }

        self.meta.write()?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct Article {
    title: String,
}

#[derive(Deserialize)]
struct Social {
    href: Box<str>,
    icon: Icon,
}

struct Conf {
    articles: Vec<(Box<str>, Article)>,
    social: Vec<Social>,
}

fn read_conf() -> Result<Conf, Error> {
    #[derive(Deserialize)]
    struct Scheme {
        article: HashMap<Box<str>, Article>,
        social: Vec<Social>,
    }

    let conf_path = "Milky.toml";
    let conf = read(conf_path).into_result()?;
    let scheme: Scheme = toml::from_str(&conf)
        .inspect_err(|_| eprintln!("failed to deserialize file {conf_path}"))
        .map_err(Error::other)?;

    let mut articles: Vec<_> = scheme.article.into_iter().collect();
    articles.sort_by(|(a, _), (b, _)| a.cmp(b));

    Ok(Conf {
        articles,
        social: scheme.social,
    })
}

#[derive(Serialize, Deserialize)]
struct ArticleMeta {
    date: Date,
}

#[derive(Serialize, Deserialize)]
struct Meta {
    version: u32,
    articles: HashMap<String, ArticleMeta>,
}

impl Meta {
    const VERSION: u32 = 0;

    fn new() -> Self {
        Self {
            version: Self::VERSION,
            articles: HashMap::new(),
        }
    }

    fn read() -> Result<Self, Error> {
        let meta_path = "Meta.toml";
        let meta = match read(meta_path) {
            Read::Content(s) => s,
            Read::NotFound => {
                eprintln!("create the Meta.toml");
                return Ok(Self::new());
            }
            Read::Failed(e) => return Err(e),
        };

        let meta = toml::from_str(&meta)
            .inspect_err(|_| eprintln!("failed to deserialize file {meta_path}"))
            .map_err(Error::other)?;

        Ok(meta)
    }

    fn write(self) -> Result<(), Error> {
        let meta = toml::to_string(&self)
            .inspect_err(|_| eprintln!("failed to serialize meta info"))
            .map_err(Error::other)?;

        write("Meta.toml", &meta)?;
        Ok(())
    }
}

enum Read {
    Content(String),
    NotFound,
    Failed(Error),
}

impl Read {
    fn into_result(self) -> Result<String, Error> {
        match self {
            Self::Content(s) => Ok(s),
            Self::NotFound => Err(ErrorKind::NotFound.into()),
            Self::Failed(e) => Err(e),
        }
    }
}

fn read(path: &str) -> Read {
    match fs::read_to_string(path) {
        Ok(s) => Read::Content(s),
        Err(e) if e.kind() == ErrorKind::NotFound => Read::NotFound,
        Err(e) => {
            eprintln!("failed to read file {path}");
            Read::Failed(e)
        }
    }
}

fn write(path: &str, contents: &str) -> Result<(), Error> {
    fs::write(path, contents).inspect_err(|_| eprintln!("failed to write file {path}"))
}

fn create_dir_all(path: &str) -> Result<(), Error> {
    fs::create_dir_all(path).inspect_err(|_| eprintln!("failed to create {path} directory"))
}

fn exists(path: &str) -> Result<bool, Error> {
    fs::exists(path).inspect_err(|_| eprintln!("failed to check path {path}"))
}

fn copy(from: &str, to: &str) -> Result<(), Error> {
    fs::copy(from, to).inspect_err(|_| eprintln!("failed to copy from {from} to {to}"))?;
    Ok(())
}
