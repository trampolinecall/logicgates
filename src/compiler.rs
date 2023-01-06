#[macro_use]
mod error;

#[macro_use]
pub(crate) mod arena;

mod ir;

mod convert_circuit1;
mod convert_circuit2;
mod type_exprs;
mod lexer;
mod make_name_tables;
mod parser;
mod resolve_type_expr;
mod type_pats;

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
    let ir = resolve_type_expr::resolve(ir)?;
    let ir = type_pats::type_(ir);
    let ir = type_exprs::type_(ir)?;
    let ir = convert_circuit1::convert(&file, ir)?;
    Some(convert_circuit2::convert(&file, ir)?)
}
