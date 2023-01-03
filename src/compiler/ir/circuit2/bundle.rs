use crate::compiler::ir::ty;

use super::GateIdx;

#[derive(Clone, Debug)]
pub(crate) enum ProducerBundle {
    CurCircuitInput,
    GateOutput(GateIdx),
    Get(Box<ProducerBundle>, String),
    Product(Vec<(String, ProducerBundle)>),
}
#[derive(Clone, Debug)]
pub(crate) enum ReceiverBundle {
    CurCircuitOutput,
    GateInput(GateIdx),
    // Get(Box<ReceiverBundle>, String), TODO: is this needed?
    // Product(Vec<(String, ReceiverBundle)>),
}

impl<'types> ProducerBundle {
    pub(crate) fn type_(&self, types: &'types mut ty::Types, circuit: &super::Circuit) -> ty::TypeSym {
        match self {
            ProducerBundle::CurCircuitInput => circuit.input_type,
            ProducerBundle::GateOutput(gate_idx) => circuit.get_gate(*gate_idx).output_type(types),
            ProducerBundle::Get(producer, field) => {
                let producer_type = producer.type_(types, circuit);
                types.get(producer_type).field_type(types, field).unwrap()
            }
            ProducerBundle::Product(tys) => {
                let ty = ty::Type::Product(tys.iter().map(|(name, subbundle)| (name.to_string(), subbundle.type_(types, circuit))).collect());
                types.intern(ty)
            }
        }
    }
}
impl<'types> ReceiverBundle {
    pub(crate) fn type_(&self, types: &'types mut ty::Types, circuit: &super::Circuit) -> ty::TypeSym {
        match self {
            ReceiverBundle::CurCircuitOutput => circuit.output_type,
            ReceiverBundle::GateInput(gate_idx) => circuit.get_gate(*gate_idx).input_type(types),
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
