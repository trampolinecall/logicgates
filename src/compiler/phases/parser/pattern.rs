use crate::compiler::{
    data::{circuit1, token::Token},
    phases::parser::{pattern, type_, ParseError, Parser},
};

pub(super) fn pattern<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<circuit1::UntypedPattern<'file>, ParseError<'file>> {
    match parser.peek() {
        Token::Identifier(_, _) => {
            let iden = Token::identifier_matcher().convert(parser.next());
            parser.expect(Token::semicolon_matcher())?;

            let type_ = type_::type_(parser)?;
            let type_span = type_.span;

            Ok(circuit1::UntypedPattern { kind: circuit1::UntypedPatternKind::Identifier(iden.0, iden.1, type_), type_info: (), span: iden.0 + type_span })
        }

        &Token::OBrack(obrack) => {
            parser.next();
            let (patterns, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), pattern::pattern)?;

            Ok(circuit1::UntypedPattern { kind: circuit1::UntypedPatternKind::Product(obrack + cbrack, patterns), type_info: (), span: obrack + cbrack })
        }
        _ => Err(parser.expected_and_next("pattern")),
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::{
        data::{circuit1, token::Token, type_expr},
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
            Ok(circuit1::UntypedPattern {
                kind: circuit1::UntypedPatternKind::Identifier(sp, "iden", type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(sp, "bit"), span: sp }),
                type_info: (),
                span: sp
            })
        );
    }
}