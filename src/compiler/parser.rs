pub(crate) mod ast;

use crate::compiler::error::CompileError;
use crate::compiler::error::Report;
use crate::compiler::lexer::Token;
use std::iter::Peekable;

struct Parser<'file, T: Iterator<Item = Token<'file>>> {
    tokens: Peekable<T>,
}

#[derive(Debug, PartialEq)]
struct ParseError {
    expected: &'static str,
    got: String,
}

impl From<ParseError> for CompileError {
    fn from(ParseError { expected, got }: ParseError) -> Self {
        CompileError { message: format!("expected {expected}, got {got}") }
    }
}

impl<'file, T: Iterator<Item = Token<'file>>> Parser<'file, T> {
    fn peek(&mut self) -> &Token<'file> {
        self.tokens.peek().unwrap()
    }
    fn next(&mut self) -> Token<'file> {
        self.tokens.next().unwrap()
    }

    fn expect(&mut self, expected: &'static str, pred: impl FnOnce(&Token<'file>) -> bool) -> Result<Token<'file>, ParseError> {
        if pred(self.peek()) {
            Ok(self.next())
        } else {
            Err(self.expected(expected))
        }
    }
    fn maybe_consume(&mut self, pred: impl FnOnce(&Token<'file>) -> bool) -> Option<Token<'file>> {
        if pred(self.peek()) {
            Some(self.next())
        } else {
            None
        }
    }

    fn expected(&mut self, thing: &'static str) -> ParseError {
        ParseError { expected: thing, got: self.peek().to_string() }
    }

    fn list<A>(
        &mut self,
        start_str: &'static str,
        start: impl Fn(&Token<'file>) -> bool,
        delim: impl Fn(&Token<'file>) -> bool,
        ending_str: &'static str,
        ending: impl Fn(&Token<'file>) -> bool,
        thing: impl for<'p> FnMut(&'p mut Parser<'file, T>) -> Result<A, ParseError>,
    ) -> Result<Vec<A>, ParseError> {
        self.expect(start_str, start)?;
        self.finish_list(delim, ending_str, ending, thing)
    }

    fn finish_list<A>(
        &mut self,
        delim: impl Fn(&Token<'file>) -> bool,
        ending_str: &'static str,
        ending: impl Fn(&Token<'file>) -> bool,
        mut thing: impl for<'p> FnMut(&'p mut Parser<'file, T>) -> Result<A, ParseError>,
    ) -> Result<Vec<A>, ParseError> {
        let mut items = Vec::new();

        while !ending(self.peek()) {
            items.push(thing(self)?);

            if delim(self.peek()) {
                self.next(); // there is a delimiter, the list may or may not continue
            } else {
                break; // if there is no delimiter, the list cannot be continued
            }
        }

        self.expect(ending_str, ending)?;

        Ok(items)
    }
}

impl<'file, T: Iterator<Item = Token<'file>>> Parser<'file, T> {
    fn parse(&mut self) -> Option<Vec<ast::Circuit<'file>>> {
        let mut circuits = Vec::new();
        while !matches!(self.peek(), Token::EOF) {
            match self.circuit() {
                Ok(circuit) => circuits.push(circuit),
                Err(e) => {
                    self.next();
                    e.report();
                }
            }
        }

        Some(circuits)
    }

    fn circuit(&mut self) -> Result<ast::Circuit<'file>, ParseError> {
        let _ = self.expect("circuit name (starting with '`')", |tok| matches!(tok, Token::Backtick))?;
        let name = self.expect("circuit name after '`'", |tok| matches!(tok, Token::Identifier(_)))?;
        let name = *name.as_identifier().unwrap();
        let arguments = self.list("'['", Token::is_obrack, Token::is_comma, "']'", Token::is_cbrack, |parser| {
            let pattern = parser.pattern()?;
            parser.expect("';' for type annotation", |tok| matches!(tok, Token::Semicolon))?;
            let type_ = parser.type_()?;
            Ok((pattern, type_))
        })?;
        let mut lets = Vec::new();

        while matches!(self.peek(), Token::Let) {
            lets.push(self.r#let()?);
        }

        let ret = self.expr()?;

        Ok(ast::Circuit { name, inputs: arguments, lets, outputs: ret })
    }

    fn r#let(&mut self) -> Result<ast::Let<'file>, ParseError> {
        self.expect("'let'", |tok| matches!(tok, Token::Let))?;
        let pat = self.pattern()?;

        self.expect("';' for type annotation", Token::is_semicolon)?;
        let type_ = self.type_()?;

        self.expect("'='", |tok| matches!(tok, Token::Equals))?;

        let val = self.expr()?;
        Ok(ast::Let { pat, type_, val })
    }

    fn pattern(&mut self) -> Result<ast::Pattern<'file>, ParseError> {
        match self.peek() {
            Token::Identifier(_) => {
                let iden = self.next();
                let iden = *iden.as_identifier().unwrap();

                /*
                let size = if matches!(self.peek(), Token::Semicolon) {
                    self.next();
                    *self.expect("pattern size", |tok| matches!(tok, Token::Number(_)))?.as_number().unwrap()
                } else {
                    1
                };
                */

                Ok(ast::Pattern(iden))
            }

            /*
            Token::OBrack => {
                self.next();

                let mut patterns = Vec::new();

                if !matches!(self.peek(), Token::CBrack) {
                    patterns.extend(self.pattern()?);
                    while matches!(self.peek(), Token::Comma) {
                        self.next();
                        patterns.extend(self.pattern()?);
                    }
                }

                self.expect("']'", |tok| matches!(tok, Token::CBrack))?;

                Ok(patterns)
            }
            */
            _ => Err(self.expected("pattern")),
        }
    }

    fn expr(&mut self) -> Result<ast::Expr<'file>, ParseError> {
        let primary = self.primary_expr()?;

        if matches!(self.peek(), Token::Semicolon) {
            self.next();
            let index = self.expect("get index", |tok| matches!(tok, Token::Number(_)))?;
            let index = index.as_number().unwrap();
            Ok(ast::Expr::Get(Box::new(primary), *index))
        } else {
            Ok(primary)
        }
    }

    fn primary_expr(&mut self) -> Result<ast::Expr<'file>, ParseError> {
        match self.peek() {
            Token::Number(_) => {
                let n = self.next();
                let n = n.as_number().unwrap();

                match n {
                    0 => Ok(ast::Expr::Const(false)),
                    1 => Ok(ast::Expr::Const(true)),
                    _ => Err(self.expected("'0' or '1'")),
                }
            }
            Token::Backtick => {
                let _ = self.next();
                let i = self.expect("circuit name after '`'", |tok| matches!(tok, Token::Identifier(_)))?;
                let i = i.as_identifier().unwrap();

                let inline = self.maybe_consume(|tok| matches!(tok, Token::Inline)).is_some();

                let args = self.expr()?;

                Ok(ast::Expr::Call(i, inline, Box::new(args)))
            }

            Token::Identifier(_) => {
                let i = self.next();
                let i = i.as_identifier().unwrap();

                Ok(ast::Expr::Ref(i))
            }

            Token::OBrack => {
                self.next();

                let mut items = Vec::new();

                if !matches!(self.peek(), Token::CBrack) {
                    items.push(self.expr()?);
                    while matches!(self.peek(), Token::Comma) {
                        self.next();
                        items.push(self.expr()?);
                    }
                }

                self.expect("']'", |tok| matches!(tok, Token::CBrack))?;

                Ok(ast::Expr::Multiple(items))
            }

            _ => Err(self.expected("expression"))?,
        }
    }

    fn type_(&mut self) -> Result<ast::Type, ParseError> {
        match self.peek() {
            Token::Backtick => {
                let _ = self.next();
                Ok(ast::Type::Bit)
            }

            Token::OBrack => {
                let _ = self.next();
                let len = self.expect("array length", |tok| matches!(tok, Token::Number(_)))?;
                let len = len.as_number().unwrap();
                let _ = self.expect("']'", |tok| matches!(tok, Token::CBrack))?;


                let ty = self.type_()?;

                Ok(ast::Type::Array(*len, Box::new(ty)))
            }

            _ => Err(self.expected("type"))
        }
    }
}

