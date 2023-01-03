use crate::circuit;

use super::ty;

#[derive(Clone, Debug)]
pub(crate) enum ProducerBundle {
    Single(circuit::ProducerIdx),
    Product(Vec<(String, ProducerBundle)>),
    InstanceOfNamed(ty::TypeSym, Box<ProducerBundle>),
}
#[derive(Clone, Debug)]
pub(crate) enum ReceiverBundle {
    Single(circuit::ReceiverIdx),
    Product(Vec<(String, ReceiverBundle)>),
    InstanceOfNamed(ty::TypeSym, Box<ReceiverBundle>),
}

impl<'types> ProducerBundle {
    pub(crate) fn type_(&self, types: &'types mut ty::Types) -> ty::TypeSym {
        match self {
            ProducerBundle::Single(_) => types.intern(ty::Type::Bit),
            ProducerBundle::Product(tys) => {
                let ty = ty::Type::Product(tys.iter().map(|(name, subbundle)| (name.to_string(), subbundle.type_(types))).collect());
                types.intern(ty)
            }
            ProducerBundle::InstanceOfNamed(ty, _) => *ty,
        }
    }
}
impl<'types> ReceiverBundle {
    pub(crate) fn type_(&self, types: &'types mut ty::Types) -> ty::TypeSym {
        match self {
            ReceiverBundle::Single(_) => types.intern(ty::Type::Bit),
            ReceiverBundle::Product(tys) => {
                let ty = ty::Type::Product(tys.iter().map(|(name, subbundle)| (name.to_string(), subbundle.type_(types))).collect());
                types.intern(ty)
            }
            ReceiverBundle::InstanceOfNamed(ty, _) => *ty,
        }
    }
}

pub(crate) fn make_receiver_bundle(types: &ty::Types, type_: ty::TypeSym, inputs: &mut impl Iterator<Item = circuit::ReceiverIdx>) -> ReceiverBundle {
    match types.get(type_) {
        ty::Type::Bit => ReceiverBundle::Single(inputs.next().expect("inputs should not run out when converting to bundle")),
        ty::Type::Product(tys) => ReceiverBundle::Product(tys.iter().map(|(name, ty)| (name.clone(), make_receiver_bundle(types, *ty, inputs))).collect()),
        ty::Type::Named(subst_type) => ReceiverBundle::InstanceOfNamed(type_, Box::new(make_receiver_bundle(types, types.get_named(*subst_type).1, inputs))),
    }
}

pub(crate) fn make_producer_bundle(types: &ty::Types, type_: ty::TypeSym, outputs: &mut impl Iterator<Item = circuit::ProducerIdx>) -> ProducerBundle {
    match types.get(type_) {
        ty::Type::Bit => ProducerBundle::Single(outputs.next().expect("outputs should not run out when converting to bundle")),
        ty::Type::Product(tys) => ProducerBundle::Product(tys.iter().map(|(name, ty)| (name.clone(), make_producer_bundle(types, *ty, outputs))).collect()),
        ty::Type::Named(subst_type) => ProducerBundle::InstanceOfNamed(type_, Box::new(make_producer_bundle(types, types.get_named(*subst_type).1, outputs))),
    }
}
