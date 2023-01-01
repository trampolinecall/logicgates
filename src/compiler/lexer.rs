use super::error::{CompileError, Report};

#[derive(PartialEq, Debug)]
pub(crate) enum Token<'file> {
    EOF,

    OBrack,
    CBrack,
    Semicolon,

    Dot,
    Comma,

    Equals,
    Arrow,

    Let,
    Inline,
    Bundle,
    Inputs,
    Outputs,

    Backtick,

    Number(&'file str, usize),
    Identifier(&'file str),
    // TODO: variadic arguments / bundles
}

impl<'file> Token<'file> {
    pub(crate) fn as_number(&self) -> Option<&usize> {
        if let Self::Number(_, v) = self {
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

    /// Returns `true` if the token is [`EOF`].
    ///
    /// [`EOF`]: Token::EOF
    #[must_use]
    pub(crate) fn is_eof(&self) -> bool {
        matches!(self, Self::EOF)
    }

    /// Returns `true` if the token is [`OBrack`].
    ///
    /// [`OBrack`]: Token::OBrack
    #[must_use]
    pub(crate) fn is_obrack(&self) -> bool {
        matches!(self, Self::OBrack)
    }

    /// Returns `true` if the token is [`CBrack`].
    ///
    /// [`CBrack`]: Token::CBrack
    #[must_use]
    pub(crate) fn is_cbrack(&self) -> bool {
        matches!(self, Self::CBrack)
    }

    /// Returns `true` if the token is [`Semicolon`].
    ///
    /// [`Semicolon`]: Token::Semicolon
    #[must_use]
    pub(crate) fn is_semicolon(&self) -> bool {
        matches!(self, Self::Semicolon)
    }

    /// Returns `true` if the token is [`Dot`].
    ///
    /// [`Dot`]: Token::Dot
    #[must_use]
    pub(crate) fn is_dot(&self) -> bool {
        matches!(self, Self::Dot)
    }

    /// Returns `true` if the token is [`Comma`].
    ///
    /// [`Comma`]: Token::Comma
    #[must_use]
    pub(crate) fn is_comma(&self) -> bool {
        matches!(self, Self::Comma)
    }

    /// Returns `true` if the token is [`Equals`].
    ///
    /// [`Equals`]: Token::Equals
    #[must_use]
    pub(crate) fn is_equals(&self) -> bool {
        matches!(self, Self::Equals)
    }

    /// Returns `true` if the token is [`Arrow`].
    ///
    /// [`Arrow`]: Token::Arrow
    #[must_use]
    pub(crate) fn is_arrow(&self) -> bool {
        matches!(self, Self::Arrow)
    }

    /// Returns `true` if the token is [`Let`].
    ///
    /// [`Let`]: Token::Let
    #[must_use]
    pub(crate) fn is_let(&self) -> bool {
        matches!(self, Self::Let)
    }

    /// Returns `true` if the token is [`Inline`].
    ///
    /// [`Inline`]: Token::Inline
    #[must_use]
    pub(crate) fn is_inline(&self) -> bool {
        matches!(self, Self::Inline)
    }

    /// Returns `true` if the token is [`Bundle`].
    ///
    /// [`Bundle`]: Token::Bundle
    #[must_use]
    pub(crate) fn is_bundle(&self) -> bool {
        matches!(self, Self::Bundle)
    }

    /// Returns `true` if the token is [`Inputs`].
    ///
    /// [`Inputs`]: Token::Inputs
    #[must_use]
    pub(crate) fn is_inputs(&self) -> bool {
        matches!(self, Self::Inputs)
    }

    /// Returns `true` if the token is [`Outputs`].
    ///
    /// [`Outputs`]: Token::Outputs
    #[must_use]
    pub(crate) fn is_outputs(&self) -> bool {
        matches!(self, Self::Outputs)
    }

    /// Returns `true` if the token is [`Backtick`].
    ///
    /// [`Backtick`]: Token::Backtick
    #[must_use]
    pub(crate) fn is_backtick(&self) -> bool {
        matches!(self, Self::Backtick)
    }

    /// Returns `true` if the token is [`Number`].
    ///
    /// [`Number`]: Token::Number
    #[must_use]
    pub(crate) fn is_number(&self) -> bool {
        matches!(self, Self::Number(..))
    }

    /// Returns `true` if the token is [`Identifier`].
    ///
    /// [`Identifier`]: Token::Identifier
    #[must_use]
    pub(crate) fn is_identifier(&self) -> bool {
        matches!(self, Self::Identifier(..))
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
            Token::Semicolon => write!(f, "';'"),

            Token::Dot => write!(f, "'.'"),
            Token::Comma => write!(f, "','"),

            Token::Equals => write!(f, "'='"),
            Token::Arrow => write!(f, "'->'"),

            Token::Let => write!(f, "'let'"),
            Token::Inline => write!(f, "'inline'"),
            Token::Bundle => write!(f, "'bundle'"),
            Token::Inputs => write!(f, "'inputs'"),
            Token::Outputs => write!(f, "'outputs'"),

            Token::Backtick => write!(f, "'`'"),

            Token::Number(_, n) => write!(f, "'{n}'"),
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
            Some((_, ' ' | '\n' | ',' | '[' | ']' | '=' | '.' | ';')) => false,
            None => false,
            _ => true,
        }
    }

    fn next_tok(&mut self) -> Token<'file> {
        let Some((start_i, start_c)) = self.1.next() else {
            return Token::EOF
        };

        let res = match start_c {
            '[' => Ok(Some(Token::OBrack)),
            ']' => Ok(Some(Token::CBrack)),
            ';' => Ok(Some(Token::Semicolon)),

            '.' => Ok(Some(Token::Dot)),
            ',' => Ok(Some(Token::Comma)),

            '=' => Ok(Some(Token::Equals)),
            '-' if matches!(self.1.peek(), Some((_, '>'))) => {
                self.1.next();
                Ok(Some(Token::Arrow))
            }

            '`' => Ok(Some(Token::Backtick)),

            '0'..='9' => {
                while self.peek_is_digit() {
                    self.1.next();
                }

                let number_str = self.slice(start_i);
                let number = number_str.parse().expect("integer parse error impossible because slice only contains digits");
                Ok(Some(Token::Number(number_str, number)))
            }

            'a'..='z' | 'A'..='Z' => {
                while self.peek_in_identifier() {
                    self.1.next();
                }

                match self.slice(start_i) {
                    "let" => Ok(Some(Token::Let)),
                    "inline" => Ok(Some(Token::Inline)),
                    "bundle" => Ok(Some(Token::Bundle)),
                    "inputs" => Ok(Some(Token::Inputs)),
                    "outputs" => Ok(Some(Token::Outputs)),
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
        assert_eq!(l.next(), Some(Token::Number(2)));
        assert_eq!(l.next(), Some(Token::EOF));
    }
}
