use crate::compiler::{
    data::{token::Token, type_expr},
    error::Span,
    phases::parser::{type_, ParseError, Parser},
};

pub(super) fn type_<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<type_expr::TypeExpr<'file>, ParseError<'file>> {
    match *parser.peek() {
        Token::OBrack(obrack) => {
            parser.next();
            product(parser, obrack)
        }

        Token::TypeIdentifier(_) => {
            let iden = Token::type_identifier_matcher().convert(parser.next());
            let iden_span = iden.span;
            Ok(type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(iden), span: iden_span })
        }

        _ => Err(parser.expected_and_next("type")),
    }
}

fn product<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>, obrack: Span<'file>) -> Result<type_expr::TypeExpr<'file>, ParseError<'file>> {
    match parser.peek() {
        Token::Semicolon(_) => {
            parser.next();

            let (types, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), |parser| {
                let iden = parser.expect(Token::plain_identifier_matcher())?;
                parser.expect(Token::semicolon_matcher())?;
                let ty = type_::type_(parser)?;
                Ok((iden.name.to_string(), ty))
            })?;

            Ok(type_expr::TypeExpr { kind: type_expr::TypeExprKind::Product(types), span: obrack + cbrack })
        }

        Token::Number(_, _, _) => {
            let (len_sp, _, len) = Token::number_matcher().convert(parser.next());

            parser.expect(Token::cbrack_matcher())?;

            let ty = type_::type_(parser)?;
            let ty_span = ty.span;

            Ok(type_expr::TypeExpr { kind: type_expr::TypeExprKind::RepProduct((len_sp, len), Box::new(ty)), span: obrack + ty_span })
        }

        _ => {
            let (types, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), type_::type_)?;
            Ok(type_expr::TypeExpr { kind: type_expr::TypeExprKind::Product(types.into_iter().enumerate().map(|(i, t)| (i.to_string(), t)).collect()), span: obrack + cbrack })
        }
    }
}

// TODO: tests types
#[cfg(test)]
mod test {}
