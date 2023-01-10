use crate::compiler::error::Span;

#[derive(PartialEq, Debug, Copy, Clone)]
pub(crate) struct PlainIdentifier<'file> {
    pub(crate) span: Span<'file>,
    pub(crate) name: &'file str,
}

#[derive(PartialEq, Debug)]
pub(crate) struct TypeIdentifier<'file> {
    pub(crate) span: Span<'file>,
    pub(crate) name: &'file str,
    pub(crate) with_tag: String,
}

#[derive(PartialEq, Debug)]
pub(crate) struct CircuitIdentifier<'file> {
    pub(crate) span: Span<'file>,
    pub(crate) name: &'file str,
    pub(crate) with_tag: String,
}

#[derive(PartialEq, Debug)]
pub(crate) enum Token<'file> {
    EOF(Span<'file>),

    OBrack(Span<'file>),
    CBrack(Span<'file>),
    Semicolon(Span<'file>),

    Dot(Span<'file>),
    Comma(Span<'file>),

    Equals(Span<'file>),

    Let(Span<'file>),
    Inline(Span<'file>),
    Struct(Span<'file>),

    // TODO: variadic arguments / bundles
    Backslash(Span<'file>),
    Slash(Span<'file>),

    Number(Span<'file>, &'file str, usize),
    PlainIdentifier(PlainIdentifier<'file>),
    TypeIdentifier(TypeIdentifier<'file>),
    CircuitIdentifier(CircuitIdentifier<'file>),
}

impl<'file> Token<'file> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            Token::EOF(sp)
            | Token::OBrack(sp)
            | Token::CBrack(sp)
            | Token::Semicolon(sp)
            | Token::Dot(sp)
            | Token::Comma(sp)
            | Token::Equals(sp)
            | Token::Let(sp)
            | Token::Inline(sp)
            | Token::Number(sp, _, _)
            | Token::Struct(sp)
            | Token::Backslash(sp)
            | Token::Slash(sp) => *sp,
            Token::PlainIdentifier(i) => i.span,
            Token::TypeIdentifier(t) => t.span,
            Token::CircuitIdentifier(c) => c.span,
        }
    }
}

pub(crate) struct TokenMatcher<'file, TokData> {
    name: &'static str,
    matches: for<'t> fn(&'t Token<'file>) -> bool,
    convert: for<'t> fn(Token<'file>) -> TokData,
}
impl<TokData> Clone for TokenMatcher<'_, TokData> {
    fn clone(&self) -> Self {
        Self { name: self.name, matches: self.matches, convert: self.convert }
    }
}
impl<TokData> Copy for TokenMatcher<'_, TokData> {}

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

    pub(super) const LET: &str = "'let'";
    pub(super) const INLINE: &str = "'inline'";
    pub(super) const STRUCT: &str = "'struct'";

    pub(super) const BACKSLASH: &str = "'\\'";
    pub(super) const SLASH: &str = "'/'";

    // different names to hopefully signal to me writing the Display impl that these constants should not be used for that
    pub(super) const NUMBER_DESC_NAME: &str = "number";
    pub(super) const TYPE_IDENTIFIER_DESC_NAME: &str = "type identifier";
    pub(super) const CIRCUIT_IDENTIFIER_DESC_NAME: &str = "circuit identifier";
    pub(super) const PLAIN_IDENTIFIER_DESC_NAME: &str = "identifier";
}

macro_rules! define_matcher {
    ($matcher_name:ident, $file:lifetime, $tok_data: ty, $name:path, $tok_pat:pat => $tok_extract:expr) => {
        mod $matcher_name {
            #![allow(dead_code, unused_imports)]
            use super::*;

            #[inline]
            pub(super) const fn matches(tok: &Token) -> bool {
                #[allow(unused_variables)]
                if let $tok_pat = tok {
                    true
                } else {
                    false
                }
            }

            #[inline]
            pub(super) fn convert<$file>(tok: Token<$file>) -> $tok_data {
                if let $tok_pat = tok {
                    $tok_extract
                } else {
                    panic!("TokenMatcher convert() for {} failed: got {:?}", Token::$matcher_name().name(), tok)
                }
            }
        }

        impl<$file> Token<$file> {
            #[inline]
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

define_matcher!(let_matcher, 'file, Span<'file>, names::LET, Token::Let(sp) => sp);
define_matcher!(inline_matcher, 'file, Span<'file>, names::INLINE, Token::Inline(sp) => sp);
define_matcher!(struct_matcher, 'file, Span<'file>, names::STRUCT, Token::Struct(sp) => sp);

define_matcher!(number_matcher, 'file, (Span<'file>, &'file str, usize), names::NUMBER_DESC_NAME, Token::Number(sp, n_str, n) => (sp, n_str, n));
define_matcher!(plain_identifier_matcher, 'file, PlainIdentifier<'file>, names::PLAIN_IDENTIFIER_DESC_NAME, Token::PlainIdentifier(i) => i);
define_matcher!(type_identifier_matcher, 'file, TypeIdentifier<'file>, names::TYPE_IDENTIFIER_DESC_NAME, Token::TypeIdentifier(i) => i);
define_matcher!(circuit_identifier_matcher, 'file, CircuitIdentifier<'file>, names::CIRCUIT_IDENTIFIER_DESC_NAME, Token::CircuitIdentifier(i) => i);

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

            Token::Let(_) => write!(f, "{}", names::LET),
            Token::Inline(_) => write!(f, "{}", names::INLINE),
            Token::Struct(_) => write!(f, "{}", names::STRUCT),

            Token::Backslash(_) => write!(f, "{}", names::BACKSLASH),
            Token::Slash(_) => write!(f, "{}", names::SLASH),

            Token::Number(_, _, n) => write!(f, "number '{n}'"),
            Token::PlainIdentifier(i) => write!(f, "identifier '{}'", i.name),
            Token::TypeIdentifier(i) => write!(f, "type identifier '{}'", i.with_tag),
            Token::CircuitIdentifier(i) => write!(f, "circuit identifier '{}'", i.with_tag),
        }
    }
}
