use crate::{circuit, compiler::parser::ast};

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
