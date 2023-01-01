use crate::compiler::error::Span;

#[derive(PartialEq, Debug)]
pub(crate) enum Token<'file> {
    EOF(Span<'file>),

    OBrack(Span<'file>),
    CBrack(Span<'file>),
    Semicolon(Span<'file>),

    Dot(Span<'file>),
    Comma(Span<'file>),

    Equals(Span<'file>),
    Arrow(Span<'file>),

    Let(Span<'file>),
    Inline(Span<'file>),
    Bundle(Span<'file>),
    Inputs(Span<'file>),
    Outputs(Span<'file>),
    // TODO: variadic arguments / bundles
    Backtick(Span<'file>),

    Number(Span<'file>, usize),
    Identifier(Span<'file>, &'file str),
}

impl<'file> Token<'file> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            Token::EOF(sp) => *sp,
            Token::OBrack(sp) => *sp,
            Token::CBrack(sp) => *sp,
            Token::Semicolon(sp) => *sp,
            Token::Dot(sp) => *sp,
            Token::Comma(sp) => *sp,
            Token::Equals(sp) => *sp,
            Token::Arrow(sp) => *sp,
            Token::Let(sp) => *sp,
            Token::Inline(sp) => *sp,
            Token::Bundle(sp) => *sp,
            Token::Inputs(sp) => *sp,
            Token::Outputs(sp) => *sp,
            Token::Backtick(sp) => *sp,
            Token::Number(sp, _) => *sp,
            Token::Identifier(sp, _) => *sp,
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct TokenMatcher<'file, TokData> {
    name: &'static str,
    matches: for<'t> fn(&'t Token<'file>) -> bool,
    convert: for<'t> fn(Token<'file>) -> TokData,
}

impl<'file, TokData> TokenMatcher<'file, TokData> {
    pub(crate) fn name(&self) -> &'static str {
        self.name
    }

    pub(crate) fn matches(&self, tok: &Token<'file>) -> bool {
        (self.matches)(tok)
    }

    pub(crate) fn convert(&self, tok: Token<'file>) -> TokData {
        (self.convert)(tok)
    }
}

mod names {
    pub(super) const EOF: &str = "eof";

    pub(super) const OBRACK: &str = "'['";
    pub(super) const CBRACK: &str = "']'";
    pub(super) const SEMICOLON: &str = "';'";

    pub(super) const DOT: &str = "'.'";
    pub(super) const COMMA: &str = "','";

    pub(super) const EQUALS: &str = "'='";
    pub(super) const ARROW: &str = "'->'";

    pub(super) const LET: &str = "'let'";
    pub(super) const INLINE: &str = "'inline'";
    pub(super) const BUNDLE: &str = "'bundle'";
    pub(super) const INPUTS: &str = "'inputs'";
    pub(super) const OUTPUTS: &str = "'outputs'";

    pub(super) const BACKTICK: &str = "'`'";

    // different names to hopefully signal to me writing the Display impl that these constants should not be used for that
    pub(super) const NUMBER_DESC_NAME: &str = "number";
    pub(super) const IDENTIFIER_DESC_NAME: &str = "identifier";
}

macro_rules! define_matcher {
    ($matcher_name:ident, $file:lifetime, $tok_data: ty, $name:path, $tok_pat:pat => $tok_extract:expr) => {
        mod $matcher_name {
            #![allow(dead_code, unused_imports)]
            use super::Token;
            use crate::compiler::error::Span;

            pub(super) fn matches(tok: &Token) -> bool {
                #[allow(unused_variables)]
                if let $tok_pat = tok {
                    true
                } else {
                    false
                }
            }

            pub(super) fn convert<$file>(tok: Token<$file>) -> $tok_data {
                if let $tok_pat = tok {
                    $tok_extract
                } else {
                    panic!("TokenMatcher convert() for {} failed: got {:?}", Token::$matcher_name().name(), tok)
                }
            }
        }

        impl<$file> Token<$file> {
            #[allow(dead_code)]
            pub(crate) const fn $matcher_name() -> TokenMatcher<$file, $tok_data> {
                TokenMatcher { name: $name, matches: $matcher_name::matches, convert: $matcher_name::convert }
            }
        }
    };
}

define_matcher!(eof_matcher, 'file, Span<'file>, names::EOF, Token::EOF(sp) => sp);

define_matcher!(obrack_matcher, 'file, Span<'file>, names::OBRACK, Token::OBrack(sp) => sp);
define_matcher!(cbrack_matcher, 'file, Span<'file>, names::OBRACK, Token::CBrack(sp) => sp);
define_matcher!(semicolon_matcher, 'file, Span<'file>, names::SEMICOLON, Token::Semicolon(sp) => sp);

define_matcher!(dot_matcher, 'file, Span<'file>, names::DOT, Token::Dot(sp) => sp);
define_matcher!(comma_matcher, 'file, Span<'file>, names::COMMA, Token::Comma(sp) => sp);

define_matcher!(equals_matcher, 'file, Span<'file>, names::EQUALS, Token::Equals(sp) => sp);
define_matcher!(arrow_matcher, 'file, Span<'file>, names::ARROW, Token::Arrow(sp) => sp);

define_matcher!(let_matcher, 'file, Span<'file>, names::LET, Token::Let(sp) => sp);
define_matcher!(inline_matcher, 'file, Span<'file>, names::INLINE, Token::Inline(sp) => sp);
define_matcher!(bundle_matcher, 'file, Span<'file>, names::BUNDLE, Token::Bundle(sp) => sp);
define_matcher!(inputs_matcher, 'file, Span<'file>, names::INPUTS, Token::Inputs(sp) => sp);
define_matcher!(outputs_matcher, 'file, Span<'file>, names::OUTPUTS, Token::Outputs(sp) => sp);

define_matcher!(backtick_matcher, 'file, Span<'file>, names::BACKTICK, Token::Backtick(sp) => sp);

define_matcher!(number_matcher, 'file, usize, names::NUMBER_DESC_NAME, Token::Number(_, n) => n);
define_matcher!(identifier_matcher, 'file, Span<'file>, names::IDENTIFIER_DESC_NAME, Token::Identifier(sp, _) => sp);

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::EOF(_) => write!(f, "{}", names::EOF),

            Token::OBrack(_) => write!(f, "{}", names::OBRACK),
            Token::CBrack(_) => write!(f, "{}", names::CBRACK),
            Token::Semicolon(_) => write!(f, "{}", names::SEMICOLON),

            Token::Dot(_) => write!(f, "{}", names::DOT),
            Token::Comma(_) => write!(f, "{}", names::COMMA),

            Token::Equals(_) => write!(f, "{}", names::EQUALS),
            Token::Arrow(_) => write!(f, "{}", names::ARROW),

            Token::Let(_) => write!(f, "{}", names::LET),
            Token::Inline(_) => write!(f, "{}", names::INLINE),
            Token::Bundle(_) => write!(f, "{}", names::BUNDLE),
            Token::Inputs(_) => write!(f, "{}", names::INPUTS),
            Token::Outputs(_) => write!(f, "{}", names::OUTPUTS),

            Token::Backtick(_) => write!(f, "{}", names::BACKTICK),

            Token::Number(_, n) => write!(f, "'{n}'"),
            Token::Identifier(_, i) => write!(f, "'{i}'"),
        }
    }
}
