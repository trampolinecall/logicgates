pub(crate) mod ast;

use crate::compiler::error::CompileError;
use crate::compiler::error::Report;
use crate::compiler::lexer::token::TokenMatcher;
use crate::compiler::lexer::Token;
use std::iter::Peekable;

struct Parser<'file, T: Iterator<Item = Token<'file>>> {
    tokens: Peekable<T>,
}

#[derive(Debug, PartialEq)]
struct ParseError {
    // TODO: more descriptive messages
    expected: &'static str,
    got: String,
}

impl From<ParseError> for CompileError {
    fn from(ParseError { expected, got }: ParseError) -> Self {
        CompileError { message: format!("expected {expected}, got {got}") }
    }
}

impl<'file, T: Iterator<Item = Token<'file>>> Parser<'file, T> {
    fn peek(&mut self) -> &Token<'file> {
        self.tokens.peek().unwrap()
    }
    fn next(&mut self) -> Token<'file> {
        self.tokens.next().unwrap()
    }

    fn expect<TokData>(&mut self, matcher: TokenMatcher<'file, TokData>) -> Result<TokData, ParseError> {
        if matcher.matches(self.peek()) {
            Ok(matcher.convert(self.next()))
        } else {
            Err(self.expected(matcher.name()))
        }
    }
    fn maybe_consume<TokData>(&mut self, matcher: TokenMatcher<'file, TokData>) -> Option<TokData> {
        if matcher.matches(self.peek()) {
            Some(matcher.convert(self.next()))
        } else {
            None
        }
    }

    fn expected(&mut self, thing: &'static str) -> ParseError {
        ParseError { expected: thing, got: self.peek().to_string() }
    }

    fn list<StartData, DelimData, EndData, A>(
        &mut self,
        start: TokenMatcher<'file, StartData>,
        delim: TokenMatcher<'file, DelimData>,
        ending: TokenMatcher<'file, EndData>,
        thing: impl for<'p> FnMut(&'p mut Parser<'file, T>) -> Result<A, ParseError>,
    ) -> Result<Vec<A>, ParseError> {
        self.expect(start)?;
        self.finish_list(delim, ending, thing)
    }

    fn finish_list<DelimData, EndData, A>(
        &mut self,
        delim: TokenMatcher<'file, DelimData>,
        ending: TokenMatcher<'file, EndData>,
        mut thing: impl for<'p> FnMut(&'p mut Parser<'file, T>) -> Result<A, ParseError>,
    ) -> Result<Vec<A>, ParseError> {
        let mut items = Vec::new();

        while !ending.matches(self.peek()) {
            items.push(thing(self)?);

            if delim.matches(self.peek()) {
                self.next(); // there is a delimiter, the list may or may not continue
            } else {
                break; // if there is no delimiter, the list cannot be continued
            }
        }

        self.expect(ending)?;

        Ok(items)
    }
}

