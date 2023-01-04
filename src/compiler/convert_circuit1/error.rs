use crate::compiler::{
    error::{CompileError, File, Span},
    ir::ty,
};

pub(super) enum Error<'file> {
    NoField { ty: ty::TypeSym, field_name_sp: Span<'file>, field_name: &'file str }, // TODO: list names of fields that do exist
    NoSuchLocal(Span<'file>, &'file str),
    NoSuchCircuit(Span<'file>, &'file str),
    TypeMismatch { /* got_span: Span<'file>, TODO */ expected_span: Span<'file>, got_type: ty::TypeSym, expected_type: ty::TypeSym },
    NoMain(&'file File),
}

impl<'file> From<(&ty::TypeContext, Error<'file>)> for CompileError<'file> {
    fn from((types, val): (&ty::TypeContext, Error<'file>)) -> Self {
        match val {
            Error::NoField { ty, field_name_sp, field_name } => CompileError::new(field_name_sp, format!("no field called '{}' on type '{}'", field_name, types.get(ty).fmt(types))),
            Error::NoSuchLocal(name_sp, name) => CompileError::new(name_sp, format!("no local called '{}'", name)),
            Error::NoSuchCircuit(name_sp, name) => CompileError::new(name_sp, format!("no circuit called '{}'", name)),
            Error::NoMain(f) => CompileError::new(f.eof_span(), "no 'main' circuit".into()),
            Error::TypeMismatch { expected_span, got_type, expected_type } => CompileError::new(
                // TODO: show on the producer and receiver spans which has which type
                expected_span,
                format!("type mismatch: expected {}, got {}", types.get(expected_type).fmt(types), types.get(got_type).fmt(types)),
            ),
        }
    }
}
