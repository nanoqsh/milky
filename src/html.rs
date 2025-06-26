use {
    crate::{
        Social,
        date::Date,
        lang::{Lang, Local},
    },
    proc_macro2::{Span, TokenStream, TokenTree},
    pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd},
    std::{collections::HashSet, fmt::Write},
};

pub struct Make<'art> {
    pub lang: Lang,
    pub local: &'art Local,
    pub title: &'art str,
    pub social: &'art [Social],
    pub target: Target<'art>,
}

pub enum Target<'art> {
    #[expect(dead_code)]
    Main,
    Article {
        md: &'art str,
        date: Date,
        deps: &'art mut HashSet<Box<str>>,
    },
}

pub fn make(make: Make<'_>) -> maud::Markup {
    let Make {
        lang,
        local,
        title,
        social,
        target,
    } = make;

    match target {
        Target::Main => todo!(),
        Target::Article { md, date, deps } => {
            let date = date.render(local, lang);
            let html = md_to_html(md, deps);
            page(title, article(&html), date, social, 1)
        }
    }
}

fn article(article: &str) -> maud::Markup {
    maud::html! {
        article .deferred.show { (maud::PreEscaped(article)) }
    }
}

fn page<C, D>(title: &str, content: C, date: D, social: &[Social], level: u8) -> maud::Markup
where
    C: maud::Render,
    D: maud::Render,
{
    maud::html! {
        (maud::DOCTYPE)
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            // link rel="preconnect" href="https://fonts.googleapis.com";
            // link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            // link href="https://fonts.googleapis.com/css2?family=Carlito:ital,wght@0,400;0,700;1,400;1,700&family=JetBrains+Mono:wght@100..800&display=swap" rel="stylesheet";
            link rel="stylesheet" href=(relative_path("style.css", level));
            title { (title) }
        }
        body {
            style { (maud::PreEscaped(include_str!("../assets/inline.css"))) }
            script { (maud::PreEscaped(include_str!("../assets/show.js"))) }
            header .deferred.show {
                h1 { (title) }
                .date { (date) }
            }
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

fn escape(s: &str, output: &mut String) {
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
            Event::Start(Tag::List(_)) => todo!(),
            Event::Start(Tag::Item) => todo!(),
            Event::Start(Tag::FootnoteDefinition(_)) => todo!(),
            Event::Start(Tag::DefinitionList) => todo!(),
            Event::Start(Tag::DefinitionListTitle) => todo!(),
            Event::Start(Tag::DefinitionListDefinition) => todo!(),
            Event::Start(Tag::Table(_)) => todo!(),
            Event::Start(Tag::TableHead) => todo!(),
            Event::Start(Tag::TableRow) => todo!(),
            Event::Start(Tag::TableCell) => todo!(),
            Event::Start(Tag::Emphasis) => todo!(),
            Event::Start(Tag::Strong) => todo!(),
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
                    let src = match highlight_rust(src) {
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
            Event::End(TagEnd::List(_)) => todo!(),
            Event::End(TagEnd::Item) => todo!(),
            Event::End(TagEnd::FootnoteDefinition) => todo!(),
            Event::End(TagEnd::DefinitionList) => todo!(),
            Event::End(TagEnd::DefinitionListTitle) => todo!(),
            Event::End(TagEnd::DefinitionListDefinition) => todo!(),
            Event::End(TagEnd::Table) => todo!(),
            Event::End(TagEnd::TableHead) => todo!(),
            Event::End(TagEnd::TableRow) => todo!(),
            Event::End(TagEnd::TableCell) => todo!(),
            Event::End(TagEnd::Emphasis) => todo!(),
            Event::End(TagEnd::Strong) => todo!(),
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

fn highlight_rust(code: &str) -> Result<String, syn::Error> {
    let stream = syn::parse_str(code)?;
    let mut tokens = vec![];
    parse(code, stream, &mut tokens);

    let mut output = String::new();
    let mut last = 0;
    for token in tokens {
        let range = token.span.byte_range();
        escape_with_comments(&code[last..range.start], &mut output);
        _ = write!(&mut output, "<span class=\"{}\">", token.kind.class());
        escape(&code[range.start..range.end], &mut output);
        output.push_str("</span>");
        last = range.end;
    }

    if let Some(s) = code.get(last..) {
        escape_with_comments(s, &mut output);
    }

    Ok(output)
}

fn escape_with_comments(s: &str, output: &mut String) {
    for comm in find_comments(s) {
        match comm {
            Ok(s) => {
                output.push_str("<span class=\"cm\">");
                escape(s, output);
                output.push_str("</span>");
            }
            Err(s) => escape(s, output),
        }
    }
}

fn find_comments(mut code: &str) -> impl Iterator<Item = Result<&str, &str>> {
    // todo: state machine to fairly return an iterator
    let mut out = vec![];

    #[derive(Clone, Copy)]
    enum Comment {
        Slash,
        Star,
    }

    let pats = [(*b"//", Comment::Slash), (*b"/*", Comment::Star)]
        .map(|(p, var)| move |w| (w == p).then_some(var));

    loop {
        let Some((pos, pat)) = find(code, pats) else {
            out.push(Err(code));
            break;
        };

        let (left, right) = code.split_at(pos);
        out.push(Err(left));
        code = right;

        let pos = match pat {
            Comment::Slash => {
                let f = |w| (w == *b"\n").then_some(());
                let Some((pos, ())) = find(code, [f]) else {
                    out.push(Ok(code));
                    break;
                };

                pos + 1
            }
            Comment::Star => {
                let f = |w| (w == *b"*/").then_some(());
                let Some((pos, ())) = find(code, [f]) else {
                    out.push(Ok(code));
                    break;
                };

                pos + 2
            }
        };

        let (left, right) = code.split_at(pos);
        out.push(Ok(left));
        code = right;
    }

    out.into_iter()
}

fn find<F, P, const N: usize, const M: usize>(code: &str, pats: [F; M]) -> Option<(usize, P)>
where
    F: Fn([u8; N]) -> Option<P>,
{
    let mut pat = None;
    let pos = code.as_bytes().windows(N).position(|w| {
        let w: [u8; N] = w.try_into().expect("n bytes window");
        pat = pats.iter().find_map(|p| p(w));
        pat.is_some()
    })?;

    Some((pos, pat?))
}

enum Kind {
    Keyword,
    Literal,
    Typing,
    Generic,
    Ident,
}

impl Kind {
    fn class(self) -> &'static str {
        match self {
            Self::Keyword => "kw",
            Self::Literal => "li",
            Self::Typing => "ty",
            Self::Generic => "ge",
            Self::Ident => "id",
        }
    }
}

struct Token {
    kind: Kind,
    span: Span,
}

fn parse(code: &str, stream: TokenStream, tokens: &mut Vec<Token>) {
    for tree in stream {
        match tree {
            TokenTree::Group(group) => parse(code, group.stream(), tokens),
            TokenTree::Ident(ident) => {
                let span = ident.span();

                // skip docs
                if code[span.byte_range()].starts_with("///") {
                    continue;
                }

                let kind = match &code[span.byte_range()] {
                    s if is_keyword(s) => Kind::Keyword,
                    s if is_generic(s) => Kind::Generic,
                    s if is_typing(s) => Kind::Typing,
                    _ => Kind::Ident,
                };

                tokens.push(Token { kind, span });
            }
            TokenTree::Punct(_) => {}
            TokenTree::Literal(literal) => {
                let span = literal.span();

                // skip docs
                if code[span.byte_range()].starts_with("///") {
                    continue;
                }

                tokens.push(Token {
                    kind: Kind::Literal,
                    span,
                });
            }
        }
    }
}

fn is_keyword(s: &str) -> bool {
    thread_local! {
        static KEYWORDS: HashSet<&'static str> = HashSet::from([
            "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
            "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
            "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
            "use", "where", "while", "async", "await", "dyn",
        ]);
    }

    KEYWORDS.with(|set| set.contains(s))
}

fn is_generic(s: &str) -> bool {
    s.len() == 1 && s.starts_with(|c: char| c.is_ascii_uppercase())
}

fn is_typing(s: &str) -> bool {
    if s.starts_with(|c: char| c.is_ascii_uppercase()) {
        return true;
    }

    thread_local! {
        static STDTYPES: HashSet<&'static str> = HashSet::from([
            "str", "char", "bool",
            "u8", "u16", "u32", "u64", "u128", "usize",
            "i8", "i16", "i32", "i64", "i128", "isize",
        ]);
    }

    STDTYPES.with(|set| set.contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_comment_slash() {
        let actual: Vec<_> = find_comments("//aaa")
            .filter(|(Ok(s) | Err(s))| !s.is_empty())
            .collect();

        assert_eq!(actual, [Ok("//aaa")]);

        let actual: Vec<_> = find_comments("aaa").collect();
        assert_eq!(actual, [Err("aaa")]);

        let actual: Vec<_> = find_comments("aa//b\nc//dd").collect();
        assert_eq!(
            actual,
            [
                Err("aa"),   //
                Ok("//b\n"), //
                Err("c"),    //
                Ok("//dd"),  //
            ]
        );
    }

    #[test]
    fn find_comment_star() {
        let actual: Vec<_> = find_comments("/*aaa")
            .filter(|(Ok(s) | Err(s))| !s.is_empty())
            .collect();

        assert_eq!(actual, [Ok("/*aaa")]);

        let actual: Vec<_> = find_comments("aaa").collect();
        assert_eq!(actual, [Err("aaa")]);

        let actual: Vec<_> = find_comments("aa/*b*/c/*dd*/eee").collect();
        assert_eq!(
            actual,
            [
                Err("aa"),    //
                Ok("/*b*/"),  //
                Err("c"),     //
                Ok("/*dd*/"), //
                Err("eee"),   //
            ]
        );
    }

    #[test]
    fn find_comment_doc() {
        let actual: Vec<_> = find_comments("///aaa")
            .filter(|(Ok(s) | Err(s))| !s.is_empty())
            .collect();

        assert_eq!(actual, [Ok("///aaa")]);
    }
}
