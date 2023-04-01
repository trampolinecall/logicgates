use crate::compiler::{
    data::{ast, nominal_type, token::Token},
    phases::parser::{expr, pattern, type_, ParseError, Parser},
};

pub(super) fn circuit<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<ast::UntypedCircuit<'file>, ParseError<'file>> {
    let name = parser.expect(/* "circuit name", */ Token::circuit_identifier_matcher())?;
    let arguments = pattern::pattern(parser)?;
    let output_type = type_::type_(parser)?;
    let mut lets = Vec::new();
    while Token::let_matcher().matches(parser.peek()) {
        lets.push(let_(parser)?);
    }

    let ret = expr::expr(parser)?;

    Ok(ast::UntypedCircuit { name, input: arguments, lets, output: ret, output_type })
}

fn let_<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<ast::UntypedLet<'file>, ParseError<'file>> {
    parser.expect(Token::let_matcher())?;
    let pat = pattern::pattern(parser)?;

    parser.expect(Token::equals_matcher())?;

    let val = expr::expr(parser)?;
    Ok(ast::UntypedLet { pat, val })
}

pub(super) fn struct_<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<nominal_type::PartiallyDefinedStruct<'file>, ParseError<'file>> {
    parser.expect(Token::struct_matcher())?;
    let name = parser.expect(Token::type_identifier_matcher())?;
    parser.expect(Token::obrack_matcher())?;

    let mut fields = Vec::new(); // TODO: use type_::product() here
    while Token::plain_identifier_matcher().matches(parser.peek()) {
        let field_name = Token::plain_identifier_matcher().convert(parser.next());
        let field_ty = type_::type_(parser)?;
        fields.push((field_name, field_ty)); // TODO: anonymous fields too
    }

    parser.expect(Token::cbrack_matcher())?;

    Ok(nominal_type::Struct { name, fields })
}

#[cfg(test)]
mod test {
    use crate::compiler::{
        data::{ast, token::Token, type_expr},
        error::File,
        phases::parser::{parse, test::make_token_stream, Parser, AST},
    };

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
                circuits: vec![ast::UntypedCircuit {
                    name: (sp, "thingy"),
                    input: ast::UntypedPattern {
                        kind: ast::UntypedPatternKind::Identifier(sp, "arg", type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(sp, "bit"), span: sp }),
                        type_info: (),
                        span: sp
                    },
                    lets: vec![ast::UntypedLet {
                        pat: ast::UntypedPattern {
                            kind: ast::UntypedPatternKind::Identifier(sp, "res", type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(sp, "bit"), span: sp }),
                            type_info: (),
                            span: sp
                        },
                        val: ast::UntypedExpr {
                            kind: ast::UntypedExprKind::Call(
                                (sp, "and"),
                                false,
                                Box::new(ast::UntypedExpr {
                                    kind: ast::UntypedExprKind::Product(vec![
                                        ast::UntypedExpr { kind: ast::UntypedExprKind::Ref(sp, "arg"), type_info: (), span: sp },
                                        ast::UntypedExpr { kind: ast::UntypedExprKind::Ref(sp, "arg"), type_info: (), span: sp }
                                    ]),
                                    type_info: (),
                                    span: sp
                                })
                            ),
                            type_info: (),
                            span: sp
                        }
                    }],
                    output: ast::UntypedExpr { kind: ast::UntypedExprKind::Ref(sp, "res"), type_info: (), span: sp },
                    output_type: type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(sp, "bit"), span: sp }
                }],
                type_decls: vec![]
            }
        );
    }

    #[test]
    fn let_() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream([Token::Let(sp), Token::Identifier(sp, "a"), Token::Semicolon(sp), Token::Identifier(sp, "bit"), Token::Equals(sp), Token::Identifier(sp, "b")], sp);
        assert_eq!(
            super::let_(&mut Parser { tokens }),
            Ok(ast::UntypedLet {
                pat: ast::UntypedPattern {
                    kind: ast::UntypedPatternKind::Identifier(sp, "a", type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(sp, "bit"), span: sp }),
                    type_info: (),
                    span: sp
                },
                val: ast::UntypedExpr { kind: ast::UntypedExprKind::Ref(sp, "b"), type_info: (), span: sp }
            })
        );
    }

    // TODO: test struct ype decl
}
