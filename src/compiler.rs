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

    let (circuits, type_decls) = parser::parse(lexer::lex(&file));
    let mut types = ty::Types::new();
    let type_table = typing::define_types(&mut types, type_decls, )?;
    let typed = typing::type_(&mut types, &type_table, circuits)?;
    circuitgen::generate(&file, &mut types, typed)
}
