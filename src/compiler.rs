#[macro_use]
mod error;

mod data;
mod phases;

use crate::simulation::circuit;
use error::File;

pub(crate) fn compile(filename: &str) -> Option<circuit::Circuit> {
    // TODO: do not return result if any errors are generated
    let file = match File::load(filename) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("error: {e}");
            return None;
        }
    };

    let tokens = phases::lexer::lex(&file);
    let ast = phases::parser::parse(tokens);
    let ir = phases::make_name_tables::make(ast)?;
    let ir = phases::resolve_type_expr::resolve(ir)?;
    let ir = phases::type_pats::type_(ir);
    let ir = phases::type_exprs::type_(ir)?;
    let ir = phases::convert_circuit1::convert(ir)?;
    phases::convert_circuit2::convert(&file, ir)
}
