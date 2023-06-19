use crate::{simulation::GateKey, ui::widgets::WidgetId};

#[derive(Copy, Clone)]
pub(crate) struct TargetedUIMessage {
    pub(crate) target: WidgetId,
    pub(crate) message: UIMessage,
}
#[derive(Copy, Clone)]
pub(crate) enum UIMessage {
    MouseDownOnGate(GateKey),
    MouseMoved(nannou::geom::Vec2),
    LeftMouseUp,
}
