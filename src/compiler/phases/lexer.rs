use crate::compiler::{
    data::token::{self, Token},
    error::{CompileError, File, Report, Span},
};

#[derive(Debug, PartialEq)]
pub(crate) struct LexError<'file>(pub(crate) Span<'file>, pub(crate) char);

impl<'file> From<LexError<'file>> for CompileError<'file> {
    fn from(le: LexError<'file>) -> Self {
        match le {
            LexError(sp, c) => CompileError::new(sp, format!("bad character: '{c}'")),
        }
    }
}

struct Lexer<'file>(&'file File, std::iter::Peekable<std::str::CharIndices<'file>>);

impl<'file> Lexer<'file> {
    fn span(&mut self, start: usize) -> Span<'file> {
        if let Some((end, _)) = self.1.peek() {
            self.0.span(start, *end)
        } else {
            self.0.span_to_end(start)
        }
    }

    fn slice(&mut self, start: usize) -> &'file str {
        if let Some((end, _)) = self.1.peek() {
            &self.0.contents[start..*end]
        } else {
            &self.0.contents[start..]
        }
    }

    fn peek_is_digit(&mut self) -> bool {
        matches!(self.1.peek(), Some((_, '0'..='9')))
    }

    fn peek_in_identifier(&mut self) -> bool {
        match self.1.peek() {
            // TODO: this is duplicated code from the first few match arms of the main lexer loop
            Some((_, ' ' | '\n' | ',' | '[' | ']' | '=' | '.' | ';')) | None => false,
            _ => true,
        }
    }

    fn next_tok(&mut self) -> Token<'file> {
        let Some((start_i, start_c)) = self.1.next() else {
            return Token::EOF({
                self.0.eof_span()
            });
        };

        let res = match start_c {
            '[' => Ok(Some(Token::OBrack(self.span(start_i)))),
            ']' => Ok(Some(Token::CBrack(self.span(start_i)))),
            ';' => Ok(Some(Token::Semicolon(self.span(start_i)))),

            '.' => Ok(Some(Token::Dot(self.span(start_i)))),
            ',' => Ok(Some(Token::Comma(self.span(start_i)))),

            '-' => Ok(Some(Token::Dash(self.span(start_i)))),
            '=' => Ok(Some(Token::Equals(self.span(start_i)))),

            '\\' => Ok(Some(Token::Backslash(self.span(start_i)))),
            '/' => Ok(Some(Token::Slash(self.span(start_i)))),

            '0'..='9' => {
                while self.peek_is_digit() {
                    self.1.next();
                }

                let span = self.span(start_i);
                let number_str = self.slice(start_i);
                let number = number_str.parse().expect("integer parse error impossible because slice only contains digits");
                Ok(Some(Token::Number(span, number_str, number)))
            }

            'a'..='z' | 'A'..='Z' => {
                while self.peek_in_identifier() {
                    self.1.next();
                }

                let span = self.span(start_i);
                let slice = self.slice(start_i);
                match slice {
                    "let" => Ok(Some(Token::Let(span))),
                    "connect" => Ok(Some(Token::Connect(span))),
                    "inline" => Ok(Some(Token::Inline(span))),
                    "struct" => Ok(Some(Token::Struct(span))),
                    name => Ok(Some(Token::PlainIdentifier(token::PlainIdentifier { span, name }))),
                }
            }

            ' ' | '\n' => Ok(None),

            _ => Err(LexError(self.span(start_i), start_c)),
        };

        match res {
            Ok(Some(t)) => t,
            Ok(None) => self.next_tok(),
            Err(e) => {
                e.report();
                self.next_tok()
            }
        }
    }
}

impl<'file> Iterator for Lexer<'file> {
    type Item = Token<'file>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_tok())
    }
}

