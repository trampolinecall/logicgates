use generational_arena::Arena;

pub(crate) mod circuit;
pub(crate) mod position;
pub(crate) mod connections;
pub(crate) mod draw;

// TODO: clean up everything in here, for example some places use indexes and some use direct references, things like that, ...

pub(crate) struct Simulation {
    pub(crate) circuits: Arena<circuit::Circuit>,
    pub(crate) gates: Arena<circuit::Gate>,

    pub(crate) main_circuit: circuit::CircuitIndex,
}