pub(crate) fn parse<'file>(tokens: impl Iterator<Item = Token<'file>>) -> Option<Vec<ast::Circuit<'file>>> {
    Parser { tokens: tokens.peekable() }.parse()
}

#[cfg(test)]
mod test {
    use super::ast;
    use super::parse;
    use super::Parser;
    use crate::compiler::lexer::Token;

    use std::iter::Peekable;

    fn make_token_stream(tokens: Vec<Token>) -> Peekable<impl Iterator<Item = Token>> {
        tokens.into_iter().chain(std::iter::repeat_with(|| Token::EOF)).peekable()
    }

    #[test]
    fn list() {
        let tokens = vec![Token::OBrack, Token::Identifier("a"), Token::Comma, Token::Identifier("b"), Token::CBrack];
        assert_eq!(
            Parser { tokens: make_token_stream(tokens) }.list("'['", |tok| matches!(tok, Token::OBrack), |tok| matches!(tok, Token::Comma), "']'", |tok| matches!(tok, Token::CBrack), Parser::pattern),
            Ok(vec![ast::Pattern("a"), ast::Pattern("b")])
        )
    }

    #[test]
    fn list_trailing_delim() {
        let tokens = vec![Token::OBrack, Token::Identifier("a"), Token::Comma, Token::Identifier("b"), Token::Comma, Token::CBrack];
        assert_eq!(
            Parser { tokens: make_token_stream(tokens) }.list("'['", |tok| matches!(tok, Token::OBrack), |tok| matches!(tok, Token::Comma), "']'", |tok| matches!(tok, Token::CBrack), Parser::pattern),
            Ok(vec![ast::Pattern("a"), ast::Pattern("b")])
        )
    }