pub(crate) fn lex(file: &File) -> impl Iterator<Item = Token> + '_ {
    let mut l = Lexer(file, file.contents.char_indices().peekable()).peekable();

    std::iter::from_fn(move || {
        let n = l.next().unwrap();
        if let Token::PlainIdentifier(_) = l.peek().expect("lexer should never return None") {
            match n {
                Token::Dash(dash_sp) => {
                    let Token::PlainIdentifier(i) = l.next().expect("lexer should never return None") else { unreachable!() };
                    return Some(Token::TypeIdentifier(token::TypeIdentifier { span: dash_sp + i.span, name: i.name, with_tag: "-".to_string() + i.name }));
                }
                Token::Backslash(backsl_sp) => {
                    let Token::PlainIdentifier(i) = l.next().expect("lexer should never return None") else { unreachable!() };
                    return Some(Token::CircuitIdentifier(token::CircuitIdentifier { span: backsl_sp + i.span, name: i.name, with_tag: "\\".to_string() + i.name }));
                }

                _ => {}
            }
        }

        Some(n)
    })
}

#[cfg(test)]
mod test {
    use crate::compiler::{
        error::File,
        phases::lexer::{lex, token, Token},
    };

    fn check_lexer_output<'file>(mut l: impl Iterator<Item = Token<'file>>, expected: impl IntoIterator<Item = Token<'file>>) {
        let mut expected = expected.into_iter();
        loop {
            let l_next = l.next();
            let expected_next = expected.next();

            assert_eq!(l_next, expected_next);

            if let Some(Token::EOF(_)) | None = l_next {
                break;
            }
        }
    }

    #[test]
    fn punctuation() {
        let mut file = File::test_file();
        let expected = make_spans!(file,
            [
                ("[", sp => Token::OBrack(sp)),
                ("]", sp => Token::CBrack(sp)),
                (";", sp => Token::Semicolon(sp)),
                (".", sp => Token::Dot(sp)),
                (",", sp => Token::Comma(sp)),
                ("-", sp => Token::Dash(sp)),
                ("=", sp => Token::Equals(sp)),
                ("\\", sp => Token::Backslash(sp)),
                ("/", sp => Token::Slash(sp)),
            ],
            sp => Token::EOF(sp),
        );

        check_lexer_output(lex(&file), expected);
    }

    #[test]
    fn numbers() {
        let mut file = File::test_file();
        let expected = make_spans!(file,
            [
                ("1", sp => Some(Token::Number(sp, "1", 1))),
                (" ", _sp => None),
                ("2", sp => Some(Token::Number(sp, "2", 2))),
                (" ", _sp => None),
                ("123", sp => Some(Token::Number(sp, "123", 123))),
            ],
            sp => Some(Token::EOF(sp)),
        );

        check_lexer_output(lex(&file), expected.into_iter().flatten());
    }

    #[test]
    fn identifiers() {
        let mut file = File::test_file();
        let expected = make_spans!(file,
            [
                ("a", sp => Some(Token::PlainIdentifier(token::PlainIdentifier {name: "a", span: sp}))),
                (" ", _sp => None),
                ("abc", sp => Some(Token::PlainIdentifier(token::PlainIdentifier {name: "abc", span: sp}))),
                (" ", _sp => None),
                ("abc87", sp => Some(Token::PlainIdentifier(token::PlainIdentifier {name: "abc87", span: sp}))),
                (" ", _sp => None),
                ("abC-'()", sp => Some(Token::PlainIdentifier(token::PlainIdentifier {name: "abC-'()", span: sp}))),
            ],
            sp => Some(Token::EOF(sp)),
        );

        check_lexer_output(lex(&file), expected.into_iter().flatten());
    }

    #[test]
    fn whitespace() {
        let mut file = File::test_file();
        let expected = make_spans!(file,
            [
                ("    ", _sp => None),
                ("abc", sp => Some(Token::PlainIdentifier(token::PlainIdentifier {name: "abc", span: sp}))),
                ("\n", _sp => None),
                ("   ", _sp => None),
                ("2", sp => Some(Token::Number(sp, "2", 2)))
            ],
            sp => Some(Token::EOF(sp)),
        );

        check_lexer_output(lex(&file), expected.into_iter().flatten());
    }
    // TODO: test keywords
}
