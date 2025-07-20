use {
    crate::html,
    proc_macro2::{Span, TokenStream, TokenTree},
    std::{collections::HashSet, fmt::Write, iter},
};

pub fn highlight(code: &str) -> Result<String, syn::Error> {
    let stream = syn::parse_str(code)?;
    let mut tokens = Tokens(vec![]);
    parse(code, stream, &mut tokens);

    let mut output = String::new();
    let mut last = 0;
    for token in tokens.0 {
        let range = token.span.byte_range();
        escape_with_comments(&code[last..range.start], &mut output);
        _ = write!(&mut output, "<span class=\"{}\">", token.kind.class());
        html::escape(&code[range.start..range.end], &mut output);
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
                html::escape(s, output);
                output.push_str("</span>");
            }
            Err(s) => html::escape(s, output),
        }
    }
}

fn find_comments(mut code: &str) -> impl Iterator<Item = Result<&str, &str>> {
    enum State {
        Skip,
        Comment(Comment),
        End,
    }

    #[derive(Clone, Copy)]
    enum Comment {
        Slash,
        Star,
    }

    let mut state = State::Skip;

    let patterns = [(*b"//", Comment::Slash), (*b"/*", Comment::Star)]
        .map(|(pattern, comment)| move |w| (w == pattern).then_some(comment));

    iter::from_fn(move || match state {
        State::Skip => {
            let Some((pos, comment)) = find(code, patterns) else {
                state = State::End;
                return Some(Err(code));
            };

            let (left, right) = code.split_at(pos);
            code = right;

            state = State::Comment(comment);
            Some(Err(left))
        }
        State::Comment(comment) => {
            let pos = match comment {
                Comment::Slash => {
                    let f = |w| (w == *b"\n").then_some(());
                    let Some((pos, ())) = find(code, [f]) else {
                        state = State::End;
                        return Some(Ok(code));
                    };

                    pos + 1
                }
                Comment::Star => {
                    let f = |w| (w == *b"*/").then_some(());
                    let Some((pos, ())) = find(code, [f]) else {
                        state = State::End;
                        return Some(Ok(code));
                    };

                    pos + 2
                }
            };

            let (left, right) = code.split_at(pos);
            code = right;

            state = State::Skip;
            Some(Ok(left))
        }
        State::End => None,
    })
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

#[derive(Clone, Copy)]
enum Kind {
    Static,
    Keyword,
    Literal,
    Typing,
    Generic,
    Ident,
    Exclamation,
    Apostrophe,
    Macro,
    Lifetime,
}

impl Kind {
    fn class(self) -> &'static str {
        match self {
            Self::Static | Self::Keyword => "kw",
            Self::Literal => "li",
            Self::Typing => "ty",
            Self::Generic => "ge",
            Self::Ident => "id",
            Self::Exclamation => "ex",
            Self::Apostrophe => "ap",
            Self::Macro => "mc",
            Self::Lifetime => "lt",
        }
    }
}

struct Token {
    kind: Kind,
    span: Span,
}

struct Tokens(Vec<Token>);

impl Tokens {
    fn push(&mut self, token: Token) {
        let token = if let Some(last) = self
            .0
            .pop_if(|last| matches!((last.kind, token.kind), (Kind::Ident, Kind::Exclamation)))
        {
            let span = last.span.join(token.span).expect("join spans");
            Token {
                kind: Kind::Macro,
                span,
            }
        } else if let Some(last) = self.0.pop_if(|last| {
            matches!(
                (last.kind, token.kind),
                (Kind::Apostrophe, Kind::Ident | Kind::Static),
            )
        }) {
            let span = last.span.join(token.span).expect("join spans");
            Token {
                kind: Kind::Lifetime,
                span,
            }
        } else {
            token
        };

        self.0.push(token);
    }
}

fn parse(code: &str, stream: TokenStream, tokens: &mut Tokens) {
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
                    "static" => Kind::Static,
                    s if is_keyword(s) => Kind::Keyword,
                    s if is_generic(s) => Kind::Generic,
                    s if is_typing(s) => Kind::Typing,
                    _ => Kind::Ident,
                };

                tokens.push(Token { kind, span });
            }
            TokenTree::Punct(punct) => {
                let span = punct.span();
                let kind = match &code[span.byte_range()] {
                    "!" => Kind::Exclamation,
                    "'" => Kind::Apostrophe,
                    _ => continue,
                };

                tokens.push(Token { kind, span });
            }
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
