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

        Token::Identifier(_, _) => {
            let iden = Token::identifier_matcher().convert(parser.next());

            Ok(type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(iden.0, iden.1), span: iden.0 })
        }

        _ => Err(parser.expected_and_next("type")),
    }
}

fn product<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>, obrack: Span<'file>) -> Result<type_expr::TypeExpr<'file>, ParseError<'file>> {
    match parser.peek() {
        &Token::Named(_) => {
            parser.next();

            let (types, cbrack) = parser.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), |parser| {
                let name = parser.expect(Token::identifier_matcher())?;
                parser.expect(Token::semicolon_matcher())?;
                let ty = type_::type_(parser)?;
                Ok((name, ty))
            })?;

            Ok(type_expr::TypeExpr { kind: type_expr::TypeExprKind::NamedProduct(types), span: obrack + cbrack })
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
            Ok(type_expr::TypeExpr { kind: type_expr::TypeExprKind::Product(types), span: obrack + cbrack })
        }
    }
}

// TODO: tests types
#[cfg(test)]
mod test {}
