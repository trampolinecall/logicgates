pub(crate) mod ast;

use std::iter::Peekable;

use crate::compiler::lexer::Token;

struct Parser<'file, T: Iterator<Item = Token<'file>>> {
    tokens: Peekable<T>,
}

impl<'file, T: std::iter::Iterator<Item = Token<'file>>> Parser<'file, T> {
    fn matches(&mut self) {}

    fn parse(&mut self) -> Option<ast::AST> {
        todo!()
    }
}

pub(crate) fn parse<'file>(tokens: impl Iterator<Item = Token<'file>>) -> Option<ast::AST> {
    Parser { tokens: tokens.peekable() }.parse()
}
