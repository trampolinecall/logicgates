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
    fn id(&self) -> WidgetId {
        self.id
    }

    fn size(&self, given: (f32, f32)) -> (f32, f32) {
        given // always fills given space
    }

    fn view(&self, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>) {
        let (drawing, subscriptions) = drawing::SimulationDrawing::new(&logic_gates.simulation, self, rect);
        (drawing, subscriptions)
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
