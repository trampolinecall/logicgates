use crate::compiler::{
    data::{circuit1, token::{Token}},
    error::Span,
    phases::parser::{ParseError, Parser},
};

pub(super) fn expr<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<circuit1::UntypedExpr<'file>, ParseError<'file>> {
    let mut left = primary(parser)?;

    while Token::dot_matcher().matches(parser.peek()) {
        parser.next();

        let field = match parser.peek() {
            Token::Number(n_sp, n_str, _) => Ok((*n_sp, *n_str)),

            Token::PlainIdentifier(i) => Ok((i.span, i.name)),

            _ => Err(parser.expected_and_next("field name (a number or identifier)")),
        }?;
        parser.next();

        let left_span = left.span;
        left = circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Get(Box::new(left), field), type_info: (), span: left_span + field.0 };
    }

    Ok(left)
}

fn primary<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<circuit1::UntypedExpr<'file>, ParseError<'file>> {
    match parser.peek() {
        Token::Number(_, _, _) => {
            let (n_sp, _, n) = Token::number_matcher().convert(parser.next());

            match n {
                0 => Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Const(n_sp, false), type_info: (), span: n_sp }),
                1 => Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Const(n_sp, true), type_info: (), span: n_sp }),
                _ => Err(parser.expected_and_next("'0' or '1'")),
            }
        }
        Token::CircuitIdentifier(_) => {
            let ci = Token::circuit_identifier_matcher().convert(parser.next());

            let inline = parser.maybe_consume(Token::inline_matcher()).is_some();

            let arg = expr(parser)?;
            let ci_span = ci.span;
            let arg_span = arg.span;

            Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Call(ci, inline, Box::new(arg)), type_info: (), span: ci_span + arg_span })
        }

        Token::PlainIdentifier(_) => {
            let i = Token::plain_identifier_matcher().convert(parser.next());

            Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(i), type_info: (), span: i.span })
        }

        &Token::OBrack(obrack) => {
            parser.next();
            product(parser, obrack)
        }

        _ => Err(parser.expected_and_next("expression"))?,
    }
}

fn product<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>, obrack: Span<'file>) -> Result<circuit1::UntypedExpr<'file>, ParseError<'file>> {
    match parser.peek() {
        Token::Semicolon(_) => {
            parser.next();

            let (exprs, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), |parser| {
                let iden = parser.expect(Token::plain_identifier_matcher())?;
                parser.expect(Token::equals_matcher())?;
                let ty = expr(parser)?;
                Ok((iden.name.to_string(), ty))
            })?;

            Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Product(exprs), type_info: (), span: obrack + cbrack })
        }

        _ => {
            let (exprs, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), expr)?;
            Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Product(exprs.into_iter().enumerate().map(|(i, e)| (i.to_string(), e)).collect()), type_info: (), span: obrack + cbrack })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::{
        data::{circuit1, token::Token},
        error::File,
        phases::parser::{expr::expr, test::make_token_stream, Parser},
    };

    #[test]
    fn const_exprs() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens_0 = make_token_stream([Token::Number(sp, "0", 0)], sp);
        assert_eq!(expr(&mut Parser { tokens: tokens_0 }), Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Const(sp, false), type_info: (), span: sp }));

        let tokens_1 = make_token_stream([Token::Number(sp, "1", 1)], sp);
        assert_eq!(expr(&mut Parser { tokens: tokens_1 }), Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Const(sp, true), type_info: (), span: sp }));
    }

    #[test]
    fn call_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Apostrophe(sp), Token::Identifier(sp, "a"), Token::Identifier(sp, "b")], sp);
        assert_eq!(
            expr(&mut Parser { tokens }),
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
        assert_eq!(expr(&mut Parser { tokens }), Ok(circuit1::UntypedExpr { kind: circuit1::UntypedExprKind::Ref(sp, "a"), type_info: (), span: sp }));
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
            expr(&mut Parser { tokens }),
            Ok(circuit1::UntypedExpr {
                kind: circuit1::UntypedExprKind::Product(vec![
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
}
