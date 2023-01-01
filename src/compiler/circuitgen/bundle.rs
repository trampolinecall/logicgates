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

impl ProducerBundle {
    pub(super) fn type_(&self) -> ty::Type {
        match self {
            ProducerBundle::Single(_) => ty::Type::Bit,
            ProducerBundle::Product(tys) => ty::Type::Product(tys.iter().map(ProducerBundle::type_).collect()),
        }
    }

    pub(super) fn flatten(&self) -> Vec<circuit::ProducerIdx> {
        match self {
            ProducerBundle::Single(i) => vec![*i],
            ProducerBundle::Product(items) => items.iter().flat_map(ProducerBundle::flatten).collect(),
        }
    }
}
impl ReceiverBundle {
    pub(super) fn type_(&self) -> ty::Type {
        match self {
            ReceiverBundle::Single(_) => ty::Type::Bit,
            ReceiverBundle::Product(tys) => ty::Type::Product(tys.iter().map(ReceiverBundle::type_).collect()),
        }
    }

    pub(super) fn flatten(&self) -> Vec<circuit::ReceiverIdx> {
        match self {
            ReceiverBundle::Single(i) => vec![*i],
            ReceiverBundle::Product(items) => items.iter().flat_map(ReceiverBundle::flatten).collect(),
        }
    }
}

pub(super) fn make_receiver_bundle(type_: &ty::Type, inputs: &mut impl Iterator<Item = circuit::ReceiverIdx>) -> ReceiverBundle {
    match type_ {
        ty::Type::Bit => ReceiverBundle::Single(inputs.next().expect("inputs should not run out when converting to bundle")),
        ty::Type::Product(tys) => ReceiverBundle::Product(tys.iter().map(|ty| make_receiver_bundle(ty, inputs)).collect()),
    }
}

pub(super) fn make_producer_bundle(type_: &ty::Type, outputs: &mut impl Iterator<Item = circuit::ProducerIdx>) -> ProducerBundle {
    match type_ {
        ty::Type::Bit => ProducerBundle::Single(outputs.next().expect("outputs should not run out when converting to bundle")),
        ty::Type::Product(tys) => ProducerBundle::Product(tys.iter().map(|ty| make_producer_bundle(ty, outputs)).collect()),
    }
}

pub(super) fn connect_bundle(circuit: &mut circuit::Circuit, expr_span: Span, producer_bundle: &ProducerBundle, receiver_bundle: &ReceiverBundle) -> Option<()> {
    if producer_bundle.type_() != receiver_bundle.type_() {
        Error::TypeMismatchInCall { expr_span, actual_type: producer_bundle.type_(), expected_type: receiver_bundle.type_() }.report();
        None?
    }

    match (producer_bundle, receiver_bundle) {
        (ProducerBundle::Single(producer_index), ReceiverBundle::Single(receiver_index)) => circuit.connect(*producer_index, *receiver_index),
        (ProducerBundle::Product(producers), ReceiverBundle::Product(receivers)) => {
            assert_eq!(producers.len(), receivers.len(), "cannot connect different amount of producers and receivers"); // sanity check
            for (p, r) in producers.iter().zip(receivers.iter()) {
                connect_bundle(circuit, expr_span, p, r); // not ideal that this rechecks the item types but
            }
        }

        _ => unreachable!("connect two bundles with different types"),
    }

    Some(())
}
