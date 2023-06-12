use crate::{
    simulation::{Gate, Simulation},
    ui::{gate::GateWidget, node::NodeWidget, simulation::SimulationWidget},
};

mod gate;
mod node;
mod simulation;

// mvc pattern inspired by elm architecture
pub(crate) struct View {
    sim: simulation::SimulationWidget,
}

trait Widget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, rect: nannou::geom::Rect);
}

pub(crate) fn view(simulation: &Simulation) -> View {
    let toplevel_gates = &simulation.toplevel_gates; // TODO: ability to switch between viewing toplevel and circuit
    View {
        sim: SimulationWidget {
            gates: toplevel_gates.iter().map(|gate| GateWidget { key: *gate }).collect(),
            nodes: toplevel_gates
                .iter()
                .flat_map(|gate| Gate::inputs(&simulation.circuits, &simulation.gates, *gate).iter().chain(Gate::outputs(&simulation.circuits, &simulation.gates, *gate)))
                .map(|node| NodeWidget { key: *node })
                .collect(),
        },
    }
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, simulation: &Simulation, view: &View) {
    view.sim.draw(simulation, draw, app.window_rect());
}
