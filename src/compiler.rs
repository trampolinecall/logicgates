#[macro_use]
mod error;

mod ir;

mod convert_circuit1;
mod convert_circuit2;
mod fill_types;
mod lexer;
mod make_circuit_table;
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

    let (circuit1s, type_decls) = parser::parse(lexer::lex(&file));
    let circuit1s = make_circuit_table::make(circuit1s)?;
    let (mut types, typed) = fill_types::fill(circuit1s, type_decls)?;
    let circuit2 = convert_circuit1::convert(&file, &mut types, typed)?;
    Some(convert_circuit2::convert(&mut types, &circuit2))
}
