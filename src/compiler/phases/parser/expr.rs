use crate::compiler::{
    data::{ast, token::Token},
    error::Span,
    phases::parser::{ParseError, Parser},
};

pub(super) fn expr<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<ast::Expr<'file, ast::Untyped>, ParseError<'file>> {
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
        left = ast::Expr { kind: ast::ExprKind::Get(Box::new(left), field), type_info: (), span: left_span + field.0 };
    }

    Ok(left)
}

fn primary<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<ast::Expr<'file, ast::Untyped>, ParseError<'file>> {
    match parser.peek() {
        Token::Number(_, _, _) => {
            let (n_sp, _, n) = Token::number_matcher().convert(parser.next());

            match n {
                0 => Ok(ast::Expr { kind: ast::ExprKind::Const(n_sp, false), type_info: (), span: n_sp }),
                1 => Ok(ast::Expr { kind: ast::ExprKind::Const(n_sp, true), type_info: (), span: n_sp }),
                _ => Err(parser.expected_and_next("'0' or '1'")),
            }
        }

        Token::PlainIdentifier(_) => {
            let i = Token::plain_identifier_matcher().convert(parser.next());

            Ok(ast::Expr { kind: ast::ExprKind::Ref(i), type_info: (), span: i.span })
        }

        &Token::OBrack(obrack) => {
            parser.next();
            product(parser, obrack)
        }

        _ => Err(parser.expected_and_next("expression"))?,
    }
}

fn product<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>, obrack: Span<'file>) -> Result<ast::Expr<'file, ast::Untyped>, ParseError<'file>> {
    match parser.peek() {
        Token::Semicolon(_) => {
            parser.next();

            let (exprs, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), |parser| {
                let iden = parser.expect(Token::plain_identifier_matcher())?;
                parser.expect(Token::equals_matcher())?;
                let ty = expr(parser)?;
                Ok((iden.name.to_string(), ty))
            })?;

            Ok(ast::Expr { kind: ast::ExprKind::Product(exprs), type_info: (), span: obrack + cbrack })
        }

        _ => {
            let (exprs, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), expr)?;
            Ok(ast::Expr { kind: ast::ExprKind::Product(exprs.into_iter().enumerate().map(|(i, e)| (i.to_string(), e)).collect()), type_info: (), span: obrack + cbrack })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::{
        data::{
            ast,
            token::{self, Token},
        },
        error::File,
        phases::parser::{expr::expr, test::make_token_stream, Parser},
    };

    #[test]
    fn const_exprs() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens_0 = make_token_stream([Token::Number(sp, "0", 0)], sp);
        assert_eq!(expr(&mut Parser { tokens: tokens_0 }), Ok(ast::Expr { kind: ast::ExprKind::Const(sp, false), type_info: (), span: sp }));

        let tokens_1 = make_token_stream([Token::Number(sp, "1", 1)], sp);
        assert_eq!(expr(&mut Parser { tokens: tokens_1 }), Ok(ast::Expr { kind: ast::ExprKind::Const(sp, true), type_info: (), span: sp }));
    }

    #[test]
    fn call_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream(
            [Token::CircuitIdentifier(token::CircuitIdentifier { span: sp, name: "a", with_tag: "\\a".to_string() }), Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "b" })],
            sp,
        );
        assert_eq!(
            expr(&mut Parser { tokens }),
            Ok(ast::Expr {
                kind: ast::ExprKind::Call(
                    token::CircuitIdentifier { span: sp, name: "a", with_tag: "\\a".to_string() },
                    false,
                    Box::new(ast::Expr { kind: ast::ExprKind::Ref(token::PlainIdentifier { span: sp, name: "b" }), type_info: (), span: sp })
                ),
                type_info: (),
                span: sp
            })
        );
    }

    #[test]
    fn iden_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "a" })], sp);
        assert_eq!(expr(&mut Parser { tokens }), Ok(ast::Expr { kind: ast::ExprKind::Ref(token::PlainIdentifier { span: sp, name: "a" }), type_info: (), span: sp }));
    }

    #[test]
    fn product_expr() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream(
            [
                Token::OBrack(sp),
                Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "a" }),
                Token::Comma(sp),
                Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "b" }),
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
            Ok(ast::Expr {
                kind: ast::ExprKind::Product(vec![
                    ("0".to_string(), ast::Expr { kind: ast::ExprKind::Ref(token::PlainIdentifier { span: sp, name: "a" }), type_info: (), span: sp }),
                    ("1".to_string(), ast::Expr { kind: ast::ExprKind::Ref(token::PlainIdentifier { span: sp, name: "b" }), type_info: (), span: sp }),
                    ("2".to_string(), ast::Expr { kind: ast::ExprKind::Const(sp, false), type_info: (), span: sp }),
                    ("3".to_string(), ast::Expr { kind: ast::ExprKind::Const(sp, true), type_info: (), span: sp }),
                ]),
                type_info: (),
                span: sp
            })
        );
    }
}
