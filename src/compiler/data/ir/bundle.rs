use crate::compiler::data::{ir::GateIdx, nominal_type, ty};

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
}

impl ProducerBundle {
    pub(crate) fn type_(&self, type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>) -> ty::TypeSym {
        match self {
            ProducerBundle::CurCircuitInput(ty) | ProducerBundle::GateOutput(ty, _) => *ty,
            ProducerBundle::Get(producer, field) => {
                let producer_type = producer.type_(type_context);
                ty::Type::get_field_type(&type_context.get(producer_type).fields(type_context), field).unwrap()
            }
            ProducerBundle::Product(tys) => {
                let ty = ty::Type::Product(tys.iter().map(|(name, subbundle)| (name.to_string(), subbundle.type_(type_context))).collect());
                type_context.intern(ty)
            }
        }
    }
}
impl ReceiverBundle {
    pub(crate) fn type_(&self, _: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>) -> ty::TypeSym {
        // keep unused parameters for symmetry with ProducerBundle::type_
        match self {
            ReceiverBundle::CurCircuitOutput(ty) | ReceiverBundle::GateInput(ty, _) => *ty,
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
