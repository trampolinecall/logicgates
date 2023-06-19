mod drawing;

use crate::{
    simulation,
    ui::{
        message::{TargetedUIMessage, UIMessage},
        widgets::{Widget, WidgetId, WidgetIdMaker},
    },
    view, Message,
};

pub(crate) struct SimulationWidget {
    id: WidgetId,
    cur_gate_drag: Option<simulation::GateKey>,
    view: Option<simulation::CircuitKey>,
}

impl SimulationWidget {
    pub(crate) fn new(id_maker: &mut WidgetIdMaker) -> Self {
        Self { cur_gate_drag: None, id: id_maker.next_id(), view: None }
    }
}

impl Widget for SimulationWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn size(&self, given: (f32, f32)) -> (f32, f32) {
        given // always fills given space
    }

    fn view(&self, _: &nannou::App, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>) {
        // TODO: show currently viewing at top of widget
        let gates_viewing = match self.view {
            Some(ck) => &logic_gates.simulation.circuits[ck].gates,
            None => &logic_gates.simulation.toplevel_gates,
        };

        let gates = gates_viewing.iter().copied();
        let nodes = gates_viewing
            .iter()
            .flat_map(|gate| {
                simulation::Gate::inputs(&logic_gates.simulation.circuits, &logic_gates.simulation.gates, *gate).iter().chain(simulation::Gate::outputs(
                    &logic_gates.simulation.circuits,
                    &logic_gates.simulation.gates,
                    *gate,
                ))
            })
            .copied();

        let (gate_drawings, node_drawings, connection_drawings) = drawing::layout(
            &logic_gates.simulation.circuits,
            &logic_gates.simulation.gates,
            &logic_gates.simulation.nodes,
            &logic_gates.simulation.connections,
            self.id,
            self.view,
            gates,
            nodes,
            rect,
        );

        let subscriptions = if self.cur_gate_drag.is_some() {
            vec![
                view::Subscription::MouseMoved({
                    let swid_id = self.id;
                    Box::new(move |_, mouse_pos| TargetedUIMessage { target: swid_id, message: UIMessage::MouseMoved(mouse_pos) })
                }),
                view::Subscription::LeftMouseUp({
                    let swid_id = self.id;
                    Box::new(move |_| TargetedUIMessage { target: swid_id, message: UIMessage::LeftMouseUp })
                }),
            ]
        } else {
            Vec::new()
        };

        (Box::new(drawing::SimulationDrawing { gates: gate_drawings, nodes: node_drawings, connections: connection_drawings, rect }), subscriptions)
    }

    fn targeted_message(&mut self, app: &nannou::App, message: TargetedUIMessage) -> Option<Message> {
        if message.target == self.id() {
            self.message(app, message.message)
        } else {
            None
        }
    }

    fn message(&mut self, _: &nannou::App, message: UIMessage) -> Option<Message> {
        match message {
            UIMessage::MouseDownOnGate(gate_key) => {
                self.cur_gate_drag = Some(gate_key);
                None
            }
            UIMessage::MouseMoved(mouse_pos) => {
                // TODO: zooming and panning
                self.cur_gate_drag.map(|cur_gate_drag| Message::GateMoved(cur_gate_drag, mouse_pos))
            }
            UIMessage::LeftMouseUp => {
                self.cur_gate_drag = None;
                None
            }
            UIMessage::MouseDownOnSlideOverToggleButton => None,
            UIMessage::MouseDownOnSlider(_, _) => None,
        }
    }
}
