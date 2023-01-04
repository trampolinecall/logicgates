use crate::compiler::error::CompileError;
use crate::compiler::error::Report;
use crate::compiler::lexer::token::TokenMatcher;
use crate::compiler::lexer::Token;
use std::iter::Peekable;

use super::ir::circuit1;
use super::ir::type_decl;
use super::ir::type_expr;


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

impl<'file, T: Iterator<Item = Token<'file>>> Parser<'file, T> {
    fn parse(&mut self) -> (Vec<circuit1::UntypedCircuit<'file>>, Vec<type_decl::TypeDecl<'file>>) {
        let mut circuits = Vec::new();
        let mut type_decls = Vec::new();

        while !Token::eof_matcher().matches(self.peek()) {
            match self.peek() {
                Token::Named(_) => match self.named_type_decl() {
                    Ok(type_decl) => type_decls.push(type_decl),
                    Err(e) => {
                        e.report();
                    }
                },
                _ => match self.circuit() {
                    Ok(circuit) => circuits.push(circuit),
                    Err(e) => {
                        e.report();
                    }
                },
            }
        }

        (circuits, type_decls)
    }

    fn circuit(&mut self) -> Result<circuit1::UntypedCircuit<'file>, ParseError<'file>> {
        self.expect(/* TODO: "circuit name (starting with '`')", */ Token::apostrophe_matcher())?;
        let name = self.expect(/* "circuit name after '`'", */ Token::identifier_matcher())?;
        let mut expressions = id_arena::Arena::new();
        let arguments = self.pattern()?;
        let output_type = self.type_()?;
        let mut lets = Vec::new();
        while Token::let_matcher().matches(self.peek()) {
            lets.push(self.r#let(&mut expressions)?);
        }

        let ret = self.expr(&mut expressions)?;

        Ok(circuit1::UntypedCircuit { name, input: arguments, lets, expressions, output: ret, output_type: (), output_type_annotation: output_type })
    }

    fn named_type_decl(&mut self) -> Result<type_decl::TypeDecl<'file>, ParseError<'file>> {
        self.expect(Token::named_matcher())?;
        let name = self.expect(Token::identifier_matcher())?;
        let ty = self.type_()?;

        Ok(type_decl::TypeDecl { name, ty })
    }

    fn r#let(&mut self, expressions: &mut circuit1::UntypedExprArena<'file>) -> Result<circuit1::UntypedLet<'file>, ParseError<'file>> {
        self.expect(Token::let_matcher())?;
        let pat = self.pattern()?;

        self.expect(Token::equals_matcher())?;

        let val = self.expr(expressions)?;
        Ok(circuit1::UntypedLet { pat, val })
    }

    fn pattern(&mut self) -> Result<circuit1::UntypedPattern<'file>, ParseError<'file>> {
        match self.peek() {
            Token::Identifier(_, _) => {
                let iden = Token::identifier_matcher().convert(self.next());
                self.expect(Token::semicolon_matcher())?;

                let type_ = self.type_()?;

                Ok(circuit1::UntypedPattern { kind: circuit1::PatternKind::Identifier(iden.0, iden.1, type_), type_info: () })
            }

            &Token::OBrack(obrack) => {
                self.next();

                let (patterns, cbrack) = self.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::pattern)?;
                Ok(circuit1::UntypedPattern { kind: circuit1::PatternKind::Product(obrack + cbrack, patterns), type_info: () })
            }
            _ => Err(self.expected_and_next("pattern")),
        }
    }

    fn expr(&mut self, expressions: &mut circuit1::UntypedExprArena<'file>) -> Result<circuit1::UntypedExprId<'file>, ParseError<'file>> {
        let mut left = self.primary_expr(expressions)?;

        while Token::dot_matcher().matches(self.peek()) {
            self.next();

            let field = match self.peek() {
                Token::Number(n_sp, n_str, _) => Ok((*n_sp, *n_str)),

                Token::Identifier(i_sp, i) => Ok((*i_sp, *i)),

                _ => Err(self.expected_and_next("field name (a number or identifier)")),
            }?;
            self.next();

            left = expressions.alloc(circuit1::Expr { kind: circuit1::ExprKind::Get(left, field), type_info: ()});
        }

        Ok(left)
    }

    fn primary_expr(&mut self, expressions: &mut circuit1::UntypedExprArena<'file>) -> Result<circuit1::UntypedExprId<'file>, ParseError<'file>> {
        match self.peek() {
            Token::Number(_, _, _) => {
                let (n_sp, _, n) = Token::number_matcher().convert(self.next());

                match n {
                    0 => Ok(expressions.alloc(circuit1::Expr { kind: circuit1::ExprKind::Const(n_sp, false), type_info: ()})),
                    1 => Ok(expressions.alloc(circuit1::Expr { kind: circuit1::ExprKind::Const(n_sp, true), type_info: ()})),
                    _ => Err(self.expected_and_next("'0' or '1'")),
                }
            }
            Token::Apostrophe(_) => {
                let _ = self.next();
                let i = self.expect(/* "circuit name after '`'", */ Token::identifier_matcher())?;

                let inline = self.maybe_consume(Token::inline_matcher()).is_some();

                let arg = self.expr(expressions)?;

                Ok(expressions.alloc(circuit1::Expr { kind: circuit1::ExprKind::Call(i, inline, arg), type_info: ()}))
            }

            Token::Identifier(_, _) => {
                let i = Token::identifier_matcher().convert(self.next());

                Ok(expressions.alloc(circuit1::Expr { kind: circuit1::ExprKind::Ref(i.0, i.1), type_info: ()}))
            }

            &Token::OBrack(obrack) => {
                self.next();

                let mut items = Vec::new();

                if !Token::cbrack_matcher().matches(self.peek()) {
                    items.push(self.expr(expressions)?);
                    while Token::comma_matcher().matches(self.peek()) {
                        self.next();
                        items.push(self.expr(expressions)?);
                    }
                }

                let cbrack = self.expect(Token::cbrack_matcher())?;

                Ok(expressions.alloc(circuit1::Expr { kind: circuit1::ExprKind::Multiple { obrack, cbrack, exprs: items }, type_info: ()}))
            }

            _ => Err(self.expected_and_next("expression"))?,
        }
    }

    fn type_(&mut self) -> Result<type_expr::TypeExpr<'file>, ParseError<'file>> {
        match *self.peek() {
            Token::Apostrophe(sp) => {
                let _ = self.next();
                Ok(type_expr::TypeExpr::Bit(sp))
            }

            Token::OBrack(obrack) => {
                let _ = self.next();

                match self.peek() {
                    &Token::Named(named) => {
                        let _ = self.next();

                        let (types, cbrack) = self.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), |parser| {
                            let name = parser.expect(Token::identifier_matcher())?;
                            let _ = parser.expect(Token::semicolon_matcher())?;
                            let ty = parser.type_()?;
                            Ok((name, ty))
                        })?;

                        Ok(type_expr::TypeExpr::NamedProduct { types, obrack, cbrack, named })
                    }

                    Token::Number(_, _, _) => {
                        let (len_sp, _, len) = Token::number_matcher().convert(self.next());

                        let cbrack = self.expect(Token::cbrack_matcher())?;

                        let ty = self.type_()?;

                        Ok(type_expr::TypeExpr::RepProduct { obrack, num: (len_sp, len), cbrack, type_: Box::new(ty) })
                    }

                    _ => {
                        let (types, cbrack) = self.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::type_)?;
                        Ok(type_expr::TypeExpr::Product { types, obrack, cbrack })
                    }
                }
            }

            Token::Identifier(_, _) => {
                let iden = Token::identifier_matcher().convert(self.next());

                Ok(type_expr::TypeExpr::Named(iden.0, iden.1))
            }

            _ => Err(self.expected_and_next("type")),
        }
    }
}

