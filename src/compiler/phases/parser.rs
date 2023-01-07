use crate::compiler::{
    data::{
        circuit1, nominal_type,
        token::{Token, TokenMatcher},
        type_expr,
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
    fn parse(&mut self) -> AST<'file> {
        let mut circuits = Vec::new();
        let mut type_decls = Vec::new();

        while !Token::eof_matcher().matches(self.peek()) {
            match self.peek() {
                Token::Struct(_) => match self.struct_decl() {
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

        AST { circuits, type_decls }
    }

    fn circuit(&mut self) -> Result<circuit1::UntypedCircuit<'file>, ParseError<'file>> {
        self.expect(/* TODO: "circuit name (starting with '`')", */ Token::apostrophe_matcher())?;
        let name = self.expect(/* "circuit name after '`'", */ Token::identifier_matcher())?;
        let arguments = self.pattern()?;
        let output_type = self.type_()?;
        let mut lets = Vec::new();
        while Token::let_matcher().matches(self.peek()) {
            lets.push(self.r#let()?);
        }

        let ret = self.expr()?;

        Ok(circuit1::UntypedCircuit { name, input: arguments, lets, output: ret, output_type })
    }

    fn struct_decl(&mut self) -> Result<nominal_type::PartiallyDefinedStruct<'file>, ParseError<'file>> {
        self.expect(Token::struct_matcher())?;
        let name = self.expect(Token::identifier_matcher())?;
        self.expect(Token::obrack_matcher())?;

        let mut fields = Vec::new();
        while Token::identifier_matcher().matches(self.peek()) {
            let field_name = Token::identifier_matcher().convert(self.next());
            let field_ty = self.type_()?;
            fields.push((field_name, field_ty)) // TODO: anonymous fields too
        }

        self.expect(Token::cbrack_matcher())?;

        Ok(nominal_type::Struct { name, fields })
    }

    fn r#let(&mut self) -> Result<circuit1::UntypedLet<'file>, ParseError<'file>> {
        self.expect(Token::let_matcher())?;
        let pat = self.pattern()?;

        self.expect(Token::equals_matcher())?;

        let val = self.expr()?;
        Ok(circuit1::UntypedLet { pat, val })
    }

    fn pattern(&mut self) -> Result<circuit1::UntypedPattern<'file>, ParseError<'file>> {
        match self.peek() {
            Token::Identifier(_, _) => {
                let iden = Token::identifier_matcher().convert(self.next());
                self.expect(Token::semicolon_matcher())?;

                let type_ = self.type_()?;
                let type_span = type_.span();

                Ok(circuit1::UntypedPattern { kind: circuit1::UntypedPatternKind::Identifier(iden.0, iden.1, type_), type_info: (), span: iden.0 + type_span })
            }

            &Token::OBrack(obrack) => {
                self.next();

                let (patterns, cbrack) = self.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::pattern)?;
                Ok(circuit1::UntypedPattern { kind: circuit1::UntypedPatternKind::Product(obrack + cbrack, patterns), type_info: (), span: obrack + cbrack })
            }
            _ => Err(self.expected_and_next("pattern")),
        }
    }

    fn expr(&mut self) -> Result<circuit1::UntypedExpr<'file>, ParseError<'file>> {
        let mut left = self.primary_expr()?;

        while Token::dot_matcher().matches(self.peek()) {
            self.next();

            let field = match self.peek() {
                Token::Number(n_sp, n_str, _) => Ok((*n_sp, *n_str)),

                Token::Identifier(i_sp, i) => Ok((*i_sp, *i)),

                _ => Err(self.expected_and_next("field name (a number or identifier)")),
            }?;
            self.next();

            let left_span = left.span;
            left = circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Get(Box::new(left), field), type_info: (), span: left_span + field.0 };
        }

        Ok(left)
    }

    fn primary_expr(&mut self) -> Result<circuit1::UntypedExpr<'file>, ParseError<'file>> {
        match self.peek() {
            Token::Number(_, _, _) => {
                let (n_sp, _, n) = Token::number_matcher().convert(self.next());

                match n {
                    0 => Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Const(n_sp, false), type_info: (), span: n_sp }),
                    1 => Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Const(n_sp, true), type_info: (), span: n_sp }),
                    _ => Err(self.expected_and_next("'0' or '1'")),
                }
            }
            &Token::Apostrophe(apos) => {
                self.next();
                let i = self.expect(/* "circuit name after '`'", */ Token::identifier_matcher())?;

                let inline = self.maybe_consume(Token::inline_matcher()).is_some();

                let arg = self.expr()?;
                let arg_span = arg.span;

                Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Call(i, inline, Box::new(arg)), type_info: (), span: apos + arg_span })
            }

            Token::Identifier(_, _) => {
                let i = Token::identifier_matcher().convert(self.next());

                Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(i.0, i.1), type_info: (), span: i.0 })
            }

            &Token::OBrack(obrack) => {
                self.next();

                let mut items = Vec::new();

                if !Token::cbrack_matcher().matches(self.peek()) {
                    items.push(self.expr()?);
                    while Token::comma_matcher().matches(self.peek()) {
                        self.next();
                        items.push(self.expr()?);
                    }
                }

                let cbrack = self.expect(Token::cbrack_matcher())?;

                Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Multiple(items), type_info: (), span: obrack + cbrack })
            }

            _ => Err(self.expected_and_next("expression"))?,
        }
    }

    fn type_(&mut self) -> Result<type_expr::TypeExpr<'file>, ParseError<'file>> {
        match *self.peek() {
            Token::OBrack(obrack) => {
                self.next();

                match self.peek() {
                    &Token::Named(named) => {
                        self.next();

                        let (types, cbrack) = self.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), |parser| {
                            let name = parser.expect(Token::identifier_matcher())?;
                            parser.expect(Token::semicolon_matcher())?;
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

                Ok(type_expr::TypeExpr::Nominal(iden.0, iden.1))
            }

            _ => Err(self.expected_and_next("type")),
        }
    }
}

pub(crate) fn parse<'file>(tokens: impl Iterator<Item = Token<'file>>) -> AST<'file> {
    Parser { tokens: tokens.peekable() }.parse()
}

