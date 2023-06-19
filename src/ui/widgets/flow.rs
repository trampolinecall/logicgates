use crate::{
    ui::{
        message::{TargetedUIMessage, UIMessage},
        widgets::{Widget, WidgetId, WidgetIdMaker},
    },
    view,
};

pub(crate) struct Flow {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
}

impl Flow {
    pub(crate) fn new(id_maker: &mut WidgetIdMaker, children: Vec<Box<dyn Widget>>) -> Self {
        Self { id: id_maker.next_id(), children }
    }
}

impl Widget for Flow {
    fn id(&self) -> super::WidgetId {
        self.id
    }

    fn view(&self, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>) {
        todo!()
    }

    fn targeted_message(&mut self, targeted_message: TargetedUIMessage) -> Option<crate::Message> {
        if targeted_message.target == self.id {
            self.message(targeted_message.message)
        } else {
            for child in &mut self.children {
                if let Some(child_response) = child.targeted_message(targeted_message) {
                    return Some(child_response);
                }
            }

            None
        }
    }

    fn message(&mut self, message: UIMessage) -> Option<crate::Message> {
        match message {
            UIMessage::MouseDownOnGate(_) => None,
            UIMessage::MouseMoved(_) => None,
            UIMessage::LeftMouseUp => None,
        }
    }
}
