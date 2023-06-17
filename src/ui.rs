use crate::simulation;

pub(crate) struct UI {
    pub(crate) simulation_widget: SimulationWidget,
}

pub(crate) struct SimulationWidget {
    pub(crate) cur_gate_drag: Option<simulation::GateKey>,
}

impl UI {
    pub(crate) fn new() -> UI {
        UI { simulation_widget: SimulationWidget { cur_gate_drag: None } }
    }
}
