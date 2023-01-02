mod circuitgen;
#[macro_use]
mod error;
mod ir;
mod lexer;
mod parser;
mod ty;
mod typing;

use crate::circuit;

use self::error::File;

pub(crate) fn compile(filename: &str) -> Option<circuit::Circuit> {
    // TODO: do not return result if any errors are generated
    let file = match File::load(filename) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("error: {e}");
            return None;
        }
    };

    circuitgen::generate(&file, typing::type_(parser::parse(lexer::lex(&file))))
}
