mod drawing;

use crate::{simulation, ui::widgets::Widget};

pub(crate) struct SimulationWidget {
    cur_gate_drag: Option<simulation::GateKey>,
}

impl SimulationWidget {
    pub(crate) fn new() -> Self {
        Self { cur_gate_drag: None }
    }
}

impl Widget for SimulationWidget {
    type Drawing = drawing::SimulationDrawing;

    fn view(&self, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> drawing::SimulationDrawing {
        drawing::SimulationDrawing::new(rect, &logic_gates.simulation)
    }
}

