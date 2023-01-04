#[macro_use]
mod error;

mod ir;

mod convert_circuit1;
mod convert_circuit2;
mod fill_types;
mod lexer;
mod make_name_tables;
mod parser;

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

    let ast = parser::parse(lexer::lex(&file));
    let ir = make_name_tables::make(ast)?;
    let ir = fill_types::fill(ir)?;
    let (mut types, circuit2) = convert_circuit1::convert(&file, ir)?;
    Some(convert_circuit2::convert(&mut types, &circuit2))
}
