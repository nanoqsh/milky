use {
    crate::{
        Social,
        date::Date,
        icon::Icon,
        lang::{Lang, Localizer},
        rust,
    },
    pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd},
    std::{borrow::Cow, cmp, collections::HashSet, fmt::Write, iter},
};

pub struct Translation {
    pub lang: Lang,
    pub href: String,
}

pub struct Make<'art> {
    pub l: Localizer<'art>,
    pub blog: &'art str,
    pub title: &'art str,
    pub translations: &'art mut dyn Iterator<Item = Translation>,
    pub social: &'art [Social],
    pub target: Target<'art>,
}

pub enum Target<'art> {
    List(&'art [Post<'art>]),
    Article {
        md: &'art str,
        date: Date,
        index_href: String,
        deps: &'art mut HashSet<Box<str>>,
    },
}

pub fn make(make: Make<'_>) -> maud::Markup {
    let Make {
        l,
        blog,
        title,
        translations,
        social,
        target,
    } = make;

    let translations_into_buttons = translations.map(Button::from_translation);

    match target {
        Target::List(posts) => {
            let placeholder = maud::html! { div {} };
            let subtitle = subtitle(placeholder, translations_into_buttons, 0);
            let header = header(blog, title, subtitle);
            page(blog, header, list(posts, l), social, 0)
        }
        Target::Article {
            md,
            date,
            index_href,
            deps,
        } => {
            let buttons =
                iter::once(Button::articles(index_href, l)).chain(translations_into_buttons);

            let html = md_to_html(md, deps);
            let date = date_block(date, l);
            let subtitle = subtitle(date, buttons, 1);
            let header = header(blog, title, subtitle);
            page(title, header, article(&html), social, 1)
        }
    }
}

#[derive(Clone, Copy)]
pub struct Post<'art> {
    pub name: &'art str,
    pub title: &'art str,
    pub date: Date,
}

impl Post<'_> {
    pub fn by_date(&self) -> impl cmp::Ord + use<> {
        let &Self {
            date: Date { day, month, year },
            ..
        } = self;

        (year, month as u8, day)
    }
}

fn list(posts: &[Post<'_>], l: Localizer<'_>) -> maud::Markup {
    let href = |name| format!("{}/{name}.html", l.lang());

    maud::html! {
        ul .content.deferred.show {
            @for Post { name, title, date } in posts {
                li .list-item {
                    a href=(href(name)) { (title) }
                    (date_block(*date, l))
                }
            }
        }
    }
}

fn article(article: &str) -> maud::Markup {
    maud::html! {
        article .content.deferred.show { (maud::PreEscaped(article)) }
    }
}

fn date_block(date: Date, l: Localizer<'_>) -> maud::Markup {
    maud::html! {
        .date { (date.render(l)) (Icon::Date) }
    }
}

fn header<S>(blog: &str, title: &str, subtitle: S) -> maud::Markup
where
    S: maud::Render,
{
    maud::html! {
        header .content.deferred.show {
            .blog-title { (blog) }
            (subtitle)
            @if !title.is_empty() {
                h1 .title { (title) }
            }
        }
    }
}

struct Button<'art> {
    icon: Icon,
    label: Cow<'art, str>,
    href: String,
}

impl<'art> Button<'art> {
    fn from_translation(Translation { lang, href }: Translation) -> Self {
        Self {
            icon: Icon::Earth,
            label: Cow::Owned(lang.to_string()),
            href,
        }
    }

    fn articles(href: String, l: Localizer<'art>) -> Self {
        Self {
            icon: Icon::Bookshelf,
            label: Cow::Borrowed(l.articles()),
            href,
        }
    }
}

fn subtitle<'art, D, B>(date: D, buttons: B, level: u8) -> maud::Markup
where
    D: maud::Render,
    B: IntoIterator<Item = Button<'art>>,
{
    maud::html! {
        .hor {
            (date)
            .hor {
                @for Button { icon, label, href } in buttons {
                    a .hor.button href=(relative_path(&href, level)) { (icon) (label) }
                }
            }
        }
    }
}

fn page<H, C>(title: &str, header: H, content: C, social: &[Social], level: u8) -> maud::Markup
where
    H: maud::Render,
    C: maud::Render,
{
    maud::html! {
        (maud::DOCTYPE)
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            link rel="icon" href=(relative_path("favicon.svg", level));
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=Alfa+Slab+One&family=Carlito:ital,wght@0,400;0,700;1,400;1,700&family=JetBrains+Mono:wght@100..800&display=swap" rel="stylesheet";
            link rel="stylesheet" href=(relative_path("style.css", level));
            title { (title) }
        }
        body {
            style { (maud::PreEscaped(include_str!("../assets/inline.css"))) }
            script { (maud::PreEscaped(include_str!("../assets/show.js"))) }
            (header)
            (content)
            footer .deferred.show {
                .socials {
                    @for s in social {
                        a .icon href=(s.href) aria-label=(s.icon.label()) target="_blank" {
                            (s.icon)
                        }
                    }
                }
            }
        }
    }
}

fn relative_path(base: &str, level: u8) -> impl maud::Render {
    struct Rel<'base>(&'base str, u8);

    impl maud::Render for Rel<'_> {
        fn render_to(&self, buffer: &mut String) {
            let &Self(base, level) = self;
            for _ in 0..level {
                buffer.push_str("../");
            }

            buffer.push_str(base);
        }
    }

    Rel(base, level)
}

