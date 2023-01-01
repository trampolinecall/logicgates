use crate::{circuit, compiler::{parser::ast, error::Report}};

use super::Error;

#[derive(Clone)]
pub(super) enum ProducerBundle {
    Single(circuit::ProducerIdx),
    // List(Vec<ProducerBundle>),
}
pub(super) enum ReceiverBundle {
    Single(circuit::ReceiverIdx),
}

impl ProducerBundle {
    pub(super) fn size(&self) -> usize {
        match self {
            ProducerBundle::Single(_) => 1,
            // ProducerBundle::List(subbundles) => subbundles.iter().map(ProducerBundle::size).sum::<usize>(),
        }
    }
    pub(super) fn type_(&self) -> ast::Type {
        match self {
            ProducerBundle::Single(_) => ast::Type::Bit,
            // ProducerBundle::List(_) => todo!(),
        }
    }

    pub(super) fn flatten(&self) -> Vec<circuit::ProducerIdx> {
        match self {
            ProducerBundle::Single(i) => vec![*i],
            // ProducerBundle::List(subbundles) => subbundles.iter().flat_map(ProducerBundle::flatten).collect(),
        }
    }
}
impl ReceiverBundle {
    pub(super) fn type_(&self) -> ast::Type {
        match self {
            ReceiverBundle::Single(_) => ast::Type::Bit,
        }
    }

    pub(super) fn flatten(&self) -> Vec<circuit::ReceiverIdx> {
        match self {
            ReceiverBundle::Single(i) => vec![*i],
        }
    }
}

// TODO: refactor
pub(super) fn make_receiver_bundles(types: &[ast::Type], mut inputs: impl Iterator<Item = circuit::ReceiverIdx>) -> Vec<ReceiverBundle> {
    let mut bundles = Vec::new();
    for input_type in types {
        bundles.push(make_receiver_bundle(input_type, &mut inputs))
    }

    bundles
}
pub(super) fn make_producer_bundles(types: &[ast::Type], mut outputs: impl Iterator<Item = circuit::ProducerIdx>) -> Vec<ProducerBundle> {
    let mut bundles = Vec::new();
    for output_type in types {
        bundles.push(make_producer_bundle(output_type, &mut outputs))
    }

    bundles
}

pub(super) fn make_receiver_bundle(type_: &ast::Type, mut inputs: impl Iterator<Item = circuit::ReceiverIdx>) -> ReceiverBundle {
    match type_ {
        ast::Type::Bit => ReceiverBundle::Single(inputs.next().expect("inputs should not run out when converting to bundle")),
    }
}

pub(super) fn make_producer_bundle(type_: &ast::Type, mut outputs: impl Iterator<Item = circuit::ProducerIdx>) -> ProducerBundle {
    match type_ {
        ast::Type::Bit => ProducerBundle::Single(outputs.next().expect("outputs should not run out when converting to bundle")),
    }
}

pub(super) fn connect_bundle(circuit: &mut circuit::Circuit, producer_bundle: &ProducerBundle, receiver_bundle: &ReceiverBundle) -> Option<()> {
    if producer_bundle.type_() != receiver_bundle.type_() {
        Error::TypeMismatchInCall { actual_type: producer_bundle.type_(), expected_type: receiver_bundle.type_() }.report();
        None?
    }

    match (producer_bundle, receiver_bundle) {
        (ProducerBundle::Single(producer_index), ReceiverBundle::Single(receiver_index)) => circuit.connect(*producer_index, *receiver_index),
    }

    Some(())
}
