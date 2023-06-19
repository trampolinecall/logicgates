pub(crate) mod flow;
pub(crate) mod simulation;
pub(crate) mod slide_over;

use crate::{
    ui::message::{TargetedUIMessage, UIMessage},
    view, Message,
};

pub(crate) trait Widget {
    fn id(&self) -> WidgetId;
    fn view(&self, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>);
    fn targeted_message(&mut self, targeted_message: TargetedUIMessage) -> Option<Message>;
    fn message(&mut self, message: UIMessage) -> Option<Message>;
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct WidgetId(u64);
pub(crate) struct WidgetIdMaker(u64);
impl WidgetIdMaker {
    pub(crate) fn new() -> WidgetIdMaker {
        WidgetIdMaker(0)
    }
    fn next_id(&mut self) -> WidgetId {
        let id = WidgetId(self.0);
        self.0 += 1;
        id
    }
}
