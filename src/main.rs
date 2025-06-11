mod html;

use {
    serde::Deserialize,
    std::{collections::HashMap, fs, io::Error, process::ExitCode},
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

    for (name, _) in conf.articles {
        println!("generate {name}.html");

        let article_path = format!("{name}.md");
        let md = read(&article_path)?;

        let page = html::make(&md);

        let page_path = format!("{dist_path}/{name}.html");
        write(&page_path, &page)?;
    }

    let style_path = "dist/style.css";
    if !exists(style_path)? {
        println!("save style.css");
        write(style_path, include_str!("../assets/style.css"))?;
    }

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