    // TODO: test inline calls
    #[test]
    fn circuit() {
        /*
        `thingy [arg; `]
            let res = `and [arg, arg]
            res
        */
        let tokens = vec![
            Token::Backtick,
            Token::Identifier("thingy"),
            Token::OBrack,
            Token::Identifier("arg"),
            Token::Semicolon,
            Token::Backtick,
            Token::CBrack,
            Token::Let,
            Token::Identifier("res"),
            Token::Equals,
            Token::Backtick,
            Token::Identifier("and"),
            Token::OBrack,
            Token::Identifier("arg"),
            Token::Comma,
            Token::Identifier("arg"),
            Token::CBrack,
            Token::Identifier("res"),
        ];

        assert_eq!(
            parse(make_token_stream(tokens)),
            Some(vec![ast::Circuit {
                name: "thingy",
                inputs: vec![(ast::Pattern("arg"), ast::Type::Bit)],
                lets: vec![ast::Let { pat: ast::Pattern("res"), val: ast::Expr::Call("and", false, Box::new(ast::Expr::Multiple(vec![ast::Expr::Ref("arg"), ast::Expr::Ref("arg")]))) }],
                outputs: ast::Expr::Ref("res")
            }])
        )
    }

    #[test]
    fn r#let() {
        let tokens = vec![Token::Let, Token::Identifier("a"), Token::Equals, Token::Identifier("b")];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.r#let(), Ok(ast::Let { pat: ast::Pattern("a"), val: ast::Expr::Ref("b") }))
    }

    #[test]
    fn iden_pattern() {
        let tokens = vec![Token::Identifier("iden")];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.pattern(), Ok(ast::Pattern("iden")))
    }

    #[test]
    fn const_false_expr() {
        let tokens = vec![Token::Number(0)];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.expr(), Ok(ast::Expr::Const(false)))
    }

    #[test]
    fn const_true_expr() {
        let tokens = vec![Token::Number(1)];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.expr(), Ok(ast::Expr::Const(true)))
    }

    #[test]
    fn call_expr() {
        let tokens = vec![Token::Backtick, Token::Identifier("a"), Token::Identifier("b")];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.expr(), Ok(ast::Expr::Call("a", false, Box::new(ast::Expr::Ref("b")))))
    }

    #[test]
    fn iden_expr() {
        let tokens = vec![Token::Identifier("a")];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.expr(), Ok(ast::Expr::Ref("a")))
    }

    #[test]
    fn multiple_expr() {
        let tokens = vec![Token::OBrack, Token::Identifier("a"), Token::Comma, Token::Identifier("b"), Token::Comma, Token::Number(0), Token::Comma, Token::Number(1), Token::CBrack];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.expr(), Ok(ast::Expr::Multiple(vec![ast::Expr::Ref("a"), ast::Expr::Ref("b"), ast::Expr::Const(false), ast::Expr::Const(true)])))
    }
}
