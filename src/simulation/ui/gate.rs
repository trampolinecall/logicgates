use std::marker::PhantomData;

use nannou::prelude::*;

use crate::simulation::{ui::Widget, Simulation};

pub(crate) struct GateWidget {
}

const GATE_COLOR: Rgb = Rgb { red: 0.584, green: 0.647, blue: 0.65, standard: PhantomData };

impl GateWidget {
    pub(crate) fn new() -> GateWidget {
        GateWidget {}
    }
}

impl Widget for GateWidget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, rect: nannou::geom::Rect) {
        draw.rect().color(GATE_COLOR).xy(rect.xy()).wh(rect.wh());
        // draw.text(simulation.gates[self.gate].name(&simulation.circuits)).xy(rect.xy()).wh(rect.wh()).center_justify().align_text_middle_y(); TODO
    }
}