impl<'file, T: Iterator<Item = Token<'file>>> Parser<'file, T> {
    fn parse(&mut self) -> Option<Vec<ast::Circuit<'file>>> {
        let mut circuits = Vec::new();
        while !Token::eof_matcher().matches(self.peek()) {
            match self.circuit() {
                Ok(circuit) => circuits.push(circuit),
                Err(e) => {
                    self.next();
                    e.report();
                }
            }
        }

        Some(circuits)
    }

    fn circuit(&mut self) -> Result<ast::Circuit<'file>, ParseError> {
        self.expect(/* TODO: "circuit name (starting with '`')", */ Token::backtick_matcher())?;
        let name = self.expect(/* "circuit name after '`'", */ Token::identifier_matcher())?;
        let arguments = self.pattern()?;
        let mut lets = Vec::new();

        while Token::let_matcher().matches(self.peek()) {
            lets.push(self.r#let()?);
        }

        let ret = self.expr()?;

        Ok(ast::Circuit { name, input: arguments, lets, output: ret })
    }

    fn r#let(&mut self) -> Result<ast::Let<'file>, ParseError> {
        self.expect(Token::let_matcher())?;
        let pat = self.pattern()?;

        self.expect(Token::equals_matcher())?;

        let val = self.expr()?;
        Ok(ast::Let { pat, val })
    }

    fn pattern(&mut self) -> Result<ast::Pattern<'file>, ParseError> {
        match self.peek() {
            Token::Identifier(_) => {
                let iden = Token::identifier_matcher().convert(self.next());

                self.expect(Token::semicolon_matcher())?;

                let type_ = self.type_()?;

                Ok(ast::Pattern::Identifier(iden, type_))
            }

            Token::OBrack => {
                self.next();

                let patterns = self.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::pattern)?;
                Ok(ast::Pattern::Product(patterns))
            }
            _ => Err(self.expected("pattern")),
        }
    }

    fn expr(&mut self) -> Result<ast::Expr<'file>, ParseError> {
        let mut left = self.primary_expr()?;

        while Token::dot_matcher().matches(self.peek()) {
            self.next();

            let field: &'file str = match self.peek() {
                Token::Number(n_str, _) => Ok(n_str),

                Token::Identifier(i) => Ok(i),

                _ => Err(self.expected("field name (a number or identifier)")),
            }?;
            self.next();

            left = ast::Expr::Get(Box::new(left), field);
        }

        Ok(left)
    }

    fn primary_expr(&mut self) -> Result<ast::Expr<'file>, ParseError> {
        match self.peek() {
            Token::Number(_, _) => {
                let n = Token::number_matcher().convert(self.next());

                match n {
                    0 => Ok(ast::Expr::Const(false)),
                    1 => Ok(ast::Expr::Const(true)),
                    _ => Err(self.expected("'0' or '1'")),
                }
            }
            Token::Backtick => {
                let _ = self.next();
                let i = self.expect(/* "circuit name after '`'", */ Token::identifier_matcher())?;

                let inline = self.maybe_consume(Token::inline_matcher()).is_some();

                let arg = self.expr()?;

                Ok(ast::Expr::Call(i, inline, Box::new(arg)))
            }

            Token::Identifier(_) => {
                let i = Token::identifier_matcher().convert(self.next());

                Ok(ast::Expr::Ref(i))
            }

            Token::OBrack => {
                self.next();

                let mut items = Vec::new();

                if !Token::cbrack_matcher().matches(self.peek()) {
                    items.push(self.expr()?);
                    while Token::comma_matcher().matches(self.peek()) {
                        self.next();
                        items.push(self.expr()?);
                    }
                }

                self.expect(Token::cbrack_matcher())?;

                Ok(ast::Expr::Multiple(items))
            }

            _ => Err(self.expected("expression"))?,
        }
    }

    fn type_(&mut self) -> Result<ast::Type, ParseError> {
        match self.peek() {
            Token::Backtick => {
                let _ = self.next();
                Ok(ast::Type::Bit)
            }

            Token::OBrack => {
                let _ = self.next();

                match self.peek() {
                    Token::Number(_, _) => {
                        let len = Token::number_matcher().convert(self.next());

                        self.expect(Token::cbrack_matcher())?;

                        let ty = self.type_()?;

                        Ok(ast::Type::Product((0..len).map(|_| ty.clone()).collect()))
                    }

                    _ => {
                        let tys = self.finish_list(Token::comma_matcher(), Token::cbrack_matcher(), Parser::type_)?;
                        Ok(ast::Type::Product(tys))
                    }
                }
            }

            _ => Err(self.expected("type")),
        }
    }
}

pub(crate) fn parse<'file>(tokens: impl Iterator<Item = Token<'file>>) -> Option<Vec<ast::Circuit<'file>>> {
    Parser { tokens: tokens.peekable() }.parse()
}

#[cfg(test)]
mod test {
    use super::ast;
    use super::parse;
    use super::Parser;
    use crate::compiler::lexer::Token;

    use std::iter::Peekable;

    fn make_token_stream(tokens: Vec<Token>) -> Peekable<impl Iterator<Item = Token>> {
        tokens.into_iter().chain(std::iter::repeat_with(|| Token::EOF)).peekable()
    }

