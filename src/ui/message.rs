use crate::{simulation::GateKey, ui::widgets::WidgetId};

pub(crate) struct TargetedUIMessage {
    pub(crate) target: WidgetId,
    pub(crate) message: UIMessage,
}
pub(crate) enum UIMessage {
    MouseDownOnGate(GateKey),
    MouseMoved(nannou::geom::Vec2),
    LeftMouseUp
}
