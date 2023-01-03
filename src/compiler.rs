#[macro_use]
mod error;

mod ir;

mod convert_circuit;
mod convert_type_exprs;
mod lexer;
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
    let mut types = ir::ty::Types::new();
    let typed = convert_type_exprs::convert(&mut types, circuit1s, type_decls)?;
    let circuit2 = convert_circuit::convert(&file, &mut types, typed);
    todo!()
}
