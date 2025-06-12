mod date;
mod html;
mod lang;

use {
    crate::date::Date,
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
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

    let mut meta = read_meta()?;

    for (name, _) in conf.articles {
        if meta.articles.contains_key(&name) && meta.version == Meta::VERSION {
            println!("skip {name}.html");
            continue;
        }

        println!("generate {name}.html");

        let article_path = format!("{name}.md");
        let md = read(&article_path)?;

        let date = date::now();
        let page = html::make(&md, date);

        let page_path = format!("{dist_path}/{name}.html");
        write(&page_path, &page)?;

        meta.articles.insert(name, ArticleMeta { date });
    }

    let style_path = "dist/style.css";
    if !exists(style_path)? {
        println!("save style.css");
        write(style_path, include_str!("../assets/style.css"))?;
    }

    write_meta(&meta)?;
    Ok(())
}

#[derive(Deserialize)]
struct Article {}

struct Conf {
    articles: Vec<(String, Article)>,
}

fn read_conf() -> Result<Conf, Error> {
    #[derive(Deserialize)]
    struct Scheme {
        article: HashMap<String, Article>,
    }

    let conf_path = "Milky.toml";
    let conf = read(conf_path)?;
    let scheme: Scheme = toml::from_str(&conf)
        .inspect_err(|_| eprintln!("failed to deserialize file {conf_path}"))
        .map_err(Error::other)?;

    let mut articles: Vec<_> = scheme.article.into_iter().collect();
    articles.sort_by(|(a, _), (b, _)| a.cmp(b));

    Ok(Conf { articles })
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
}

fn read_meta() -> Result<Meta, Error> {
    let meta_path = "Meta.toml";
    let meta = match read(meta_path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            eprintln!("create the Meta.toml");
            return Ok(Meta::new());
        }
        Err(e) => return Err(e),
    };

    let meta = toml::from_str(&meta)
        .inspect_err(|_| eprintln!("failed to deserialize file {meta_path}"))
        .map_err(Error::other)?;

    Ok(meta)
}

fn write_meta(meta: &Meta) -> Result<(), Error> {
    let meta = toml::to_string(meta)
        .inspect_err(|_| eprintln!("failed to serialize meta info"))
        .map_err(Error::other)?;

    write("Meta.toml", &meta)?;
    Ok(())
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
