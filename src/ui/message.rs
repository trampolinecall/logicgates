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
    MouseDownOnSlideOverToggleButton,
    MouseDownOnSlider(nannou::geom::Vec2, f32), // TODO: figure out a way to remove the f32 from this
}
