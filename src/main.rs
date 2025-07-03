mod date;
mod html;
mod icon;
mod lang;

use {
    crate::{
        date::Date,
        html::{Make, Post, Target, Translation},
        icon::Icon,
        lang::{Lang, Local},
    },
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, HashSet},
        env, fs,
        io::{Error, ErrorKind},
        process::ExitCode,
    },
};

fn main() -> ExitCode {
    let force = env::args()
        .skip(1)
        .any(|arg| arg == "-f" || arg == "--force");

    if let Err(e) = run(force) {
        eprintln!("error: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(force: bool) -> Result<(), Error> {
    let conf = read_conf()?;
    let mut gener = Generator::new(force, &conf)?;
    for (name, info) in &conf.articles {
        let mut generate = gener.generate(name);
        for (&lang, article) in info {
            let mut more = info.keys().filter(|l| **l != lang).copied();
            let langs = Langs {
                lang,
                more: &mut more,
            };

            generate(langs, article)?;
        }
    }

    gener.generate_list()?;
    gener.save()?;
    Ok(())
}

struct Langs<'it> {
    lang: Lang,
    more: &'it mut dyn Iterator<Item = Lang>,
}

struct Generator<'conf> {
    conf: &'conf Conf,
    meta: Meta,
    rerender: bool,
    deps: HashSet<Box<str>>,
    langs: HashSet<Lang>,
    posts: HashMap<Lang, Vec<Post<'conf>>>,
}

impl<'conf> Generator<'conf> {
    const DIST_PATH: &'static str = "dist";

    fn new(force: bool, conf: &'conf Conf) -> Result<Self, Error> {
        create_dir_all(Self::DIST_PATH)?;

        let mut meta = Meta::read()?;
        let rerender = meta.version != Meta::VERSION || force;
        if rerender {
            meta.version = Meta::VERSION;
        }

        Ok(Self {
            conf,
            meta,
            rerender,
            deps: HashSet::new(),
            langs: HashSet::new(),
            posts: HashMap::new(),
        })
    }

    fn generate(
        &mut self,
        name: &'conf str,
    ) -> impl FnMut(Langs<'_>, &'conf Article) -> Result<(), Error> {
        let skip = !self.rerender;
        let meta = self
            .meta
            .articles
            .entry(name.to_owned())
            .or_insert_with(|| ArticleMeta {
                date: date::now(),
                langs: HashSet::new(),
            });

        let conf = self.conf;
        let deps = &mut self.deps;
        let langs = &mut self.langs;
        let posts = &mut self.posts;

        move |Langs { lang, more }, Article { title }| {
            posts.entry(lang).or_default().push(Post {
                name,
                title,
                date: meta.date,
            });

            if skip && meta.langs.contains(&lang) {
                return Ok(());
            }

            if langs.insert(lang) {
                create_dir_all(&format!("{}/{lang}", Self::DIST_PATH))?;
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

            let mut translations = more.map(|lang| Translation {
                lang,
                href: format!("{lang}/{name}.html"),
            });

            let page = html::make(Make {
                l: conf.local.bind(lang),
                title,
                translations: &mut translations,
                social: &conf.social,
                target: Target::Article {
                    md: &md,
                    date: meta.date,
                    deps,
                },
            });

            write(&page_path, &page.into_string())?;
            meta.langs.insert(lang);

            Ok(())
        }
    }

    fn generate_list(&mut self) -> Result<(), Error> {
        for (&lang, posts) in &mut self.posts {
            let page_path = format!("{}/{lang}.html", Self::DIST_PATH);
            println!("generate {page_path}");

            posts.sort_by_key(Post::by_date);

            let mut translations = self
                .langs
                .iter()
                .copied()
                .filter(|l| *l != lang)
                .map(|lang| Translation {
                    lang,
                    href: format!("{lang}.html"),
                });

            let page = html::make(Make {
                l: self.conf.local.bind(lang),
                title: &self.conf.blog.title,
                translations: &mut translations,
                social: &self.conf.social,
                target: Target::List(posts),
            });

            write(&page_path, &page.into_string())?;
        }

        Ok(())
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
struct Blog {
    title: Box<str>,
}

impl Default for Blog {
    fn default() -> Self {
        Self {
            title: Box::from("Blog title"),
        }
    }
}

#[derive(Deserialize)]
struct Article {
    title: Box<str>,
}

#[derive(Deserialize)]
struct Social {
    href: Box<str>,
    icon: Icon,
}

type ArticleInfo = HashMap<Lang, Article>;

struct Conf {
    blog: Blog,
    articles: Vec<(Box<str>, ArticleInfo)>,
    social: Vec<Social>,
    local: Local,
}

fn read_conf() -> Result<Conf, Error> {
    #[derive(Deserialize)]
    struct Scheme {
        #[serde(default)]
        blog: Blog,
        #[serde(default)]
        article: HashMap<Box<str>, ArticleInfo>,
        #[serde(default)]
        social: Vec<Social>,
    }

    let conf_path = "Milky.toml";
    let conf = read(conf_path).into_result()?;
    let scheme: Scheme = toml::from_str(&conf)
        .inspect_err(|_| eprintln!("failed to deserialize file {conf_path}"))
        .map_err(Error::other)?;

    let mut articles: Vec<_> = scheme.article.into_iter().collect();
    articles.sort_by(|(a, _), (b, _)| a.cmp(b));

    let local_path = "Local.toml";
    let local = match read(local_path) {
        Read::Content(s) => toml::from_str(&s)
            .inspect_err(|_| eprintln!("failed to deserialize file {local_path}"))
            .map_err(Error::other)?,
        Read::NotFound => {
            eprintln!("file Local.toml not found");
            Local::new()
        }
        Read::Failed(e) => return Err(e),
    };

    Ok(Conf {
        blog: scheme.blog,
        articles,
        social: scheme.social,
        local,
    })
}

#[derive(Serialize, Deserialize)]
struct ArticleMeta {
    date: Date,
    #[serde(default)]
    langs: HashSet<Lang>,
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
