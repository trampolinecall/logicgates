use crate::{
    circuit,
    compiler::{error::Report, parser::ast},
};

use super::Error;

#[derive(Clone)]
pub(super) enum ProducerBundle {
    Single(circuit::ProducerIdx),
    Array(Vec<ProducerBundle>),
}
pub(super) enum ReceiverBundle {
    Single(circuit::ReceiverIdx),
    Array(Vec<ReceiverBundle>),
}

impl ProducerBundle {
    pub(super) fn type_(&self) -> ast::Type {
        // TODO: should not use the type ast as a representation for types
        match self {
            ProducerBundle::Single(_) => ast::Type::Bit,
            ProducerBundle::Array(items) => ast::Type::Array(items.len(), Box::new(items[0].type_())),
        }
    }

    pub(super) fn flatten(&self) -> Vec<circuit::ProducerIdx> {
        match self {
            ProducerBundle::Single(i) => vec![*i],
            ProducerBundle::Array(items) => items.iter().flat_map(ProducerBundle::flatten).collect(),
        }
    }
}
impl ReceiverBundle {
    pub(super) fn type_(&self) -> ast::Type {
        match self {
            ReceiverBundle::Single(_) => ast::Type::Bit,
            ReceiverBundle::Array(items) => ast::Type::Array(items.len(), Box::new(items[0].type_())),
        }
    }

    pub(super) fn flatten(&self) -> Vec<circuit::ReceiverIdx> {
        match self {
            ReceiverBundle::Single(i) => vec![*i],
            ReceiverBundle::Array(items) => items.iter().flat_map(ReceiverBundle::flatten).collect(),
        }
    }
}

// TODO: refactor
pub(super) fn make_receiver_bundles(types: &[ast::Type], mut inputs: &mut impl Iterator<Item = circuit::ReceiverIdx>) -> Vec<ReceiverBundle> {
    let mut bundles = Vec::new();
    for input_type in types {
        bundles.push(make_receiver_bundle(input_type, &mut inputs))
    }

    bundles
}
pub(super) fn make_producer_bundles(types: &[ast::Type], mut outputs: &mut impl Iterator<Item = circuit::ProducerIdx>) -> Vec<ProducerBundle> {
    let mut bundles = Vec::new();
    for output_type in types {
        bundles.push(make_producer_bundle(output_type, &mut outputs))
    }

    bundles
}

pub(super) fn make_receiver_bundle(type_: &ast::Type, inputs: &mut impl Iterator<Item = circuit::ReceiverIdx>) -> ReceiverBundle {
    match type_ {
        ast::Type::Bit => ReceiverBundle::Single(inputs.next().expect("inputs should not run out when converting to bundle")),
        ast::Type::Array(len, item_type) => ReceiverBundle::Array((0..*len).map(|_| make_receiver_bundle(item_type, inputs)).collect()),
    }
}

pub(super) fn make_producer_bundle(type_: &ast::Type, outputs: &mut impl Iterator<Item = circuit::ProducerIdx>) -> ProducerBundle {
    match type_ {
        ast::Type::Bit => ProducerBundle::Single(outputs.next().expect("outputs should not run out when converting to bundle")),
        ast::Type::Array(len, item_type) => ProducerBundle::Array((0..*len).map(|_| make_producer_bundle(item_type, outputs)).collect()),
    }
}

pub(super) fn connect_bundle(circuit: &mut circuit::Circuit, producer_bundle: &ProducerBundle, receiver_bundle: &ReceiverBundle) -> Option<()> {
    if producer_bundle.type_() != receiver_bundle.type_() {
        Error::TypeMismatchInCall { actual_type: producer_bundle.type_(), expected_type: receiver_bundle.type_() }.report();
        None?
    }

    match (producer_bundle, receiver_bundle) {
        (ProducerBundle::Single(producer_index), ReceiverBundle::Single(receiver_index)) => circuit.connect(*producer_index, *receiver_index),
        (ProducerBundle::Array(producers), ReceiverBundle::Array(receivers)) => {
            assert_eq!(producers.len(), receivers.len(), "cannot connect different amount of producers and receivers"); // sanity check
            for (p, r) in producers.iter().zip(receivers.iter()) {
                connect_bundle(circuit, p, r);
            }
        }

        _ => unreachable!("connect two bundles with different types"),
    }

    Some(())
}
