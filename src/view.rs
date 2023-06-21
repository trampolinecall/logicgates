pub(crate) mod id;
pub(crate) mod layout_cache;
pub(crate) mod lens;

use nannou::prelude::*;

#[derive(Copy, Clone)]
pub(crate) enum TargetedEvent {
    LeftMouseDown,
}
#[derive(Copy, Clone)]
pub(crate) enum GeneralEvent {
    MouseMoved(Vec2),
    LeftMouseUp,
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) struct SizeConstraints {
    pub(crate) min: Vec2,
    pub(crate) max: Vec2,
}
impl SizeConstraints {
    pub(crate) fn with_no_min(&self) -> SizeConstraints {
        SizeConstraints { min: Vec2::ZERO, max: self.max }
    }
}

// new view system heavilty inspired by xilem
// specifically this blog post: https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html
// kind of like a merge of the old Widget and old Drawing trait
pub(crate) trait View<Data> {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, center: nannou::geom::Vec2, size_constraints: SizeConstraints, hover: Option<id::ViewId>);
    fn find_hover(&self, center: nannou::geom::Vec2, size_constraints: SizeConstraints, mouse: nannou::geom::Vec2) -> Option<id::ViewId>;
    fn size(&self, size_constraints: SizeConstraints) -> Vec2;

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: id::ViewId, event: TargetedEvent);
    fn targeted_event(&self, app: &nannou::App, data: &mut Data, event: TargetedEvent);
    fn general_event(&self, app: &nannou::App, data: &mut Data, event: GeneralEvent);
}
// TODO: make sure that draws to inner views do not draw outside of their given boxes

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, logic_gates: &crate::LogicGates) {
    let view = crate::view(app, logic_gates);

    let view_center = Vec2::ZERO;
    let size_constraints = SizeConstraints { min: Vec2::ZERO, max: app.window_rect().wh() };

    let hover = view.find_hover(view_center, size_constraints, app.mouse.position());
    view.draw(app, draw, view_center, size_constraints, hover);
}

pub(crate) fn event(app: &nannou::App, logic_gates: &mut crate::LogicGates, event: nannou::Event) {
    let view = crate::view(app, logic_gates);

    let view_center = Vec2::ZERO;
    let size_constraints = SizeConstraints { min: Vec2::ZERO, max: app.window_rect().wh() };

    if let nannou::Event::WindowEvent { id: _, simple: Some(event) } = event {
        match event {
            WindowEvent::MousePressed(MouseButton::Left) => {
                let hovered = view.find_hover(view_center, size_constraints, app.mouse.position());
                if let Some(hovered) = hovered {
                    view.send_targeted_event(app, logic_gates, hovered, TargetedEvent::LeftMouseDown);
                }
            }

            WindowEvent::MouseMoved(mouse_pos) => view.general_event(app, logic_gates, GeneralEvent::MouseMoved(mouse_pos)),

            WindowEvent::MouseReleased(MouseButton::Left) => view.general_event(app, logic_gates, GeneralEvent::LeftMouseUp),

            _ => {}
        }
    }
}
