mod circuitgen;
mod error;
mod lexer;
mod parser;

use crate::circuit;

pub fn compile(filename: &str) -> Option<circuit::Circuit> {
    let file = load_file(filename)?;
    let tokens = lexer::lex(&file);
    circuitgen::generate(parser::parse(tokens)?)
}

fn load_file(filename: &str) -> Option<String> {
    match std::fs::read_to_string(filename) {
        Ok(o) => Some(o),
        Err(e) => {
            eprintln!("{e}");
            None
        }
    }
}