pub(crate) fn parse<'file>(tokens: impl Iterator<Item = Token<'file>>) -> (Vec<circuit1::UntypedCircuit<'file>>, Vec<type_decl::TypeDecl<'file>>) {
    Parser { tokens: tokens.peekable() }.parse()
}

#[cfg(test)]
mod test {
    use super::ast;
    use super::parse;
    use super::Parser;
    use crate::compiler::error::File;
    use crate::compiler::error::Span;
    use crate::compiler::ir;
    use crate::compiler::lexer::Token;

    use std::iter::Peekable;

    fn make_token_stream<'file>(tokens: impl IntoIterator<Item = Token<'file>>, sp: Span<'file>) -> Peekable<impl Iterator<Item = Token<'file>>> {
        tokens.into_iter().chain(std::iter::repeat_with(move || Token::EOF(sp))).peekable()
    }

    // TODO: test list() if used
    #[test]
    fn list() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "a"), Token::Comma(sp), Token::Identifier(sp, "b"), Token::CBrack(sp)], sp);

        assert_eq!(Parser { tokens }.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::expr), Ok((vec![ExprKind::Ref(sp, "a"), ExprKind::Ref(sp, "b")], sp)));
    }

    #[test]
    fn list_trailing_delim() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "a"), Token::Comma(sp), Token::Identifier(sp, "b"), Token::Comma(sp), Token::CBrack(sp)], sp);

        assert_eq!(Parser { tokens }.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::expr), Ok((vec![ExprKind::Ref(sp, "a"), ExprKind::Ref(sp, "b")], sp)));
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
                Token::Apostrophe(sp),
                Token::Identifier(sp, "thingy"),
                Token::Identifier(sp, "arg"),
                Token::Semicolon(sp),
                Token::Apostrophe(sp),
                Token::Use(sp),
                Token::Identifier(sp, "res"),
                Token::Semicolon(sp),
                Token::Apostrophe(sp),
                Token::Equals(sp),
                Token::Apostrophe(sp),
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
            (vec![circuit1::UntypedCircuit {
                name: (sp, "thingy"),
                input: circuit1::UntypedPattern { kind: PatternKind::Identifier(sp, "arg", type_expr::TypeExpr::Bit(sp)), type_info: () },
                lets: vec![LetAST {
                    pat: circuit1::UntypedPattern { kind: PatternKind::Identifier(sp, "res", type_expr::TypeExpr::Bit(sp)), type_info: () },
                    val: ExprKind::Call((sp, "and"), false, Box::new(ExprKind::Multiple { obrack: sp, cbrack: sp, exprs: vec![ExprKind::Ref(sp, "arg"), ExprKind::Ref(sp, "arg")] }))
                }],
                output: ExprKind::Ref(sp, "res")
            }])
        );
    }

    #[test]
    fn r#let() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Use(sp), Token::Identifier(sp, "a"), Token::Semicolon(sp), Token::Apostrophe(sp), Token::Equals(sp), Token::Identifier(sp, "b")], sp);
        assert_eq!(
            Parser { tokens: tokens }.r#let(),
            Ok(LetAST { pat: circuit1::UntypedPattern { kind: PatternKind::Identifier(sp, "a", type_expr::TypeExpr::Bit(sp)), type_info: () }, val: ExprKind::Ref(sp, "b") })
        );
    }

    #[test]
    fn iden_pattern() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "iden"), Token::Semicolon(sp), Token::Apostrophe(sp)], sp);
        assert_eq!(Parser { tokens }.pattern(), Ok(circuit1::UntypedPattern { kind: PatternKind::Identifier(sp, "iden", type_expr::TypeExpr::Bit(sp)), type_info: () }));
    }

    #[test]
    fn const_exprs() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens_0 = make_token_stream([Token::Number(sp, "0", 0)], sp);
        assert_eq!(Parser { tokens: tokens_0 }.expr(), Ok(ExprKind::Const(sp, false)));

        let tokens_1 = make_token_stream([Token::Number(sp, "1", 1)], sp);
        assert_eq!(Parser { tokens: tokens_1 }.expr(), Ok(ExprKind::Const(sp, true)));
    }

    #[test]
    fn call_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Apostrophe(sp), Token::Identifier(sp, "a"), Token::Identifier(sp, "b")], sp);
        assert_eq!(Parser { tokens: tokens }.expr(), Ok(ExprKind::Call((sp, "a"), false, Box::new(ExprKind::Ref(sp, "b")))));
    }

    #[test]
    fn iden_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "a")], sp);
        assert_eq!(Parser { tokens }.expr(), Ok(ExprKind::Ref(sp, "a")));
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
        assert_eq!(
            Parser { tokens }.expr(),
            Ok(ExprKind::Multiple { obrack: sp, cbrack: sp, exprs: vec![ExprKind::Ref(sp, "a"), ExprKind::Ref(sp, "b"), ExprKind::Const(sp, false), ExprKind::Const(sp, true)] })
        );
    }

    // TODO: test array types, types in general
}
