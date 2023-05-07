use crate::compiler::{
    data::{ast, nominal_type, token::Token},
    phases::parser::{expr, pattern, type_, ParseError, Parser},
};

pub(super) fn circuit<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<ast::Circuit<'file, ast::Untyped>, ParseError<'file>> {
    let name = parser.expect(/* "circuit name", */ Token::circuit_identifier_matcher())?;
    let input = pattern::pattern(parser)?;
    let output = pattern::pattern(parser)?;

    let mut lets = Vec::new();
    let mut connects = Vec::new();
    let mut aliases = Vec::new();
    loop {
        if Token::let_matcher().matches(parser.peek()) {
            lets.push(let_(parser)?);
        } else if Token::connect_matcher().matches(parser.peek()) {
            connects.push(connect(parser)?);
        } else if Token::alias_matcher().matches(parser.peek()) {
            aliases.push(alias(parser)?);
        } else {
            break;
        }
    }

    Ok(ast::Circuit { name, input, output, lets, aliases, connects })
}

fn let_<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<ast::Let<'file, ast::Untyped>, ParseError<'file>> {
    parser.expect(Token::let_matcher())?;
    let gate = parser.expect(Token::circuit_identifier_matcher())?;
    let inputs = pattern::pattern(parser)?;
    let outputs = pattern::pattern(parser)?;

    Ok(ast::Let { inputs, outputs, gate })
}

fn connect<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<ast::Connect<'file, ast::Untyped>, ParseError<'file>> {
    parser.expect(Token::connect_matcher())?;
    let start = expr::expr(parser)?;
    let end = expr::expr(parser)?;
    Ok(ast::Connect { start, end })
}

fn alias<'file>(parser: &mut Parser<'file, impl Iterator<Item = Token<'file>>>) -> Result<ast::Alias<'file, ast::Untyped>, ParseError<'file>> {
    parser.expect(Token::alias_matcher())?;
    let pat = pattern::pattern(parser)?;
    parser.expect(Token::equals_matcher())?;
    let expr = expr::expr(parser)?;
    Ok(ast::Alias { pat, expr })
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
        data::{
            ast,
            token::{self, Token},
            type_expr,
        },
        error::File,
        phases::parser::{parse, test::make_token_stream, Parser, AST},
    };

    // TODO: test inline calls
    #[test]
    fn circuit() {
        let file = File::test_file();
        let sp = file.eof_span();

        /*
        \thingy arg; -bit -bit
            let res; -bit = \and [arg, arg]
            res
        */
        let tokens = make_token_stream(
            [
                Token::CircuitIdentifier(token::CircuitIdentifier { span: sp, name: "thingy", with_tag: "\\thingy".to_string() }),
                Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "arg" }),
                Token::Semicolon(sp),
                Token::TypeIdentifier(token::TypeIdentifier { span: sp, name: "bit", with_tag: "-bit".to_string() }),
                Token::TypeIdentifier(token::TypeIdentifier { span: sp, name: "bit", with_tag: "-bit".to_string() }),
                Token::Let(sp),
                Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "res" }),
                Token::Semicolon(sp),
                Token::TypeIdentifier(token::TypeIdentifier { span: sp, name: "bit", with_tag: "-bit".to_string() }),
                Token::Equals(sp),
                Token::CircuitIdentifier(token::CircuitIdentifier { span: sp, name: "and", with_tag: "\\and".to_string() }),
                Token::OBrack(sp),
                Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "arg" }),
                Token::Comma(sp),
                Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "arg" }),
                Token::CBrack(sp),
                Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "res" }),
            ],
            sp,
        );

        assert_eq!(
            parse(tokens),
            AST {
                circuits: vec![ast::Circuit {
                    name: token::CircuitIdentifier { span: sp, name: "thingy", with_tag: "\\thingy".to_string() },
                    input: ast::Pattern {
                        kind: ast::PatternKind::Identifier(
                            token::PlainIdentifier { span: sp, name: "arg" },
                            type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(token::TypeIdentifier { span: sp, name: "bit", with_tag: "-bit".to_string() }), span: sp }
                        ),
                        type_info: (),
                        span: sp
                    },
                    lets: vec![ast::Let {
                        pat: ast::Pattern {
                            kind: ast::PatternKind::Identifier(
                                token::PlainIdentifier { span: sp, name: "res" },
                                type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(token::TypeIdentifier { span: sp, name: "bit", with_tag: "-bit".to_string() }), span: sp }
                            ),
                            type_info: (),
                            span: sp
                        },
                        val: ast::Expr {
                            kind: ast::ExprKind::Call(
                                token::CircuitIdentifier { span: sp, name: "and", with_tag: "\\and".to_string() },
                                false,
                                Box::new(ast::Expr {
                                    kind: ast::ExprKind::Product(vec![
                                        ("0".to_string(), ast::Expr { kind: ast::ExprKind::Ref(token::PlainIdentifier { span: sp, name: "arg" }), type_info: (), span: sp }),
                                        ("1".to_string(), ast::Expr { kind: ast::ExprKind::Ref(token::PlainIdentifier { span: sp, name: "arg" }), type_info: (), span: sp }),
                                    ]),
                                    type_info: (),
                                    span: sp
                                })
                            ),
                            type_info: (),
                            span: sp
                        }
                    }],
                    output: ast::Expr { kind: ast::ExprKind::Ref(token::PlainIdentifier { span: sp, name: "res" }), type_info: (), span: sp },
                    output: type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(token::TypeIdentifier { span: sp, name: "bit", with_tag: "-bit".to_string() }), span: sp }
                }],
                type_decls: vec![]
            }
        );
    }

    #[test]
    fn let_() {
        let file = File::test_file();
        let sp = file.eof_span();

        let tokens = make_token_stream(
            [
                Token::Let(sp),
                Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "a" }),
                Token::Semicolon(sp),
                Token::TypeIdentifier(token::TypeIdentifier { span: sp, name: "bit", with_tag: "-bit".to_string() }),
                Token::Equals(sp),
                Token::PlainIdentifier(token::PlainIdentifier { span: sp, name: "b" }),
            ],
            sp,
        );
        assert_eq!(
            super::let_(&mut Parser { tokens }),
            Ok(ast::Let {
                pat: ast::Pattern {
                    kind: ast::PatternKind::Identifier(
                        token::PlainIdentifier { span: sp, name: "a" },
                        type_expr::TypeExpr { kind: type_expr::TypeExprKind::Nominal(token::TypeIdentifier { span: sp, name: "bit", with_tag: "-bit".to_string() }), span: sp }
                    ),
                    type_info: (),
                    span: sp
                },
                val: ast::Expr { kind: ast::ExprKind::Ref(token::PlainIdentifier { span: sp, name: "b" }), type_info: (), span: sp }
            })
        );
    }

    // TODO: test struct type decl
}