#[cfg(test)]
mod test {
    use crate::compiler::{
        data::{circuit1, token::Token, type_expr},
        error::{File, Span},
        phases::parser::{parse, Parser, AST},
    };

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

        assert_eq!(
            Parser { tokens }.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::expr),
            Ok((vec![circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "a"), type_info: (), span: sp }, circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "b"), type_info: (), span: sp }], sp))
        );
    }

    #[test]
    fn list_trailing_delim() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "a"), Token::Comma(sp), Token::Identifier(sp, "b"), Token::Comma(sp), Token::CBrack(sp)], sp);

        assert_eq!(
            Parser { tokens }.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::expr),
            Ok((vec![circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "a"), type_info: (), span: sp }, circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "b"), type_info: (), span: sp }], sp))
        );
    }

    // TODO: test inline calls
    #[test]
    fn circuit() {
        let file = File::test_file();
        let sp = file.eof_span();

        /*
        'thingy arg; bit bit
            let res; bit = 'and [arg, arg]
            res
        */
        let tokens = make_token_stream(
            [
                Token::Apostrophe(sp),
                Token::Identifier(sp, "thingy"),
                Token::Identifier(sp, "arg"),
                Token::Semicolon(sp),
                Token::Identifier(sp, "bit"),
                Token::Identifier(sp, "bit"),
                Token::Let(sp),
                Token::Identifier(sp, "res"),
                Token::Semicolon(sp),
                Token::Identifier(sp, "bit"),
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
            AST {
                circuits: vec![circuit1::UntypedCircuit {
                    name: (sp, "thingy"),
                    input: circuit1::UntypedPattern { kind: circuit1::UntypedPatternKind::Identifier(sp, "arg", type_expr::TypeExpr::Nominal(sp, "bit")), type_info: (), span: sp },
                    lets: vec![circuit1::UntypedLet {
                        pat: circuit1::UntypedPattern { kind: circuit1::UntypedPatternKind::Identifier(sp, "res", type_expr::TypeExpr::Nominal(sp, "bit")), type_info: (), span: sp },
                        val: circuit1::UntypedExpr {
                            kind: circuit1::UntypedExprKind::Call(
                                (sp, "and"),
                                false,
                                Box::new(circuit1::UntypedExpr {
                                    kind: circuit1::UntypedExprKind::Multiple(vec![
                                        circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "arg"), type_info: (), span: sp },
                                        circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "arg"), type_info: (), span: sp }
                                    ]),
                                    type_info: (),
                                    span: sp
                                })
                            ),
                            type_info: (),
                            span: sp
                        }
                    }],
                    output: circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "res"), type_info: (), span: sp },
                    output_type: type_expr::TypeExpr::Nominal(sp, "bit")
                }],
                type_decls: vec![]
            }
        );
    }

    #[test]
    fn r#let() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Let(sp), Token::Identifier(sp, "a"), Token::Semicolon(sp), Token::Identifier(sp, "bit"), Token::Equals(sp), Token::Identifier(sp, "b")], sp);
        assert_eq!(
            Parser { tokens: tokens }.r#let(),
            Ok(circuit1::UntypedLet {
                pat: circuit1::UntypedPattern { kind: circuit1::UntypedPatternKind::Identifier(sp, "a", type_expr::TypeExpr::Nominal(sp, "bit")), type_info: (), span: sp },
                val: circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "b"), type_info: (), span: sp }
            })
        );
    }

    #[test]
    fn iden_pattern() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "iden"), Token::Semicolon(sp), Token::Identifier(sp, "bit")], sp);
        assert_eq!(Parser { tokens }.pattern(), Ok(circuit1::UntypedPattern { kind: circuit1::UntypedPatternKind::Identifier(sp, "iden", type_expr::TypeExpr::Nominal(sp, "bit")), type_info: (), span: sp }));
    }

    #[test]
    fn const_exprs() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens_0 = make_token_stream([Token::Number(sp, "0", 0)], sp);
        assert_eq!(Parser { tokens: tokens_0 }.expr(), Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Const(sp, false), type_info: (), span: sp }));

        let tokens_1 = make_token_stream([Token::Number(sp, "1", 1)], sp);
        assert_eq!(Parser { tokens: tokens_1 }.expr(), Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Const(sp, true), type_info: (), span: sp }));
    }

    #[test]
    fn call_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Apostrophe(sp), Token::Identifier(sp, "a"), Token::Identifier(sp, "b")], sp);
        assert_eq!(
            Parser { tokens: tokens }.expr(),
            Ok(circuit1::UntypedExpr {
                kind: circuit1::UntypedExprKind::Call((sp, "a"), false, Box::new(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "b"), type_info: (), span: sp })),
                type_info: (),
                span: sp
            })
        );
    }

    #[test]
    fn iden_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "a")], sp);
        assert_eq!(Parser { tokens }.expr(), Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "a"), type_info: (), span: sp }));
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
            Ok(circuit1::UntypedExpr {
                kind: circuit1::UntypedExprKind::Multiple(vec![
                    circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "a"), type_info: (), span: sp },
                    circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "b"), type_info: (), span: sp },
                    circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Const(sp, false), type_info: (), span: sp },
                    circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Const(sp, true), type_info: (), span: sp }
                ]),
                type_info: (),
                span: sp
            })
        );
    }

    // TODO: test array types, types in general
}
