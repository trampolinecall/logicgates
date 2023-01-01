pub(crate) mod token;

use super::error::{CompileError, Report, Span, File};

pub(crate) use token::Token;

#[derive(Debug, PartialEq)]
pub(crate) enum LexError<'file> {
    BadChar(Span<'file>, char),
}

impl<'file> From<LexError<'file>> for CompileError<'file> {
    fn from(le: LexError<'file>) -> Self {
        match le {
            LexError::BadChar(sp, c) => CompileError::new(sp, format!("bad character: '{c}'")),
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

    fn peek_is_digit(&mut self) -> bool {
        matches!(self.1.peek(), Some((_, '0'..='9')))
    }

    fn peek_in_identifier(&mut self) -> bool {
        match self.1.peek() {
            // TODO: this is duplicated code from the first few match arms of the main lexer loop
            Some((_, ' ' | '\n' | ',' | '[' | ']' | '=' | '.' | ';')) => false,
            None => false,
            _ => true,
        }
    }

    fn next_tok(&mut self) -> Token<'file> {
        let Some((start_i, start_c)) = self.1.next() else {
            return Token::EOF({
                self.0.eof_span()
            })
        };

        let res = match start_c {
            '[' => Ok(Some(Token::OBrack(self.span(start_i)))),
            ']' => Ok(Some(Token::CBrack(self.span(start_i)))),
            ';' => Ok(Some(Token::Semicolon(self.span(start_i)))),

            '.' => Ok(Some(Token::Dot(self.span(start_i)))),
            ',' => Ok(Some(Token::Comma(self.span(start_i)))),

            '=' => Ok(Some(Token::Equals(self.span(start_i)))),
            '-' if matches!(self.1.peek(), Some((_, '>'))) => {
                self.1.next();
                Ok(Some(Token::Arrow(self.span(start_i))))
            }

            '`' => Ok(Some(Token::Backtick(self.span(start_i)))),

            '0'..='9' => {
                while self.peek_is_digit() {
                    self.1.next();
                }

                let span = self.span(start_i);
                let number_str = span.slice();
                let number = number_str.parse().expect("integer parse error impossible because slice only contains digits");
                Ok(Some(Token::Number(span, number)))
            }

            'a'..='z' | 'A'..='Z' => {
                while self.peek_in_identifier() {
                    self.1.next();
                }

                let span = self.span(start_i);
                let slice = span.slice();
                match slice {
                    "let" => Ok(Some(Token::Let(span))),
                    "inline" => Ok(Some(Token::Inline(span))),
                    "bundle" => Ok(Some(Token::Bundle(span))),
                    "inputs" => Ok(Some(Token::Inputs(span))),
                    "outputs" => Ok(Some(Token::Outputs(span))),
                    iden => Ok(Some(Token::Identifier(span, iden))),
                }
            }

            ' ' | '\n' => Ok(None),

            _ => Err(LexError::BadChar(self.span(start_i), start_c)),
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
    Lexer(file, file.contents.char_indices().peekable())
}

#[cfg(test)]
mod test {
    use super::lex;
    use super::Token;

    #[test]
    fn punctuation() {
        let mut l = lex(",[]=.;");
        assert_eq!(l.next(), Some(Token::Comma));
        assert_eq!(l.next(), Some(Token::OBrack));
        assert_eq!(l.next(), Some(Token::CBrack));
        assert_eq!(l.next(), Some(Token::Equals));
        assert_eq!(l.next(), Some(Token::Dot));
        assert_eq!(l.next(), Some(Token::Semicolon));
        assert_eq!(l.next(), Some(Token::EOF));
    }

    #[test]
    fn numbers() {
        let mut l = lex("1 2 123");
        assert_eq!(l.next(), Some(Token::Number("1", 1)));
        assert_eq!(l.next(), Some(Token::Number("2", 2)));
        assert_eq!(l.next(), Some(Token::Number("123", 123)));
        assert_eq!(l.next(), Some(Token::EOF));
    }

    #[test]
    fn identifiers() {
        let mut l = lex("a abc abc87 abC-'()");
        assert_eq!(l.next(), Some(Token::Identifier("a")));
        assert_eq!(l.next(), Some(Token::Identifier("abc")));
        assert_eq!(l.next(), Some(Token::Identifier("abc87")));
        assert_eq!(l.next(), Some(Token::Identifier("abC-'()")));
        assert_eq!(l.next(), Some(Token::EOF));
    }

    #[test]
    fn backticks_and_circuit_identifiers() {
        let mut l = lex("`a `abc `abc87 `abC-'()");
        assert_eq!(l.next(), Some(Token::Backtick));
        assert_eq!(l.next(), Some(Token::Identifier("a")));
        assert_eq!(l.next(), Some(Token::Backtick));
        assert_eq!(l.next(), Some(Token::Identifier("abc")));
        assert_eq!(l.next(), Some(Token::Backtick));
        assert_eq!(l.next(), Some(Token::Identifier("abc87")));
        assert_eq!(l.next(), Some(Token::Backtick));
        assert_eq!(l.next(), Some(Token::Identifier("abC-'()")));
        assert_eq!(l.next(), Some(Token::EOF));
    }

    #[test]
    fn whitespace() {
        let mut l = lex("    abc\n   2");
        assert_eq!(l.next(), Some(Token::Identifier("abc")));
        assert_eq!(l.next(), Some(Token::Number("2", 2)));
        assert_eq!(l.next(), Some(Token::EOF));
    }
}
