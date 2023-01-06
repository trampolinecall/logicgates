use crate::compiler::{
    error::{CompileError, Span},
    ir::{named_type, ty},
};

pub(super) enum Error<'file> {
    TypeMismatch { /* got_span: Span<'file>, TODO */ expected_span: Span<'file>, got_type: ty::TypeSym, expected_type: ty::TypeSym },
}

impl<'file> From<(&ty::TypeContext<named_type::FullyDefinedNamedType>, Error<'file>)> for CompileError<'file> {
    fn from((types, val): (&ty::TypeContext<named_type::FullyDefinedNamedType>, Error<'file>)) -> Self {
        match val {
            Error::TypeMismatch { expected_span, got_type, expected_type } => CompileError::new(
                // TODO: show on the producer and receiver spans which has which type
                expected_span,
                format!("type mismatch: expected {}, got {}", types.get(expected_type).fmt(types), types.get(got_type).fmt(types)),
            ),
        }
    }
}
