use crate::compiler::ir::{named_type, ty};

use super::GateIdx;

#[derive(Clone, Debug)]
pub(crate) enum ProducerBundle {
    CurCircuitInput(ty::TypeSym),
    GateOutput(ty::TypeSym, GateIdx),
    Get(Box<ProducerBundle>, String),
    Product(Vec<(String, ProducerBundle)>),
}
#[derive(Clone, Debug)]
pub(crate) enum ReceiverBundle {
    CurCircuitOutput(ty::TypeSym),
    GateInput(ty::TypeSym, GateIdx),
    // Get(Box<ReceiverBundle>, String), TODO: is this needed?
    // Product(Vec<(String, ReceiverBundle)>),
}

impl<'types> ProducerBundle {
    pub(crate) fn type_(&self, type_context: &'types mut ty::TypeContext<named_type::FullyDefinedNamedType>, circuit: &super::Circuit) -> ty::TypeSym {
        match self {
            ProducerBundle::CurCircuitInput(ty) => *ty,
            ProducerBundle::GateOutput(ty, gate_idx) => *ty,
            ProducerBundle::Get(producer, field) => {
                let producer_type = producer.type_(type_context, circuit);
                type_context.get(producer_type).field_type(type_context, field).unwrap()
            }
            ProducerBundle::Product(tys) => {
                let ty = ty::Type::Product(tys.iter().map(|(name, subbundle)| (name.to_string(), subbundle.type_(type_context, circuit))).collect());
                type_context.intern(ty)
            }
        }
    }
}
impl<'types> ReceiverBundle {
    pub(crate) fn type_(&self, type_context: &'types mut ty::TypeContext<named_type::FullyDefinedNamedType>, circuit: &super::Circuit) -> ty::TypeSym {
        match self {
            ReceiverBundle::CurCircuitOutput(ty) => *ty,
            ReceiverBundle::GateInput(ty, gate_idx) => *ty,
            /*
            ReceiverBundle::Get(producer, field) => {
                let producer_type = producer.type_(types, circuit);
                types.get(producer_type).field_type(types, field).unwrap()
            }
            ReceiverBundle::Product(tys) => {
                let ty = ty::Type::Product(tys.iter().map(|(name, subbundle)| (name.to_string(), subbundle.type_(types, circuit))).collect());
                types.intern(ty)
            }
            */
        }
    }
}
