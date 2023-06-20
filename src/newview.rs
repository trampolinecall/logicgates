pub(crate) mod id;
pub(crate) mod lens;
pub(crate) mod widgets;

use nannou::prelude::*;

pub(crate) enum Event {
    LeftMouseDown,
}
pub(crate) enum Subscription<'a, Data> {
    MouseMoved(Box<dyn Fn(&nannou::App, &mut Data, Vec2) + 'a>),
    LeftMouseUp(Box<dyn Fn(&nannou::App, &mut Data) + 'a>),
}

// new view system heavilty inspired by xilem
// specifically this blog post: https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html
// kind of like a merge of the old Widget and old Drawing trait
pub(crate) trait View<Data> {
    fn id(&self) -> id::ViewId;

    fn draw(&self, app: &nannou::App, data: &Data, draw: &nannou::Draw, hover: Option<id::ViewId>);
    fn find_hover(&self, mouse: nannou::geom::Vec2) -> Option<id::ViewId>;

    fn targeted_event(&self, app: &nannou::App, data: &mut Data, target: id::ViewId, event: Event);
    fn event(&self, app: &nannou::App, data: &mut Data, event: Event);
    fn subscriptions(&self, data: &Data) -> Vec<Subscription<Data>>;
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, logic_gates: &crate::LogicGates) {
    let view = view(app);
    let hover = view.find_hover(app.mouse.position());
    view.draw(app, logic_gates, draw, hover);
}

pub(crate) fn event(app: &nannou::App, logic_gates: &mut crate::LogicGates, event: nannou::Event) {
    let view = view(app);
    if let nannou::Event::WindowEvent { id: _, simple: Some(event) } = event {
        match event {
            WindowEvent::MousePressed(MouseButton::Left) => {
                let hovered = view.find_hover(app.mouse.position());
                if let Some(hovered) = hovered {
                    view.targeted_event(app, logic_gates, hovered, Event::LeftMouseDown);
                }
            }

            WindowEvent::MouseMoved(mouse_pos) => {
                for sub in view.subscriptions(logic_gates) {
                    match sub {
                        Subscription::MouseMoved(callback) => callback(app, logic_gates, mouse_pos),
                        Subscription::LeftMouseUp(_) => {}
                    }
                }
            }

            WindowEvent::MouseReleased(MouseButton::Left) => {
                for sub in view.subscriptions(logic_gates) {
                    match sub {
                        Subscription::MouseMoved(_) => {}
                        Subscription::LeftMouseUp(callback) => callback(app, logic_gates),
                    }
                }
            }

            _ => {}
        }
    }
}

fn view(app: &nannou::App) -> impl View<crate::LogicGates> {
    let mut id_maker = id::ViewIdMaker::new();
    widgets::button::button(
        &mut id_maker,
        nannou::geom::Rect::from_x_y_w_h(0.0, 0.0, 100.0, 100.0),
        lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.newui, |logic_gates: &mut crate::LogicGates| &mut logic_gates.newui),
    )
}
