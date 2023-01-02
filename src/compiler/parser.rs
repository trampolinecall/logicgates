pub(crate) mod ast;

use crate::compiler::error::CompileError;
use crate::compiler::error::Report;
use crate::compiler::lexer::token::TokenMatcher;
use crate::compiler::lexer::Token;
use std::iter::Peekable;

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

impl<'file, T: Iterator<Item = Token<'file>>> Parser<'file, T> {
    fn parse(&mut self) -> Option<Vec<ast::Circuit<'file>>> {
        let mut circuits = Vec::new();
        while !Token::eof_matcher().matches(self.peek()) {
            match self.circuit() {
                Ok(circuit) => circuits.push(circuit),
                Err(e) => {
                    e.report();
                }
            }
        }

        Some(circuits)
    }

    fn circuit(&mut self) -> Result<ast::Circuit<'file>, ParseError<'file>> {
        self.expect(/* TODO: "circuit name (starting with '`')", */ Token::backtick_matcher())?;
        let name = self.expect(/* "circuit name after '`'", */ Token::identifier_matcher())?;
        let arguments = self.pattern()?;
        let mut lets = Vec::new();

        while Token::let_matcher().matches(self.peek()) {
            lets.push(self.r#let()?);
        }

        let ret = self.expr()?;

        Ok(ast::Circuit { name, input: arguments, lets, output: ret })
    }

    fn r#let(&mut self) -> Result<ast::Let<'file>, ParseError<'file>> {
        self.expect(Token::let_matcher())?;
        let pat = self.pattern()?;

        self.expect(Token::equals_matcher())?;

        let val = self.expr()?;
        Ok(ast::Let { pat, val })
    }

    fn pattern(&mut self) -> Result<ast::Pattern<'file>, ParseError<'file>> {
        match self.peek() {
            Token::Identifier(_, _) => {
                let iden = Token::identifier_matcher().convert(self.next());

                self.expect(Token::semicolon_matcher())?;

                let type_ = self.type_()?;

                Ok(ast::Pattern::Identifier(iden, type_))
            }

            &Token::OBrack(obrack) => {
                self.next();

                let (patterns, cbrack) = self.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::pattern)?;
                Ok(ast::Pattern::Product(obrack + cbrack, patterns))
            }
            _ => Err(self.expected_and_next("pattern")),
        }
    }

    fn expr(&mut self) -> Result<ast::Expr<'file>, ParseError<'file>> {
        let mut left = self.primary_expr()?;

        while Token::dot_matcher().matches(self.peek()) {
            self.next();

            let field = match self.peek() {
                Token::Number(n_sp, n_str, _) => Ok((*n_sp, *n_str)),

                Token::Identifier(i_sp, i) => Ok((*i_sp, *i)),

                _ => Err(self.expected_and_next("field name (a number or identifier)")),
            }?;
            self.next();

            left = ast::Expr::Get(Box::new(left), field);
        }

        Ok(left)
    }

    fn primary_expr(&mut self) -> Result<ast::Expr<'file>, ParseError<'file>> {
        match self.peek() {
            &Token::Number(_, _, _) => {
                let (n_sp, _, n) = Token::number_matcher().convert(self.next());

                match n {
                    0 => Ok(ast::Expr::Const(n_sp, false)),
                    1 => Ok(ast::Expr::Const(n_sp, true)),
                    _ => Err(self.expected_and_next("'0' or '1'")),
                }
            }
            Token::Backtick(_) => {
                let _ = self.next();
                let i = self.expect(/* "circuit name after '`'", */ Token::identifier_matcher())?;

                let inline = self.maybe_consume(Token::inline_matcher()).is_some();

                let arg = self.expr()?;

                Ok(ast::Expr::Call(i, inline, Box::new(arg)))
            }

            Token::Identifier(_, _) => {
                let i = Token::identifier_matcher().convert(self.next());

                Ok(ast::Expr::Ref(i.0, i.1))
            }

            Token::OBrack(obrack_sp) => {
                let obrack_sp = *obrack_sp;
                self.next();

                let mut items = Vec::new();

                if !Token::cbrack_matcher().matches(self.peek()) {
                    items.push(self.expr()?);
                    while Token::comma_matcher().matches(self.peek()) {
                        self.next();
                        items.push(self.expr()?);
                    }
                }

                let cbrack_sp = self.expect(Token::cbrack_matcher())?;

                Ok(ast::Expr::Multiple(obrack_sp + cbrack_sp, items))
            }

            _ => Err(self.expected_and_next("expression"))?,
        }
    }

    fn type_(&mut self) -> Result<ast::Type<'file>, ParseError<'file>> {
        match *self.peek() {
            Token::Backtick(sp) => {
                let _ = self.next();
                Ok(ast::Type::Bit(sp))
            }

            Token::OBrack(obrack) => {
                let _ = self.next();

                match self.peek() {
                    Token::Number(_, _, _) => {
                        let (len_sp, _, len) = Token::number_matcher().convert(self.next());

                        let cbrack = self.expect(Token::cbrack_matcher())?;

                        let ty = self.type_()?;

                        Ok(ast::Type::RepProduct { obrack, num: (len_sp, len), cbrack, type_: Box::new(ty) })
                    }

                    _ => {
                        let (types, cbrack) = self.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::type_)?;
                        Ok(ast::Type::Product { types, obrack, cbrack })
                    }
                }
            }

            _ => Err(self.expected_and_next("type")),
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
    use crate::compiler::error::File;
    use crate::compiler::error::Span;
    use crate::compiler::lexer::Token;

    use std::iter::Peekable;

    fn make_token_stream<'file>(tokens: impl IntoIterator<Item = Token<'file>>, sp: Span<'file>) -> Peekable<impl Iterator<Item = Token<'file>>> {
        tokens.into_iter().chain(std::iter::repeat_with(move || Token::EOF(sp))).peekable()
    }

    #[test]
    fn list() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::OBrack(sp), Token::Identifier(sp, "a"), Token::Comma(sp), Token::Identifier(sp, "b"), Token::CBrack(sp)], sp);

        assert_eq!(Parser { tokens }.list(Token::obrack_matcher(), Token::comma_matcher(), Token::cbrack_matcher(), Parser::expr), Ok((vec![ast::Expr::Ref(sp, "a"), ast::Expr::Ref(sp, "b")], sp)));
    }

    #[test]
    fn list_trailing_delim() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::OBrack(sp), Token::Identifier(sp, "a"), Token::Comma(sp), Token::Identifier(sp, "b"), Token::Comma(sp), Token::CBrack(sp)], sp);

        assert_eq!(Parser { tokens }.list(Token::obrack_matcher(), Token::comma_matcher(), Token::cbrack_matcher(), Parser::expr), Ok((vec![ast::Expr::Ref(sp, "a"), ast::Expr::Ref(sp, "b")], sp)));
    }

    // TODO: test inline calls
    #[test]
    fn circuit() {
        let file = File::test_file();
        let sp = file.eof_span();

        /*
        `thingy arg; `
            let res; ` = `and [arg, arg]
            res
        */
        let tokens = make_token_stream(
            [
                Token::Backtick(sp),
                Token::Identifier(sp, "thingy"),
                Token::Identifier(sp, "arg"),
                Token::Semicolon(sp),
                Token::Backtick(sp),
                Token::Let(sp),
                Token::Identifier(sp, "res"),
                Token::Semicolon(sp),
                Token::Backtick(sp),
                Token::Equals(sp),
                Token::Backtick(sp),
                Token::Identifier(sp, "and"),
                Token::OBrack(sp),
                Token::Identifier(sp, "arg"),
                Token::Comma(sp),
                Token::Identifier(sp, "arg"),
                Token::CBrack(sp),
                Token::Identifier(sp, "res"),
            ],
            sp,
        );

        assert_eq!(
            parse(tokens),
            Some(vec![ast::Circuit {
                name: (sp, "thingy"),
                input: ast::Pattern::Identifier((sp, "arg"), ast::Type::Bit(sp)),
                lets: vec![ast::Let {
                    pat: ast::Pattern::Identifier((sp, "res"), ast::Type::Bit(sp)),
                    val: ast::Expr::Call((sp, "and"), false, Box::new(ast::Expr::Multiple(sp, vec![ast::Expr::Ref(sp, "arg"), ast::Expr::Ref(sp, "arg")])))
                }],
                output: ast::Expr::Ref(sp, "res")
            }])
        );
    }

    #[test]
    fn r#let() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Let(sp), Token::Identifier(sp, "a"), Token::Semicolon(sp), Token::Backtick(sp), Token::Equals(sp), Token::Identifier(sp, "b")], sp);
        assert_eq!(Parser { tokens: tokens }.r#let(), Ok(ast::Let { pat: ast::Pattern::Identifier((sp, "a"), ast::Type::Bit(sp)), val: ast::Expr::Ref(sp, "b") }));
    }

    #[test]
    fn iden_pattern() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "iden"), Token::Semicolon(sp), Token::Backtick(sp)], sp);
        assert_eq!(Parser { tokens }.pattern(), Ok(ast::Pattern::Identifier((sp, "iden"), ast::Type::Bit(sp))));
    }

    #[test]
    fn const_exprs() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens_0 = make_token_stream([Token::Number(sp, "0", 0)], sp);
        assert_eq!(Parser { tokens: tokens_0 }.expr(), Ok(ast::Expr::Const(sp, false)));

        let tokens_1 = make_token_stream([Token::Number(sp, "1", 1)], sp);
        assert_eq!(Parser { tokens: tokens_1 }.expr(), Ok(ast::Expr::Const(sp, true)));
    }

    #[test]
    fn call_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Backtick(sp), Token::Identifier(sp, "a"), Token::Identifier(sp, "b")], sp);
        assert_eq!(Parser { tokens: tokens }.expr(), Ok(ast::Expr::Call((sp, "a"), false, Box::new(ast::Expr::Ref(sp, "b")))));
    }

    #[test]
    fn iden_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "a")], sp);
        assert_eq!(Parser { tokens }.expr(), Ok(ast::Expr::Ref(sp, "a")));
    }

    #[test]
    fn multiple_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream(
            [
                Token::OBrack(sp),
                Token::Identifier(sp, "a"),
                Token::Comma(sp),
                Token::Identifier(sp, "b"),
                Token::Comma(sp),
                Token::Number(sp, "0", 0),
                Token::Comma(sp),
                Token::Number(sp, "0", 1),
                Token::CBrack(sp),
            ],
            sp,
        );
        assert_eq!(Parser { tokens }.expr(), Ok(ast::Expr::Multiple(sp, vec![ast::Expr::Ref(sp, "a"), ast::Expr::Ref(sp, "b"), ast::Expr::Const(sp, false), ast::Expr::Const(sp, true)])));
    }

    // TODO: test array types, types in general
}
