use crate::{
    circuit,
    compiler::error::{Report, Span},
};

use super::{ty, Error};

#[derive(Clone, Debug)]
pub(super) enum ProducerBundle {
    Single(circuit::ProducerIdx),
    Product(Vec<(String, ProducerBundle)>),
    InstanceOfNamed(ty::TypeSym, Box<ProducerBundle>),
}
#[derive(Clone, Debug)]
pub(super) enum ReceiverBundle {
    Single(circuit::ReceiverIdx),
    Product(Vec<(String, ReceiverBundle)>),
    InstanceOfNamed(ty::TypeSym, Box<ReceiverBundle>),
}

impl<'types> ProducerBundle {
    pub(super) fn type_(&self, types: &'types mut ty::Types) -> ty::TypeSym {
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
    pub(super) fn type_(&self, types: &'types mut ty::Types) -> ty::TypeSym {
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

pub(super) fn make_receiver_bundle(types: &ty::Types, type_: ty::TypeSym, inputs: &mut impl Iterator<Item = circuit::ReceiverIdx>) -> ReceiverBundle {
    match types.get(type_) {
        ty::Type::Bit => ReceiverBundle::Single(inputs.next().expect("inputs should not run out when converting to bundle")),
        ty::Type::Product(tys) => ReceiverBundle::Product(tys.iter().map(|(name, ty)| (name.clone(), make_receiver_bundle(types, *ty, inputs))).collect()),
        ty::Type::Named(subst_type) => ReceiverBundle::InstanceOfNamed(type_, Box::new(make_receiver_bundle(types, types.get_named(*subst_type).1, inputs))),
    }
}

pub(super) fn make_producer_bundle(types: &ty::Types, type_: ty::TypeSym, outputs: &mut impl Iterator<Item = circuit::ProducerIdx>) -> ProducerBundle {
    match types.get(type_) {
        ty::Type::Bit => ProducerBundle::Single(outputs.next().expect("outputs should not run out when converting to bundle")),
        ty::Type::Product(tys) => ProducerBundle::Product(tys.iter().map(|(name, ty)| (name.clone(), make_producer_bundle(types, *ty, outputs))).collect()),
        ty::Type::Named(subst_type) => ProducerBundle::InstanceOfNamed(type_, Box::new(make_producer_bundle(types, types.get_named(*subst_type).1, outputs))),
    }
}

pub(super) fn connect_bundle(
    types: &mut ty::Types,
    circuit: &mut circuit::Circuit,
    // got_span: Span,
    expected_span: Span,
    producer_bundle: &ProducerBundle,
    receiver_bundle: &ReceiverBundle,
) -> Option<()> {
    let producer_type = producer_bundle.type_(types);
    let receiver_type = receiver_bundle.type_(types);
    if producer_type != receiver_type {
        (&*types, Error::TypeMismatch { got_type: producer_type, expected_type: receiver_type, /* got_span, */ expected_span }).report();
        None?;
    }

    match (producer_bundle, receiver_bundle) {
        (ProducerBundle::Single(producer_index), ReceiverBundle::Single(receiver_index)) => circuit.connect(*producer_index, *receiver_index),
        (ProducerBundle::Product(producers), ReceiverBundle::Product(receivers)) => {
            assert_eq!(producers.len(), receivers.len(), "cannot connect different amount of producers and receivers"); // sanity check
            for ((_, p), (_, r)) in producers.iter().zip(receivers.iter()) {
                connect_bundle(types, circuit, /* got_span, */ expected_span, p, r);
                // not ideal that this rechecks the item types but
            }
        }

        _ => unreachable!("connect two bundles with different types"),
    }

    Some(())
}
