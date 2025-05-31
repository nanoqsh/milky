use {
    proc_macro2::{Span, TokenStream, TokenTree},
    pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd},
    std::{
        collections::HashSet,
        fmt::Write,
        io::{self, Error, Read},
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
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;
    let page = page(&article(&input), "The article name");
    println!("{page}");
    Ok(())
}

fn page(article: &str, title: &str) -> String {
    maud::html! {
        (maud::DOCTYPE)
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            link rel="stylesheet" href="style.css";
            title { (title) }
        }
        body {
            header {
                h1 { (title) }
            }
            article { (maud::PreEscaped(article)) }
        }
    }
    .into_string()
}

fn escape(s: &str, output: &mut String) {
    for c in s.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            _ => output.push(c),
        }
    }
}

fn article(input: &str) -> String {
    let mut html = String::new();
    let mut code = None;

    for event in Parser::new(input) {
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
            Event::Start(Tag::Image { .. }) => todo!(),
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
            Event::End(TagEnd::Image) => todo!(),
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
            Event::SoftBreak => todo!(),
            Event::HardBreak => todo!(),
            Event::Rule => todo!(),
            Event::TaskListMarker(_) => todo!(),
        }
    }

    html
}

fn highlight_rust(code: &str) -> Result<String, syn::Error> {
    let stream: TokenStream = syn::parse_str(code)?;
    let mut tokens = vec![];
    parse(code, stream, &mut tokens);

    let mut output = String::new();
    let mut last = 0;
    for token in tokens {
        let range = token.span.byte_range();
        escape(&code[last..range.start], &mut output);
        _ = write!(&mut output, "<span class=\"{}\">", token.kind.class());
        escape(&code[range.start..range.end], &mut output);
        output.push_str("</span>");
        last = range.end;
    }

    if let Some(s) = code.get(last..) {
        escape(s, &mut output);
    }

    Ok(output)
}

enum Kind {
    Keyword,
    Literal,
    Ident,
}

impl Kind {
    fn class(self) -> &'static str {
        match self {
            Self::Keyword => "kw",
            Self::Literal => "li",
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
                let s = &code[span.byte_range()];
                let kind = if is_keyword(s) {
                    Kind::Keyword
                } else {
                    Kind::Ident
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
