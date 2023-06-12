use std::marker::PhantomData;

use nannou::prelude::*;

use crate::{
    simulation::{GateKey, Simulation},
    ui::Widget,
};

pub(crate) struct GateWidget {
    pub(crate) key: GateKey,
    pub(crate) rect: nannou::geom::Rect,
}

const GATE_COLOR: Rgb = Rgb { red: 0.584, green: 0.647, blue: 0.65, standard: PhantomData };

impl Widget for GateWidget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw) {
        draw.rect().color(GATE_COLOR).xy(self.rect.xy()).wh(self.rect.wh());
        draw.text(simulation.gates[self.key].name(&simulation.circuits)).xy(self.rect.xy()).wh(self.rect.wh()).center_justify().align_text_middle_y();
    }
}
