use crate::{
    circuit,
    compiler::error::{Report, Span},
};

use super::{ty, Error};

#[derive(Clone, Debug)]
pub(super) enum ProducerBundle {
    Single(circuit::ProducerIdx),
    Product(Vec<ProducerBundle>),
}
#[derive(Clone, Debug)]
pub(super) enum ReceiverBundle {
    Single(circuit::ReceiverIdx),
    Product(Vec<ReceiverBundle>),
}

impl<'types> ProducerBundle {
    pub(super) fn type_(&self, types: &'types mut ty::Types) -> ty::TypeSym {
        match self {
            ProducerBundle::Single(_) => types.intern(ty::Type::Bit),
            ProducerBundle::Product(tys) => {
                let ty = ty::Type::Product(tys.iter().map(|subbundle| subbundle.type_(types)).collect());
                types.intern(ty)
            }
        }
    }

    pub(super) fn flatten(&self) -> Vec<circuit::ProducerIdx> {
        match self {
            ProducerBundle::Single(i) => vec![*i],
            ProducerBundle::Product(items) => items.iter().flat_map(ProducerBundle::flatten).collect(),
        }
    }
}
impl<'types> ReceiverBundle {
    pub(super) fn type_(&self, types: &'types mut ty::Types) -> ty::TypeSym {
        match self {
            ReceiverBundle::Single(_) => types.intern(ty::Type::Bit),
            ReceiverBundle::Product(tys) => {
                let ty = ty::Type::Product(tys.iter().map(|subbundle| subbundle.type_(types)).collect());
                types.intern(ty)
            }
        }
    }

    pub(super) fn flatten(&self) -> Vec<circuit::ReceiverIdx> {
        match self {
            ReceiverBundle::Single(i) => vec![*i],
            ReceiverBundle::Product(items) => items.iter().flat_map(ReceiverBundle::flatten).collect(),
        }
    }
}

pub(super) fn make_receiver_bundle(types: &ty::Types, type_: ty::TypeSym, inputs: &mut impl Iterator<Item = circuit::ReceiverIdx>) -> ReceiverBundle {
    match types.get(type_) {
        ty::Type::Bit => ReceiverBundle::Single(inputs.next().expect("inputs should not run out when converting to bundle")),
        ty::Type::Product(tys) => ReceiverBundle::Product(tys.iter().map(|ty| make_receiver_bundle(types, *ty, inputs)).collect()),
    }
}

pub(super) fn make_producer_bundle(types: &ty::Types, type_: ty::TypeSym, outputs: &mut impl Iterator<Item = circuit::ProducerIdx>) -> ProducerBundle {
    match types.get(type_) {
        ty::Type::Bit => ProducerBundle::Single(outputs.next().expect("outputs should not run out when converting to bundle")),
        ty::Type::Product(tys) => ProducerBundle::Product(tys.iter().map(|ty| make_producer_bundle(types, *ty, outputs)).collect()),
    }
}

pub(super) fn connect_bundle<'types>(types: &'types mut ty::Types, circuit: &mut circuit::Circuit, expr_span: Span, producer_bundle: &ProducerBundle, receiver_bundle: &ReceiverBundle) -> Option<()> {
    let producer_type = producer_bundle.type_(types);
    let receiver_type = receiver_bundle.type_(types);
    if producer_type != receiver_type {
        (&*types, Error::TypeMismatchInCall { expr_span, actual_type: producer_type, expected_type: receiver_type }).report();
        None?
    }

    match (producer_bundle, receiver_bundle) {
        (ProducerBundle::Single(producer_index), ReceiverBundle::Single(receiver_index)) => circuit.connect(*producer_index, *receiver_index),
        (ProducerBundle::Product(producers), ReceiverBundle::Product(receivers)) => {
            assert_eq!(producers.len(), receivers.len(), "cannot connect different amount of producers and receivers"); // sanity check
            for (p, r) in producers.iter().zip(receivers.iter()) {
                connect_bundle(types, circuit, expr_span, p, r); // not ideal that this rechecks the item types but
            }
        }

        _ => unreachable!("connect two bundles with different types"),
    }

    Some(())
}
