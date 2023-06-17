use std::marker::PhantomData;

use nannou::prelude::*;

use crate::{
    simulation::{logic, NodeKey, NodeMap},
    view::Drawing,
    LogicGates,
};

pub(crate) struct NodeDrawing {
    pub(crate) key: NodeKey,
    pub(crate) location: nannou::geom::Vec2,
}

// TODO: refactor so that this doesnt need this to be pub(crate)
pub(crate) const CIRCLE_RAD: f32 = 5.0;

const ON_COLOR: Rgb = Rgb { red: 0.18, green: 0.8, blue: 0.521, standard: PhantomData };
const OFF_COLOR: Rgb = Rgb { red: 0.498, green: 0.549, blue: 0.552, standard: PhantomData };
const HIGH_IMPEDANCE_COLOR: Rgb = Rgb { red: 52.0 / 255.0, green: 152.0 / 255.0, blue: 219.0 / 255.0, standard: PhantomData };
const ERR_COLOR: Rgb = Rgb { red: 231.0 / 255.0, green: 76.0 / 255.0, blue: 60.0 / 255.0, standard: PhantomData };

impl Drawing for NodeDrawing {
    fn draw(&self, simulation: &LogicGates, draw: &nannou::Draw, hovered: Option<&dyn Drawing>) {
        if let Some(hovered) = hovered {
            if std::ptr::eq(hovered, self) {
                // TODO: fix clippy lint about this
                draw.ellipse().xy(self.location).radius(CIRCLE_RAD + 5.0).color(Rgba { color: Rgb::from_components((1.0, 1.0, 1.0)), alpha: 0.2 });
            }
        }

        let color = node_color(&simulation.simulation.nodes, self.key, true);
        draw.ellipse().xy(self.location).radius(CIRCLE_RAD).color(color);
    }

    fn find_hover(&self, mouse_pos: nannou::geom::Vec2) -> Option<&dyn Drawing> {
        if self.location.distance(mouse_pos) < CIRCLE_RAD + 2.0 {
            // TODO: move 2 to a constant "HOVER_DISTANCE"
            return Some(self);
        }
        None
    }
}

// TODO: also refactor so that this does not need to be pub(crate)
pub(crate) fn node_color(nodes: &NodeMap, node: NodeKey, use_production: bool) -> Rgb {
    fn value_to_color(v: logic::Value) -> Rgb {
        match v {
            logic::Value::H => ON_COLOR,
            logic::Value::L => OFF_COLOR,
            logic::Value::Z => HIGH_IMPEDANCE_COLOR,
            logic::Value::X => ERR_COLOR,
        }
    }
    if use_production {
        if let Some(v) = logic::get_node_production(nodes, node) {
            value_to_color(v)
        } else {
            value_to_color(logic::get_node_value(nodes, node))
        }
    } else {
        value_to_color(logic::get_node_value(nodes, node))
    }
}
