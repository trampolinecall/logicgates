use crate::{
    circuit,
    compiler::{error::Report, parser::ast},
};

use super::Error;

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
    pub(super) fn type_(&self) -> ast::Type {
        // TODO: should not use the type ast as a representation for types
        match self {
            ProducerBundle::Single(_) => ast::Type::Bit,
            ProducerBundle::Product(tys) => ast::Type::Product(tys.iter().map(ProducerBundle::type_).collect()),
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
    pub(super) fn type_(&self) -> ast::Type {
        match self {
            ReceiverBundle::Single(_) => ast::Type::Bit,
            ReceiverBundle::Product(tys) => ast::Type::Product(tys.iter().map(ReceiverBundle::type_).collect()),
        }
    }

    pub(super) fn flatten(&self) -> Vec<circuit::ReceiverIdx> {
        match self {
            ReceiverBundle::Single(i) => vec![*i],
            ReceiverBundle::Product(items) => items.iter().flat_map(ReceiverBundle::flatten).collect(),
        }
    }
}

// TODO: refactor
pub(super) fn make_receiver_bundle(type_: &ast::Type, inputs: &mut impl Iterator<Item = circuit::ReceiverIdx>) -> ReceiverBundle {
    match type_ {
        ast::Type::Bit => ReceiverBundle::Single(inputs.next().expect("inputs should not run out when converting to bundle")),
        ast::Type::Product(tys) => ReceiverBundle::Product(tys.iter().map(|ty| make_receiver_bundle(ty, inputs)).collect()),
    }
}

pub(super) fn make_producer_bundle(type_: &ast::Type, outputs: &mut impl Iterator<Item = circuit::ProducerIdx>) -> ProducerBundle {
    match type_ {
        ast::Type::Bit => ProducerBundle::Single(outputs.next().expect("outputs should not run out when converting to bundle")),
        ast::Type::Product(tys) => ProducerBundle::Product(tys.iter().map(|ty| make_producer_bundle(ty, outputs)).collect()),
    }
}

pub(super) fn connect_bundle(circuit: &mut circuit::Circuit, expr: &ast::Expr, producer_bundle: &ProducerBundle, receiver_bundle: &ReceiverBundle) -> Option<()> {
    if producer_bundle.type_() != receiver_bundle.type_() {
        Error::TypeMismatchInCall { expr, actual_type: producer_bundle.type_(), expected_type: receiver_bundle.type_() }.report();
        None?
    }

    match (producer_bundle, receiver_bundle) {
        (ProducerBundle::Single(producer_index), ReceiverBundle::Single(receiver_index)) => circuit.connect(*producer_index, *receiver_index),
        (ProducerBundle::Product(producers), ReceiverBundle::Product(receivers)) => {
            assert_eq!(producers.len(), receivers.len(), "cannot connect different amount of producers and receivers"); // sanity check
            for (p, r) in producers.iter().zip(receivers.iter()) {
                connect_bundle(circuit, expr, p, r); // not ideal that this rechecks the item types but
            }
        }

        _ => unreachable!("connect two bundles with different types"),
    }

    Some(())
}
