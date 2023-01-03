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
    pub(crate) fn type_(&self, types: &'types mut ty::Types, circuit: &super::CustomCircuit) -> ty::TypeSym {
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
    pub(crate) fn type_(&self, types: &'types mut ty::Types, circuit: &super::CustomCircuit) -> ty::TypeSym {
        match self {
            ReceiverBundle::CurCircuitOutput => circuit.result_type,
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

/*
pub(crate) fn make_receiver_bundle(types: &ty::Types, type_: ty::TypeSym, inputs: &mut impl Iterator<Item = circuit::ReceiverIdx>) -> ReceiverBundle {
    /*
    match types.get(type_) {
        ty::Type::Bit => ReceiverBundle::Single(inputs.next().expect("inputs should not run out when converting to bundle")),
        ty::Type::Product(tys) => ReceiverBundle::Product(tys.iter().map(|(name, ty)| (name.clone(), make_receiver_bundle(types, *ty, inputs))).collect()),
        ty::Type::Named(subst_type) => ReceiverBundle::InstanceOfNamed(type_, Box::new(make_receiver_bundle(types, types.get_named(*subst_type).1, inputs))),
    }
    */
    todo!()
}

pub(crate) fn make_producer_bundle(types: &ty::Types, type_: ty::TypeSym, outputs: &mut impl Iterator<Item = circuit::ProducerIdx>) -> ProducerBundle {
    /*
    match types.get(type_) {
        ty::Type::Bit => ProducerBundle::Single(outputs.next().expect("outputs should not run out when converting to bundle")),
        ty::Type::Product(tys) => ProducerBundle::Product(tys.iter().map(|(name, ty)| (name.clone(), make_producer_bundle(types, *ty, outputs))).collect()),
        ty::Type::Named(subst_type) => ProducerBundle::InstanceOfNamed(type_, Box::new(make_producer_bundle(types, types.get_named(*subst_type).1, outputs))),
    }
    */
    todo!()
}
TODO
*/
