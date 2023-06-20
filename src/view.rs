pub(crate) mod id;
pub(crate) mod lens;
pub(crate) mod layout_cache;

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

// new view system heavilty inspired by xilem
// specifically this blog post: https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html
// kind of like a merge of the old Widget and old Drawing trait
pub(crate) trait View<Data> {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, rect: nannou::geom::Rect, hover: Option<id::ViewId>);
    fn find_hover(&self, rect: nannou::geom::Rect, mouse: nannou::geom::Vec2) -> Option<id::ViewId>;
    fn size(&self, given: (f32, f32)) -> (f32, f32); // TODO: this should eventually take some kind of constraint type instead of just a given size

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: id::ViewId, event: TargetedEvent);
    fn targeted_event(&self, app: &nannou::App, data: &mut Data, event: TargetedEvent);
    fn general_event(&self, app: &nannou::App, data: &mut Data, event: GeneralEvent);
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, logic_gates: &crate::LogicGates) {
    let view = crate::view(app, logic_gates);
    let hover = view.find_hover(app.window_rect(), app.mouse.position());
    view.draw(app, draw, app.window_rect(), hover);
}

pub(crate) fn event(app: &nannou::App, logic_gates: &mut crate::LogicGates, event: nannou::Event) {
    let view = crate::view(app, logic_gates);
    if let nannou::Event::WindowEvent { id: _, simple: Some(event) } = event {
        match event {
            WindowEvent::MousePressed(MouseButton::Left) => {
                let hovered = view.find_hover(app.window_rect(), app.mouse.position());
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
