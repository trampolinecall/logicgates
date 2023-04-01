use crate::compiler::{
    data::{ast, token::Token},
    error::Span,
    phases::parser::{pattern, type_, ParseError, Parser},
};

pub(super) fn pattern<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<ast::UntypedPattern<'file>, ParseError<'file>> {
    match parser.peek() {
        Token::PlainIdentifier(_) => {
            let iden = Token::plain_identifier_matcher().convert(parser.next());
            parser.expect(Token::semicolon_matcher())?;

            let type_ = type_::type_(parser)?;
            let type_span = type_.span;

            Ok(ast::UntypedPattern { kind: ast::UntypedPatternKind::Identifier(iden, type_), type_info: (), span: iden.span + type_span })
        }

        &Token::OBrack(obrack) => {
            parser.next();
            product(parser, obrack)
        }

        _ => Err(parser.expected_and_next("pattern")),
    }
}

fn product<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>, obrack: Span<'file>) -> Result<ast::UntypedPattern<'file>, ParseError<'file>> {
    match parser.peek() {
        Token::Semicolon(_) => {
            parser.next();

            let (patterns, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), |parser| {
                let iden = parser.expect(Token::plain_identifier_matcher())?;
                parser.expect(Token::equals_matcher())?;
                let ty = pattern(parser)?;
                Ok((iden.name.to_string(), ty))
            })?;

            Ok(ast::UntypedPattern { kind: ast::UntypedPatternKind::Product(patterns), type_info: (), span: obrack + cbrack })
        }

        _ => {
            let (patterns, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), pattern::pattern)?;
            Ok(ast::UntypedPattern { kind: ast::UntypedPatternKind::Product(patterns.into_iter().enumerate().map(|(i, p)| (i.to_string(), p)).collect()), type_info: (), span: obrack + cbrack })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::{
        data::{ast, token::Token, type_expr},
        error::File,
        phases::parser::{pattern::pattern, test::make_token_stream, Parser},
    };

    #[test]
    fn iden_pattern() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Identifier(sp, "iden"), Token::Semicolon(sp), Token::Identifier(sp, "bit")], sp);
        assert_eq!(
            pattern(&mut Parser { tokens }),
            Ok(ast::UntypedPattern {
                kind: ast::UntypedPatternKind::Identifier(sp, "iden", type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(sp, "bit"), span: sp }),
                type_info: (),
                span: sp
            })
        );
    }
}
