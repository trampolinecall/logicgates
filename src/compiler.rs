mod circuitgen;
mod error;
mod lexer;
mod parser;

use crate::circuit;

pub(crate) fn compile(filename: &str) -> Option<circuit::Circuit> {
    // TODO: do not return result if any errors are generated
    let file = load_file(filename)?;
    circuitgen::generate(parser::parse(lexer::lex(&file))?)
    todo!("compiling")
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
