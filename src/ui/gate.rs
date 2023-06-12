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
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, hovered: Option<&dyn Widget>) {
        let mut rect = draw.rect().xy(self.rect.xy()).wh(self.rect.wh()).color(GATE_COLOR);
        if let Some(hovered) = hovered {
            if std::ptr::eq(hovered, self) {
                // TODO: fix clippy lint about this
                rect = rect.stroke(Rgba { color: Rgb::from_components((1.0, 1.0, 1.0)), alpha: 0.5 }).stroke_weight(5.0);
                // TODO: use constant for stoke weight, hover color
            }
        }
        rect.finish();

        draw.text(simulation.gates[self.key].name(&simulation.circuits)).xy(self.rect.xy()).wh(self.rect.wh()).center_justify().align_text_middle_y();
    }

    fn find_hover(&self, mouse_pos: nannou::geom::Vec2) -> Option<&dyn Widget> {
        if self.rect.contains(mouse_pos) {
            // TODO: hover distance
            return Some(self);
        }
        None
    }
}
