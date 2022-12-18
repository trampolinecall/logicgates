use super::error::{CompileError, Report};

#[derive(PartialEq, Debug)]
pub(crate) enum Token<'file> {
    EOF,
    Dot,
    Equals,
    OBrack,
    CBrack,
    Comma,
    Number(u32),
    Semicolon,
    Identifier(&'file str),
}

#[derive(Debug, PartialEq)]
pub(crate) enum LexError {
    BadChar(char),
}

impl From<LexError> for CompileError {
    fn from(le: LexError) -> Self {
        match le {
            LexError::BadChar(c) => CompileError { message: format!("bad character: '{c}'") },
        }
    }
}

struct Lexer<'file>(&'file str, std::iter::Peekable<std::str::CharIndices<'file>>);

impl<'file> Lexer<'file> {
    fn slice(&mut self, start: usize) -> &'file str {
        if let Some((end, _)) = self.1.peek() {
            &self.0[start..*end]
        } else {
            &self.0[start..]
        }
    }

    fn peek_is_digit(&mut self) -> bool {
        matches!(self.1.peek(), Some((_, '0'..='9')))
    }

    fn peek_in_identifier(&mut self) -> bool {
        match self.1.peek() {
            // TODO: this is duplicated code from the first few match arms of the main lexer loop
            Some((_, ' ' | '\n' | ',' | '[' | ']' | '=' | '.' | ';')) => false,
            _ => true,
        }
    }
}

impl<'file> Iterator for Lexer<'file> {
    type Item = Token<'file>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some((start_i, start_c)) = self.1.next() else {
            return Some(Token::EOF)
        };

        let res = match start_c {
            ',' => Ok(Some(Token::Comma)),
            '[' => Ok(Some(Token::OBrack)),
            ']' => Ok(Some(Token::CBrack)),
            '=' => Ok(Some(Token::Equals)),
            '.' => Ok(Some(Token::Dot)),
            ';' => Ok(Some(Token::Semicolon)),

            '0'..='9' => {
                while self.peek_is_digit() {
                    self.1.next();
                }

                let number = self.slice(start_i).parse().expect("integer parse error impossible because slice only contains digits");
                Ok(Some(Token::Number(number)))
            }

            'a'..='z' | 'A'..='Z' => {
                while self.peek_in_identifier() {
                    self.1.next();
                }

                Ok(Some(Token::Identifier(self.slice(start_i))))
            }

            ' ' | '\n' => Ok(None),

            _ => Err(LexError::BadChar(start_c)),
        };

        match res {
            Ok(Some(t)) => Some(t),
            Ok(None) => self.next(),
            Err(e) => {
                e.report();
                self.next()
            }
        }
    }
}

pub(crate) fn lex(file: &str) -> impl Iterator<Item = Token> + '_ {
    Lexer(file, file.char_indices().peekable())
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
        assert_eq!(l.next(), Some(Token::Number(1)));
        assert_eq!(l.next(), Some(Token::Number(2)));
        assert_eq!(l.next(), Some(Token::Number(123)));
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
    fn whitespace() {
        let mut l = lex("    abc\n   2");
        assert_eq!(l.next(), Some(Token::Identifier("abc")));
        assert_eq!(l.next(), Some(Token::Number(2)));
        assert_eq!(l.next(), Some(Token::EOF));
    }
}
