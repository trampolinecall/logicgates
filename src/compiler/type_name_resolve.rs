use std::collections::HashMap;

use super::{
    arena,
    ir::{circuit1, named_type, ty},
    make_name_tables::{self, CircuitOrIntrinsicId, TypeDeclId},
};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<circuit1::UntypedCircuitOrIntrinsic<'file>, CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<String, CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<named_type::PartiallyDefinedNamedType<'file>>,
    pub(crate) type_table: HashMap<String, TypeDeclId>,
}

pub(crate) fn resolve(ir: make_name_tables::IR) -> Option<IR> {
    todo!()
}
