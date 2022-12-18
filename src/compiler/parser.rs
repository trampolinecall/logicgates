pub(crate) mod ast;

use crate::compiler::error::CompileError;
use crate::compiler::error::Report;
use crate::compiler::lexer::Token;
use std::iter::Peekable;

struct Parser<'file, T: Iterator<Item = Token<'file>>> {
    tokens: Peekable<T>,
}

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
                    e.report()
                }
            }
        }

        Some(gates)
    }

    fn parse_gate(&mut self) -> Result<ast::Gate<'file>, ParseError> {
        let name = self.expect("gate name", |tok| matches!(tok, Token::Identifier(_)))?;
        let name = *name.as_identifier().unwrap();
        let arguments = self.parse_pattern()?;
        self.expect("':'", |tok| matches!(tok, Token::Colon))?;
        let mut lets = Vec::new();

        while matches!(self.peek(), Token::Let) {
            lets.push(self.parse_let()?);
        }

        let ret = self.parse_expr()?;

        Ok(ast::Gate { name, arguments, lets, ret })
    }

    fn parse_pattern(&mut self) -> Result<ast::Pattern<'file>, ParseError> {
        // for now, only identifier patterns
        if matches!(self.peek(), Token::Identifier(_)) {
            let iden = self.next();
            let iden = *iden.as_identifier().unwrap();
            Ok(ast::Pattern::Iden(iden, 1))
        } else {
            Err(self.expected("pattern"))
        }
    }

    fn parse_let(&mut self) -> Result<ast::Let<'file>, ParseError> {
        self.expect("'let'", |tok| matches!(tok, Token::Let))?;
        let pat = self.parse_pattern()?;
        self.expect("'='", |tok| matches!(tok, Token::Equals))?;

        let val = self.parse_expr()?;
        Ok(ast::Let { pat, val })
    }

    fn parse_expr(&mut self) -> Result<ast::Expr<'file>, ParseError> {
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
            Token::Identifier(_) => {
                let i = self.next();
                let i = i.as_identifier().unwrap();

                if matches!(self.peek(), Token::OBrack) {
                    let mut args = Vec::new();
                    args.push(self.parse_expr()?);
                    while !matches!(self.peek(), Token::Comma) {
                        self.next();
                        args.push(self.parse_expr()?);
                    }

                    self.expect("']'", |tok| matches!(tok, Token::CBrack))?;

                    Ok(ast::Expr::Call(i, args))
                } else {
                    Ok(ast::Expr::Ref(i, vec![0]))
                }
            }

            _ => Err(self.expected("expression")),
        }
    }
}

pub(crate) fn parse<'file>(tokens: impl Iterator<Item = Token<'file>>) -> Option<Vec<ast::Gate<'file>>> {
    Parser { tokens: tokens.peekable() }.parse()
}
