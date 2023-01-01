#[derive(PartialEq, Debug)]
pub(crate) enum Token<'file> {
    EOF,

    OBrack,
    CBrack,
    Semicolon,

    Dot,
    Comma,

    Equals,
    Arrow,

    Let,
    Inline,
    Bundle,
    Inputs,
    Outputs,
    // TODO: variadic arguments / bundles
    Backtick,

    Number(&'file str, usize),
    Identifier(&'file str),
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
            #![allow(dead_code)]
            use super::Token;

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

define_matcher!(eof_matcher, 'file, (), names::EOF, Token::EOF => ());

define_matcher!(obrack_matcher, 'file, (), names::OBRACK, Token::OBrack => ());
define_matcher!(cbrack_matcher, 'file, (), names::OBRACK, Token::CBrack => ());
define_matcher!(semicolon_matcher, 'file, (), names::SEMICOLON, Token::Semicolon => ());

define_matcher!(dot_matcher, 'file, (), names::DOT, Token::Dot => ());
define_matcher!(comma_matcher, 'file, (), names::COMMA, Token::Comma => ());

define_matcher!(equals_matcher, 'file, (), names::EQUALS, Token::Equals => ());
define_matcher!(arrow_matcher, 'file, (), names::ARROW, Token::Arrow => ());

define_matcher!(let_matcher, 'file, (), names::LET, Token::Let => ());
define_matcher!(inline_matcher, 'file, (), names::INLINE, Token::Inline => ());
define_matcher!(bundle_matcher, 'file, (), names::BUNDLE, Token::Bundle => ());
define_matcher!(inputs_matcher, 'file, (), names::INPUTS, Token::Inputs => ());
define_matcher!(outputs_matcher, 'file, (), names::OUTPUTS, Token::Outputs => ());

define_matcher!(backtick_matcher, 'file, (), names::BACKTICK, Token::Backtick => ());

define_matcher!(number_matcher, 'file, usize, names::NUMBER_DESC_NAME, Token::Number(_, n) => n);
define_matcher!(identifier_matcher, 'file, &'file str, names::IDENTIFIER_DESC_NAME, Token::Identifier(i) => i);

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::EOF => write!(f, "{}", names::EOF),

            Token::OBrack => write!(f, "{}", names::OBRACK),
            Token::CBrack => write!(f, "{}", names::CBRACK),
            Token::Semicolon => write!(f, "{}", names::SEMICOLON),

            Token::Dot => write!(f, "{}", names::DOT),
            Token::Comma => write!(f, "{}", names::COMMA),

            Token::Equals => write!(f, "{}", names::EQUALS),
            Token::Arrow => write!(f, "{}", names::ARROW),

            Token::Let => write!(f, "{}", names::LET),
            Token::Inline => write!(f, "{}", names::INLINE),
            Token::Bundle => write!(f, "{}", names::BUNDLE),
            Token::Inputs => write!(f, "{}", names::INPUTS),
            Token::Outputs => write!(f, "{}", names::OUTPUTS),

            Token::Backtick => write!(f, "{}", names::BACKTICK),

            Token::Number(_, n) => write!(f, "'{n}'"),
            Token::Identifier(i) => write!(f, "'{i}'"),
        }
    }
}
