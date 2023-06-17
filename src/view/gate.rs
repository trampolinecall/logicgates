use std::marker::PhantomData;

use nannou::prelude::*;

use crate::{simulation::GateKey, view::Drawing, LogicGates};

pub(crate) struct GateDrawing {
    pub(crate) key: GateKey,
    pub(crate) rect: nannou::geom::Rect,
}

const GATE_COLOR: Rgb = Rgb { red: 0.584, green: 0.647, blue: 0.65, standard: PhantomData };

impl Drawing for GateDrawing {
    fn draw(&self, simulation: &LogicGates, draw: &nannou::Draw, hovered: Option<&dyn Drawing>) {
        if let Some(hovered) = hovered {
            if std::ptr::eq(hovered, self) {
                // TODO: fix clippy lint about this
                let hover_rect = self.rect.pad_left(-5.0).pad_top(-5.0).pad_right(-5.0).pad_bottom(-5.0); // expand by 5, this is the "stroke weight"
                draw.rect().xy(hover_rect.xy()).wh(hover_rect.wh()).color(Rgba { color: Rgb::from_components((1.0, 1.0, 1.0)), alpha: 0.2 });
                // TODO: use constant for stoke weight, hover color
            }
        }

        draw.rect().xy(self.rect.xy()).wh(self.rect.wh()).color(GATE_COLOR);

        draw.text(simulation.simulation.gates[self.key].name(&simulation.simulation.circuits)).xy(self.rect.xy()).wh(self.rect.wh()).center_justify().align_text_middle_y();
    }

    fn find_hover(&self, mouse_pos: nannou::geom::Vec2) -> Option<&dyn Drawing> {
        if self.rect.contains(mouse_pos) {
            // TODO: hover distance
            return Some(self);
        }
        None
    }

    fn left_mouse_down(&self) -> Option<crate::Message> {
        Some(crate::Message::MouseDownOnGate(self.key))
    }
}
