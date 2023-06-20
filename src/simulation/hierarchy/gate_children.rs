use crate::simulation::GateKey;

pub(crate) struct GateChildren {
    gates: Vec<GateKey>,
}

impl GateChildren {
    pub(crate) fn new() -> Self {
        Self { gates: Vec::new() }
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<GateKey> {
        self.gates.iter()
    }

    pub(crate) fn add_gate(&mut self, gate: GateKey) {
        self.gates.push(gate);
    }

    pub(crate) fn contains(&self, gk: GateKey) -> bool {
        self.gates.contains(&gk)
    }
}

impl<'a> IntoIterator for &'a GateChildren {
    type Item = &'a GateKey;
    type IntoIter = std::slice::Iter<'a, GateKey>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