    #[test]
    fn list() {
        let tokens = vec![Token::OBrack, Token::Identifier("a"), Token::Comma, Token::Identifier("b"), Token::CBrack];
        assert_eq!(
            Parser { tokens: make_token_stream(tokens) }.list(Token::obrack_matcher(), Token::comma_matcher(), Token::cbrack_matcher(), Parser::expr),
            Ok(vec![ast::Expr::Ref("a"), ast::Expr::Ref("b")])
        )
    }

    #[test]
    fn list_trailing_delim() {
        let tokens = vec![Token::OBrack, Token::Identifier("a"), Token::Comma, Token::Identifier("b"), Token::Comma, Token::CBrack];
        assert_eq!(
            Parser { tokens: make_token_stream(tokens) }.list(Token::obrack_matcher(), Token::comma_matcher(), Token::cbrack_matcher(), Parser::expr),
            Ok(vec![ast::Expr::Ref("a"), ast::Expr::Ref("b")])
        )
    }

    // TODO: test inline calls
    #[test]
    fn circuit() {
        /*
        `thingy arg; `
            let res; ` = `and [arg, arg]
            res
        */
        let tokens = vec![
            Token::Backtick,
            Token::Identifier("thingy"),
            Token::Identifier("arg"),
            Token::Semicolon,
            Token::Backtick,
            Token::Let,
            Token::Identifier("res"),
            Token::Semicolon,
            Token::Backtick,
            Token::Equals,
            Token::Backtick,
            Token::Identifier("and"),
            Token::OBrack,
            Token::Identifier("arg"),
            Token::Comma,
            Token::Identifier("arg"),
            Token::CBrack,
            Token::Identifier("res"),
        ];

        assert_eq!(
            parse(make_token_stream(tokens)),
            Some(vec![ast::Circuit {
                name: "thingy",
                input: ast::Pattern::Identifier("arg", ast::Type::Bit),
                lets: vec![ast::Let {
                    pat: ast::Pattern::Identifier("res", ast::Type::Bit),
                    val: ast::Expr::Call("and", false, Box::new(ast::Expr::Multiple(vec![ast::Expr::Ref("arg"), ast::Expr::Ref("arg")])))
                }],
                output: ast::Expr::Ref("res")
            }])
        )
    }

    #[test]
    fn r#let() {
        let tokens = vec![Token::Let, Token::Identifier("a"), Token::Semicolon, Token::Backtick, Token::Equals, Token::Identifier("b")];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.r#let(), Ok(ast::Let { pat: ast::Pattern::Identifier("a", ast::Type::Bit), val: ast::Expr::Ref("b") }))
    }

    #[test]
    fn iden_pattern() {
        let tokens = vec![Token::Identifier("iden"), Token::Semicolon, Token::Backtick];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.pattern(), Ok(ast::Pattern::Identifier("iden", ast::Type::Bit)))
    }

    #[test]
    fn const_false_expr() {
        let tokens = vec![Token::Number("0", 0)];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.expr(), Ok(ast::Expr::Const(false)))
    }

    #[test]
    fn const_true_expr() {
        let tokens = vec![Token::Number("0", 1)];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.expr(), Ok(ast::Expr::Const(true)))
    }

    #[test]
    fn call_expr() {
        let tokens = vec![Token::Backtick, Token::Identifier("a"), Token::Identifier("b")];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.expr(), Ok(ast::Expr::Call("a", false, Box::new(ast::Expr::Ref("b")))))
    }

    #[test]
    fn iden_expr() {
        let tokens = vec![Token::Identifier("a")];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.expr(), Ok(ast::Expr::Ref("a")))
    }

    #[test]
    fn multiple_expr() {
        let tokens = vec![Token::OBrack, Token::Identifier("a"), Token::Comma, Token::Identifier("b"), Token::Comma, Token::Number("0", 0), Token::Comma, Token::Number("0", 1), Token::CBrack];
        assert_eq!(Parser { tokens: make_token_stream(tokens) }.expr(), Ok(ast::Expr::Multiple(vec![ast::Expr::Ref("a"), ast::Expr::Ref("b"), ast::Expr::Const(false), ast::Expr::Const(true)])))
    }

    // TODO: test array types, types in general
}
