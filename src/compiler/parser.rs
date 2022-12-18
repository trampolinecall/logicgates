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

impl<'file, T: std::iter::Iterator<Item = Token<'file>>> Parser<'file, T> {
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

    fn expected(&mut self, thing: &'static str) -> ParseError {
        ParseError { expected: thing, got: self.peek().to_string() }
    }

    fn parse(&mut self) -> Option<Vec<ast::Gate<'file>>> {
        let mut gates = Vec::new();
        while !matches!(self.peek(), Token::EOF) {
            match self.parse_gate() {
                Ok(gate) => gates.push(gate),
                Err(e) => {
                    self.next();
                    e.report();
                }
            }
        }

        Some(gates)
    }

    fn parse_gate(&mut self) -> Result<ast::Gate<'file>, ParseError> {
        let name = self.expect("gate name", |tok| matches!(tok, Token::GateIdentifier(_)))?;
        let name = *name.as_gate_identifier().unwrap();
        let arguments = self.parse_pattern()?;
        self.expect("':'", |tok| matches!(tok, Token::Colon))?;
        let mut lets = Vec::new();

        while matches!(self.peek(), Token::Let) {
            lets.push(self.parse_let()?);
        }

        let ret = self.parse_expr()?;

        Ok(ast::Gate { name, arguments, lets, ret })
    }

    fn parse_let(&mut self) -> Result<ast::Let<'file>, ParseError> {
        self.expect("'let'", |tok| matches!(tok, Token::Let))?;
        let pat = self.parse_pattern()?;
        self.expect("'='", |tok| matches!(tok, Token::Equals))?;

        let val = self.parse_expr()?;
        Ok(ast::Let { pat, val })
    }

    fn parse_pattern(&mut self) -> Result<Vec<ast::Pattern<'file>>, ParseError> {
        // for now, only identifier patterns
        if matches!(self.peek(), Token::LocalIdentifier(_)) {
            let iden = self.next();
            let iden = *iden.as_local_identifier().unwrap();
            Ok(vec![ast::Pattern(iden, 1)])
        } else {
            Err(self.expected("pattern"))
        }
    }

    fn parse_expr(&mut self) -> Result<Vec<ast::Expr<'file>>, ParseError> {
        match self.peek() {
            Token::Number(_) => {
                let n = self.next();
                let n = n.as_number().unwrap();

                match n {
                    0 => Ok(vec![ast::Expr::Const(false)]),
                    1 => Ok(vec![ast::Expr::Const(true)]),
                    _ => Err(self.expected("'0' or '1'")),
                }
            }
            Token::GateIdentifier(_) => {
                let i = self.next();
                let i = i.as_gate_identifier().unwrap();
                let args = self.parse_expr()?;

                Ok(vec![ast::Expr::Call(i, args)])
            }

            Token::LocalIdentifier(_) => {
                let i = self.next();
                let i = i.as_local_identifier().unwrap();

                Ok(vec![ast::Expr::Ref(i, vec![0])])
            }

            Token::OBrack => {
                self.next();

                let mut items = Vec::new();

                if !matches!(self.peek(), Token::CBrack) {
                    items.extend(self.parse_expr()?);
                    while matches!(self.peek(), Token::Comma) {
                        self.next();
                        items.extend(self.parse_expr()?);
                    }
                }

                self.expect("']'", |tok| matches!(tok, Token::CBrack))?;

                Ok(items)
            }

            _ => Err(self.expected("expression")),
        }
    }
}

pub(crate) fn parse<'file>(tokens: impl Iterator<Item = Token<'file>>) -> Option<Vec<ast::Gate<'file>>> {
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
    fn gate() {
        /*
        `thingy arg:
            let res = `and [arg, arg]
            res
        */
        let tokens = vec![
            Token::GateIdentifier("thingy"),
            Token::LocalIdentifier("arg"),
            Token::Colon,
            Token::Let,
            Token::LocalIdentifier("res"),
            Token::Equals,
            Token::GateIdentifier("and"),
            Token::OBrack,
            Token::LocalIdentifier("arg"),
            Token::Comma,
            Token::LocalIdentifier("arg"),
            Token::CBrack,
            Token::LocalIdentifier("res"),
        ];

        assert_eq!(
            parse(make_token_stream(tokens)),
            Some(vec![ast::Gate {
                name: "thingy",
                arguments: vec![ast::Pattern("arg", 1)],
                lets: vec![ast::Let { pat: vec![ast::Pattern("res", 1)], val: vec![ast::Expr::Call("and", vec![ast::Expr::Ref("arg", vec![0]), ast::Expr::Ref("arg", vec![0])])] }],
                ret: vec![ast::Expr::Ref("res", vec![0])]
            }])
        )
    }

    #[test]
    fn r#let() {
        let tokens = vec![Token::Let, Token::LocalIdentifier("a"), Token::Equals, Token::LocalIdentifier("b")];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.parse_let(), Ok(ast::Let { pat: vec![ast::Pattern("a", 1)], val: vec![ast::Expr::Ref("b", vec![0])] }))
    }

    #[test]
    fn iden_pattern() {
        let tokens = vec![Token::LocalIdentifier("iden")];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.parse_pattern(), Ok(vec![ast::Pattern("iden", 1)]))
    }

    #[test]
    fn const_false_expr() {
        let tokens = vec![Token::Number(0)];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.parse_expr(), Ok(vec![ast::Expr::Const(false)]))
    }

    #[test]
    fn const_true_expr() {
        let tokens = vec![Token::Number(1)];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.parse_expr(), Ok(vec![ast::Expr::Const(true)]))
    }

    #[test]
    fn call_expr() {
        let tokens = vec![Token::GateIdentifier("a"), Token::OBrack, Token::LocalIdentifier("b"), Token::CBrack];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.parse_expr(), Ok(vec![ast::Expr::Call("a", vec![ast::Expr::Ref("b", vec![0])])]))
    }

    #[test]
    fn iden_expr() {
        let tokens = vec![Token::LocalIdentifier("a")];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.parse_expr(), Ok(vec![ast::Expr::Ref("a", vec![0])]))
    }

    #[test]
    fn multiple_expr() {
        let tokens = vec![Token::OBrack, Token::LocalIdentifier("a"), Token::Comma, Token::LocalIdentifier("b"), Token::Comma, Token::Number(0), Token::Comma, Token::Number(1), Token::CBrack];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.parse_expr(), Ok(vec![ast::Expr::Ref("a", vec![0]), ast::Expr::Ref("b", vec![0]), ast::Expr::Const(false), ast::Expr::Const(true)]))
    }
}
