use std::collections::HashMap;

use crate::circuit;

use super::error::CompileError;
use super::error::Report;
use super::parser::ast;

enum CircuitGenError<'file> {
    Duplicate(&'file str),
}

impl From<CircuitGenError<'_>> for CompileError {
    fn from(val: CircuitGenError) -> Self {
        match val {
            CircuitGenError::Duplicate(name) => CompileError { message: format!("circuit '{name}' defined more than once") },
        }
    }
}

pub(crate) fn generate(ast: Vec<ast::Circuit>) -> Option<circuit::Circuit> {
    let mut name_table = HashMap::new();

    for circuit in ast {
        let (name, circuit) = lower_circuit(circuit)?; // TODO: report multiple errors from this
        if name_table.contains_key(name) {
            CircuitGenError::Duplicate(name).report();
            None?
        } else {
            name_table.insert(name, circuit);
        }
    }
    todo!()
}

fn lower_circuit(circuit_ast: ast::Circuit) -> Option<(&str, circuit::Circuit)> {
    let name = circuit_ast.name;

    let gates: Vec<circuit::Gate> = Vec::new();
    for r#let in circuit_ast.lets {
        let result = lower_expr(r#let.val);
    }

    let circuit = circuit::Circuit { num_inputs: pattern_size(&circuit_ast.arguments), gates: todo!(), outputs: todo!() };

    Some((name, circuit))
}

fn lower_expr(val: Vec<ast::Expr>) -> Option<Vec<circuit::Value>> {
    todo!()
}

fn pattern_size(arguments: &[ast::Pattern]) -> usize {
    arguments.iter().map(|ast::Pattern(_, size)| size).sum::<usize>()
}
