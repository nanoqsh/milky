mod date;
mod html;
mod icon;
mod lang;

use {
    crate::{date::Date, html::Html, icon::Icon, lang::Lang},
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, hash_map::Entry},
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

    let dist_path = "dist";
    create_dir_all(dist_path)?;

    let mut gener = Generator::new(dist_path, &conf.social)?;
    for (name, article) in conf.articles {
        let generate = gener.generate(&name, &article);
        for lang in Lang::ENUM {
            generate(lang)?;
        }
    }

    gener.save_style()?;
    gener.save_meta()?;
    Ok(())
}

struct Generator<'soc> {
    meta: Meta,
    rerender: bool,
    dist_path: &'static str,
    social: &'soc [Social],
}

impl<'soc> Generator<'soc> {
    fn new(dist_path: &'static str, social: &'soc [Social]) -> Result<Self, Error> {
        let mut meta = Meta::read()?;
        let rerender = meta.version != Meta::VERSION;
        if rerender {
            meta.version = Meta::VERSION;
        }

        Ok(Self {
            meta,
            rerender,
            dist_path,
            social,
        })
    }

    fn generate(&mut self, name: &str, article: &Article) -> impl Fn(Lang) -> Result<(), Error> {
        let article_meta = self.meta.articles.entry(name.to_owned());
        let skip = matches!(article_meta, Entry::Occupied(_)) && !self.rerender;

        let article_meta = article_meta.or_insert_with(|| ArticleMeta { date: date::now() });
        let dist_path = self.dist_path;
        let social = self.social;

        move |_lang| {
            if skip {
                return Ok(());
            }

            println!("generate {name}.html");

            let article_path = format!("{name}.md");
            let md = read(&article_path)?;

            let Html { page, deps } = html::make(&md, &article.title, article_meta.date, social);

            let page_path = format!("{dist_path}/{name}.html",);
            write(&page_path, &page)?;

            for dep in deps {
                let to = format!("{dist_path}/{dep}");
                println!("save {to}");
                copy(&dep, &to)?;
            }

            Ok(())
        }
    }

    fn save_style(&self) -> Result<(), Error> {
        let style_path = "dist/style.css";
        if self.rerender || !exists(style_path)? {
            println!("save style.css");
            write(style_path, include_str!("../assets/style.css"))?;
        }

        Ok(())
    }

    fn save_meta(self) -> Result<(), Error> {
        self.meta.write()
    }
}

#[derive(Deserialize)]
struct Article {
    title: String,
}

#[derive(Deserialize)]
struct Social {
    href: String,
    icon: Icon,
}

struct Conf {
    articles: Vec<(String, Article)>,
    social: Vec<Social>,
}

fn read_conf() -> Result<Conf, Error> {
    #[derive(Deserialize)]
    struct Scheme {
        article: HashMap<String, Article>,
        social: Vec<Social>,
    }

    let conf_path = "Milky.toml";
    let conf = read(conf_path)?;
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
    const VERSION: u32 = 1;

    fn new() -> Self {
        Self {
            version: Self::VERSION,
            articles: HashMap::new(),
        }
    }

    fn read() -> Result<Self, Error> {
        let meta_path = "Meta.toml";
        let meta = match read(meta_path) {
            Ok(meta) => meta,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                eprintln!("create the Meta.toml");
                return Ok(Self::new());
            }
            Err(e) => return Err(e),
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

fn read(path: &str) -> Result<String, Error> {
    fs::read_to_string(path).inspect_err(|_| eprintln!("failed to read file {path}"))
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
