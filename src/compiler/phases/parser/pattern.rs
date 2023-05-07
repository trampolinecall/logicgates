use crate::compiler::{
    data::{ast, token::Token},
    error::Span,
    phases::parser::{pattern, type_, ParseError, Parser},
};

pub(super) fn pattern<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<ast::Pattern<'file, ast::Untyped>, ParseError<'file>> {
    match parser.peek() {
        Token::PlainIdentifier(_) => {
            let iden = Token::plain_identifier_matcher().convert(parser.next());
            parser.expect(Token::semicolon_matcher())?;

            let type_ = type_::type_(parser)?;
            let type_span = type_.span;

            Ok(ast::Pattern { kind: ast::PatternKind::Identifier(iden, type_), type_info: (), span: iden.span + type_span })
        }

        &Token::OBrack(obrack) => {
            parser.next();
            product(parser, obrack)
        }

        _ => Err(parser.expected_and_next("pattern")),
    }
}

fn product<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>, obrack: Span<'file>) -> Result<ast::Pattern<'file, ast::Untyped>, ParseError<'file>> {
    match parser.peek() {
        Token::Semicolon(_) => {
            parser.next();

            let (patterns, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), |parser| {
                let iden = parser.expect(Token::plain_identifier_matcher())?;
                parser.expect(Token::equals_matcher())?;
                let ty = pattern(parser)?;
                Ok((iden.name.to_string(), ty))
            })?;

            Ok(ast::Pattern { kind: ast::PatternKind::Product(patterns), type_info: (), span: obrack + cbrack })
        }

        _ => {
            let (patterns, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), pattern::pattern)?;
            Ok(ast::Pattern { kind: ast::PatternKind::Product(patterns.into_iter().enumerate().map(|(i, p)| (i.to_string(), p)).collect()), type_info: (), span: obrack + cbrack })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::{
        data::{
            ast,
            token::{self, Token},
            type_expr,
        },
        error::File,
        phases::parser::{pattern::pattern, test::make_token_stream, Parser},
    };

    #[test]
    fn iden_pattern() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream(
            [
                Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "iden" }),
                Token::Semicolon(sp),
                Token::TypeIdentifier(token::TypeIdentifier { span: sp, name: "bit", with_tag: "-bit".to_string() }),
            ],
            sp,
        );
        assert_eq!(
            pattern(&mut Parser { tokens }),
            Ok(ast::Pattern {
                kind: ast::PatternKind::Identifier(
                    token::PlainIdentifier { span: sp, name: "iden" },
                    type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(token::TypeIdentifier { span: sp, name: "bit", with_tag: "-bit".to_string() }), span: sp }
                ),
                type_info: (),
                span: sp
            })
        );
    }
}