pub fn escape(s: &str, output: &mut String) {
    // don't reinvent the wheel,
    // the `maud` already has an implementation of escaping
    _ = maud::Escaper::new(output).write_str(s);
}

fn md_to_html(md: &str, deps: &mut HashSet<Box<str>>) -> String {
    let mut html = String::new();
    let mut code = None;

    for event in Parser::new(md) {
        match event {
            Event::Start(Tag::Paragraph) => html.push_str("<p>"),
            Event::Start(Tag::Heading { level, .. }) => _ = write!(&mut html, "<{level}>"),
            Event::Start(Tag::BlockQuote(_)) => todo!(),
            Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => html.push_str("<pre><code>"),
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(s))) => {
                if &*s == "rust" {
                    html.push_str("<pre><code>");
                    let len = html.len();
                    code = Some((len, len));
                } else {
                    html.push_str("<pre><code>");
                }
            }
            Event::Start(Tag::HtmlBlock) => todo!(),
            Event::Start(Tag::List(_)) => html.push_str("<ul>"),
            Event::Start(Tag::Item) => html.push_str("<li>"),
            Event::Start(Tag::FootnoteDefinition(_)) => todo!(),
            Event::Start(Tag::DefinitionList) => todo!(),
            Event::Start(Tag::DefinitionListTitle) => todo!(),
            Event::Start(Tag::DefinitionListDefinition) => todo!(),
            Event::Start(Tag::Table(_)) => todo!(),
            Event::Start(Tag::TableHead) => todo!(),
            Event::Start(Tag::TableRow) => todo!(),
            Event::Start(Tag::TableCell) => todo!(),
            Event::Start(Tag::Emphasis) => html.push_str("<em>"),
            Event::Start(Tag::Strong) => html.push_str("<strong>"),
            Event::Start(Tag::Strikethrough) => todo!(),
            Event::Start(Tag::Superscript) => todo!(),
            Event::Start(Tag::Subscript) => todo!(),
            Event::Start(Tag::Link { dest_url, .. }) => {
                _ = write!(&mut html, "<a href=\"{dest_url}\" target=\"_blank\">");
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                _ = write!(&mut html, "<img src=\"../{dest_url}\">");
                deps.insert(Box::from(&*dest_url));
            }
            Event::Start(Tag::MetadataBlock(_)) => todo!(),
            Event::End(TagEnd::Paragraph) => html.push_str("</p>"),
            Event::End(TagEnd::Heading(level)) => _ = write!(&mut html, "</{level}>"),
            Event::End(TagEnd::BlockQuote(_)) => todo!(),
            Event::End(TagEnd::CodeBlock) => {
                if let Some((start, end)) = code.take() {
                    let src = &html[start..end];
                    let src = match rust::highlight(src) {
                        Ok(src) => src,
                        Err(e) => {
                            eprintln!("highlight rust error: {e}");
                            src.to_owned()
                        }
                    };

                    html.truncate(start);
                    html.push_str(&src);
                }

                html.push_str("</code></pre>");
            }
            Event::End(TagEnd::HtmlBlock) => todo!(),
            Event::End(TagEnd::List(_)) => html.push_str("</ul>"),
            Event::End(TagEnd::Item) => html.push_str("</li>"),
            Event::End(TagEnd::FootnoteDefinition) => todo!(),
            Event::End(TagEnd::DefinitionList) => todo!(),
            Event::End(TagEnd::DefinitionListTitle) => todo!(),
            Event::End(TagEnd::DefinitionListDefinition) => todo!(),
            Event::End(TagEnd::Table) => todo!(),
            Event::End(TagEnd::TableHead) => todo!(),
            Event::End(TagEnd::TableRow) => todo!(),
            Event::End(TagEnd::TableCell) => todo!(),
            Event::End(TagEnd::Emphasis) => html.push_str("</em>"),
            Event::End(TagEnd::Strong) => html.push_str("</strong>"),
            Event::End(TagEnd::Strikethrough) => todo!(),
            Event::End(TagEnd::Superscript) => todo!(),
            Event::End(TagEnd::Subscript) => todo!(),
            Event::End(TagEnd::Link) => html.push_str("</a>"),
            Event::End(TagEnd::Image) => html.push_str("</img>"),
            Event::End(TagEnd::MetadataBlock(_)) => todo!(),
            Event::Text(s) => {
                if let Some((start, _)) = code {
                    html.push_str(&s);
                    code = Some((start, html.len()));
                } else {
                    escape(&s, &mut html);
                }
            }
            Event::Code(s) => {
                html.push_str("<code class=\"inline\">");
                escape(&s, &mut html);
                html.push_str("</code>");
            }
            Event::InlineMath(_) => todo!(),
            Event::DisplayMath(_) => todo!(),
            Event::Html(s) => html.push_str(&s),
            Event::InlineHtml(s) => html.push_str(&s),
            Event::FootnoteReference(_) => todo!(),
            Event::SoftBreak => html.push_str("<br>"),
            Event::HardBreak => todo!(),
            Event::Rule => todo!(),
            Event::TaskListMarker(_) => todo!(),
        }
    }

    html
}
