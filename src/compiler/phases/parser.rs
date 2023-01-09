use crate::compiler::{
    data::{
        circuit1, nominal_type,
        token::{Token, TokenMatcher},
    },
    error::{CompileError, Report},
};

use std::iter::Peekable;

#[derive(PartialEq, Debug)]
pub(crate) struct AST<'file> {
    pub(crate) circuits: Vec<circuit1::UntypedCircuit<'file>>,
    pub(crate) type_decls: Vec<nominal_type::PartiallyDefinedStruct<'file>>,
}

struct Parser<'file, T: Iterator<Item = Token<'file>>> {
    tokens: Peekable<T>,
}

#[derive(Debug, PartialEq)]
struct ParseError<'file> {
    // TODO: more descriptive messages
    expected: &'static str,
    got: Token<'file>,
}

impl<'file> From<ParseError<'file>> for CompileError<'file> {
    fn from(ParseError { expected, got }: ParseError<'file>) -> Self {
        CompileError::new(got.span(), format!("expected {expected}, got {got}"))
    }
}

mod decl;
mod expr;
mod pattern;
mod type_;

impl<'file, T: Iterator<Item = Token<'file>>> Parser<'file, T> {
    fn peek(&mut self) -> &Token<'file> {
        self.tokens.peek().unwrap()
    }
    fn next(&mut self) -> Token<'file> {
        self.tokens.next().unwrap()
    }

    fn expect<TokData>(&mut self, matcher: TokenMatcher<'file, TokData>) -> Result<TokData, ParseError<'file>> {
        if matcher.matches(self.peek()) {
            Ok(matcher.convert(self.next()))
        } else {
            Err(self.expected_and_next(matcher.name()))
        }
    }

    fn maybe_consume<TokData>(&mut self, matcher: TokenMatcher<'file, TokData>) -> Option<TokData> {
        if matcher.matches(self.peek()) {
            Some(matcher.convert(self.next()))
        } else {
            None
        }
    }

    fn expected_and_next(&mut self, thing: &'static str) -> ParseError<'file> {
        ParseError { expected: thing, got: self.next() }
    }

    /* (unused)
    fn list<StartData, DelimData, EndData, A>(
        &mut self,
        start: TokenMatcher<'file, StartData>,
        delim: TokenMatcher<'file, DelimData>,
        ending: TokenMatcher<'file, EndData>,
        thing: impl for<'p> FnMut(&'p mut Parser<'file, T>) -> Result<A, ParseError<'file>>,
    ) -> Result<(Vec<A>, EndData), ParseError<'file>> {
        self.expect(start)?;
        self.finish_list(delim, ending, thing)
    }
    */

    fn finish_list<DelimData, EndData, A>(
        &mut self,
        delim: TokenMatcher<'file, DelimData>,
        ending: TokenMatcher<'file, EndData>,
        mut thing: impl for<'p> FnMut(&'p mut Parser<'file, T>) -> Result<A, ParseError<'file>>,
    ) -> Result<(Vec<A>, EndData), ParseError<'file>> {
        let mut items = Vec::new();

        while !ending.matches(self.peek()) {
            items.push(thing(self)?);

            if delim.matches(self.peek()) {
                self.next(); // there is a delimiter, the list may or may not continue
            } else {
                break; // if there is no delimiter, the list cannot be continued
            }
        }

        let end_data = self.expect(ending)?;

        Ok((items, end_data))
    }
}

pub(crate) fn parse<'file>(tokens: impl Iterator<Item = Token<'file>>) -> AST<'file> {
    let mut parser = Parser { tokens: tokens.peekable() };
    let mut circuits = Vec::new();
    let mut type_decls = Vec::new();

    while !Token::eof_matcher().matches(parser.peek()) {
        match parser.peek() {
            Token::Struct(_) => match decl::struct_(&mut parser) {
                Ok(type_decl) => type_decls.push(type_decl),
                Err(e) => {
                    e.report();
                }
            },
            _ => match decl::circuit(&mut parser) {
                Ok(circuit) => circuits.push(circuit),
                Err(e) => {
                    e.report();
                }
            },
        }
    }

    AST { circuits, type_decls }
}

#[cfg(test)]
mod test {
    use crate::compiler::{
        data::{circuit1, token::Token},
        error::{File, Span},
        phases::parser::{expr, Parser},
    };

    use std::iter::Peekable;

    pub(super) fn make_token_stream<'file>(tokens: impl IntoIterator<Item = Token<'file>>, sp: Span<'file>) -> Peekable<impl Iterator<Item = Token<'file>>> {
        tokens.into_iter().chain(std::iter::repeat_with(move || Token::EOF(sp))).peekable()
    }

    // TODO: test list() if used
    #[test]
    fn list() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "a"), Token::Comma(sp), Token::Identifier(sp, "b"), Token::CBrack(sp)], sp);

        assert_eq!(
            Parser { tokens }.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), expr::expr),
            Ok((
                vec![
                    circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "a"), type_info: (), span: sp },
                    circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "b"), type_info: (), span: sp }
                ],
                sp
            ))
        );
    }

    #[test]
    fn list_trailing_delim() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "a"), Token::Comma(sp), Token::Identifier(sp, "b"), Token::Comma(sp), Token::CBrack(sp)], sp);

        assert_eq!(
            Parser { tokens }.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), expr::expr),
            Ok((
                vec![
                    circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "a"), type_info: (), span: sp },
                    circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "b"), type_info: (), span: sp }
                ],
                sp
            ))
        );
    }
}
