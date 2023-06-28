use std::collections::HashMap;

// TODO: remove dependency on draw system
use crate::simulation::{CircuitMap, GateKey, GateMap, Simulation};

pub(crate) struct GateLocation {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl From<(f32, f32)> for GateLocation {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}
