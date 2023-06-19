use crate::{ui::{
    message::{TargetedUIMessage, UIMessage},
    widgets::{Widget, WidgetId, WidgetIdMaker},
}, view};

pub(crate) struct SlideOver<Base: Widget, Over: Widget> {
    id: WidgetId,
    base: Base,
    over: Over,
}

impl<Base: Widget, Over: Widget> SlideOver<Base, Over> {
    pub(crate) fn new(id_maker: &mut WidgetIdMaker, base: Base, over: Over) -> Self {
        Self { id: id_maker.next_id(), base, over }
    }
}

impl<Base: Widget, Over: Widget> Widget for SlideOver<Base, Over> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn view(&self, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>) {
        todo!()
    }

    fn targeted_message(&mut self, targeted_message: TargetedUIMessage) -> Option<crate::Message> {
        if targeted_message.target == self.id {
            self.message(targeted_message.message)
        } else if let Some(base_response) = self.base.targeted_message(targeted_message) {
            Some(base_response)
        } else if let Some(over_response) = self.over.targeted_message(targeted_message) {
            Some(over_response)
        } else {
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
