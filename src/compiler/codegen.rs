use std::collections::HashMap;
use super::parser::ast;

pub(crate) fn compile(ast: HashMap<&str, ast::Gate>) -> Option<crate::circuit::Circuit> {
    todo!()
}
