use std::marker::PhantomData;

use nannou::prelude::*;

use crate::{
    simulation::{logic, NodeKey, NodeMap, Simulation},
    ui::Widget,
};

pub(crate) struct NodeWidget {
    node_key: NodeKey,
}

impl NodeWidget {
    pub(crate) fn new(node_key: NodeKey) -> NodeWidget {
        NodeWidget { node_key }
    }
}

const ON_COLOR: Rgb = Rgb { red: 0.18, green: 0.8, blue: 0.521, standard: PhantomData };
const OFF_COLOR: Rgb = Rgb { red: 0.498, green: 0.549, blue: 0.552, standard: PhantomData };
const HIGH_IMPEDANCE_COLOR: Rgb = Rgb { red: 52.0 / 255.0, green: 152.0 / 255.0, blue: 219.0 / 255.0, standard: PhantomData };
const ERR_COLOR: Rgb = Rgb { red: 231.0 / 255.0, green: 76.0 / 255.0, blue: 60.0 / 255.0, standard: PhantomData };

impl Widget for NodeWidget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, rect: nannou::geom::Rect) {
        let color = node_color(&simulation.nodes, self.node_key, true);
        draw.ellipse().color(color).xy(rect.xy()).radius(rect.w() / 2.0);
    }
}

fn node_color(nodes: &NodeMap, node: NodeKey, use_production: bool) -> Rgb {
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
