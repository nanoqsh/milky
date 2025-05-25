use {
    pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd},
    std::{
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
    println!("{}", make_html(&input));
    Ok(())
}

fn make_html(input: &str) -> String {
    let mut html = String::new();
    let mut code = None;

    for event in Parser::new(&input) {
        match event {
            Event::Start(Tag::Paragraph) => html.push_str("<p>"),
            Event::Start(Tag::Heading { level, .. }) => _ = write!(&mut html, "<{level}>"),
            Event::Start(Tag::BlockQuote(_)) => todo!(),
            Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => html.push_str("<pre>"),
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(s))) => {
                html.push_str("<pre>");
                if &*s == "rust" {
                    let len = html.len();
                    code = Some((len, len));
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
                _ = write!(&mut html, "<a href=\"{dest_url}\">")
            }
            Event::Start(Tag::Image { .. }) => todo!(),
            Event::Start(Tag::MetadataBlock(_)) => todo!(),
            Event::End(TagEnd::Paragraph) => html.push_str("</p>"),
            Event::End(TagEnd::Heading(level)) => _ = write!(&mut html, "</{level}>"),
            Event::End(TagEnd::BlockQuote(_)) => todo!(),
            Event::End(TagEnd::CodeBlock) => {
                if let Some((start, end)) = code.take() {
                    let highlighted = highlight_rust(&html[start..end]);
                    html.truncate(start);
                    html.push_str(&highlighted);
                }

                html.push_str("</pre>")
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
                html.push_str(&s);
                if let Some((start, _)) = code {
                    code = Some((start, html.len()));
                }
            }
            Event::Code(s) => _ = write!(&mut html, "<code>{s}</code>"),
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

fn highlight_rust(code: &str) -> String {
    format!("RUST!{code}RUST!")
}
