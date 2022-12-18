use super::error::{CompileError, Report};

#[derive(PartialEq, Debug)]
pub(crate) enum Token<'file> {
    EOF,
    OBrack,
    CBrack,
    Semicolon,
    Colon,
    Dot,
    Comma,
    Equals,
    Let,
    Number(u32),
    Identifier(&'file str),
}

impl<'file> Token<'file> {
    pub(crate) fn as_number(&self) -> Option<&u32> {
        if let Self::Number(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(crate) fn as_identifier(&self) -> Option<&&'file str> {
        if let Self::Identifier(v) = self {
            Some(v)
        } else {
            None
        }
    }
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

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::EOF => write!(f, "eof"),
            Token::OBrack => write!(f, "'['"),
            Token::CBrack => write!(f, "']'"),
            Token::Dot => write!(f, "'.'"),
            Token::Equals => write!(f, "'='"),
            Token::Comma => write!(f, "','"),
            Token::Semicolon => write!(f, "';'"),
            Token::Colon => write!(f, "':'"),
            Token::Let => write!(f, "'let'"),
            Token::Number(n) => write!(f, "'{n}'"),
            Token::Identifier(i) => write!(f, "'{i}'"),
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
            Some((_, ' ' | '\n' | ',' | '[' | ']' | '=' | '.' | ';' | ':')) => false,
            None => false,
            _ => true,
        }
    }

    fn next_tok(&mut self) -> Token<'file> {
        let Some((start_i, start_c)) = self.1.next() else {
            return Token::EOF
        };

        let res = match start_c {
            ',' => Ok(Some(Token::Comma)),
            '[' => Ok(Some(Token::OBrack)),
            ']' => Ok(Some(Token::CBrack)),
            ':' => Ok(Some(Token::Colon)),
            ';' => Ok(Some(Token::Semicolon)),
            '.' => Ok(Some(Token::Dot)),
            '=' => Ok(Some(Token::Equals)),

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

                match self.slice(start_i) {
                    "let" => Ok(Some(Token::Let)),
                    iden => Ok(Some(Token::Identifier(iden))),
                }
            }

            ' ' | '\n' => Ok(None),

            _ => Err(LexError::BadChar(start_c)),
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
