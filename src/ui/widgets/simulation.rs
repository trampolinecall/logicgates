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
}

impl SimulationWidget {
    pub(crate) fn new(id_maker: &mut WidgetIdMaker) -> Self {
        Self { cur_gate_drag: None, id: id_maker.next_id() }
    }
}

impl Widget for SimulationWidget {
    type Drawing = drawing::SimulationDrawing;

    fn id(&self) -> WidgetId {
        self.id
    }

    fn view(&self, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> (drawing::SimulationDrawing, Vec<view::Subscription>) {
        drawing::SimulationDrawing::new(&logic_gates.simulation, self, rect)
    }

    fn targeted_message(&mut self, message: TargetedUIMessage) -> Option<Message> {
        if message.target == self.id() {
            self.message(message.message)
        } else {
            None
        }
    }

    fn message(&mut self, message: UIMessage) -> Option<Message> {
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
        }
    }
}
