use crate::{ui::message::TargetedUIMessage, LogicGates};

use nannou::prelude::*;

pub(crate) struct View<D: Drawing> {
    main_drawing: D,
    subscriptions: Vec<Subscription>,
}

pub(crate) enum Subscription {
    MouseMoved(Box<dyn Fn(Vec2) -> TargetedUIMessage>),
    LeftMouseUp(Box<dyn Fn() -> TargetedUIMessage>),
}

pub(crate) trait Drawing {
    fn draw(&self, logic_gates: &LogicGates, draw: &nannou::Draw, hovered: Option<&dyn Drawing>);
    // iterate through this and child widgets in z order to check which one the mouse is currently over
    fn find_hover(&self, mouse: Vec2) -> Option<&dyn Drawing>;

    fn left_mouse_down(&self) -> Option<TargetedUIMessage> {
        None
    }
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, logic_gates: &LogicGates) {
    let view = view(app, logic_gates);
    let hover = view.main_drawing.find_hover(app.mouse.position());
    view.main_drawing.draw(logic_gates, draw, hover);
}

pub(crate) fn event(app: &nannou::App, logic_gates: &LogicGates, event: nannou::Event) -> Vec<TargetedUIMessage> {
    let view = view(app, logic_gates);
    if let nannou::Event::WindowEvent { id: _, simple: Some(event) } = event {
        match event {
            WindowEvent::MousePressed(MouseButton::Left) => {
                let hovered = view.main_drawing.find_hover(app.mouse.position());
                if let Some(hovered) = hovered {
                    hovered.left_mouse_down().into_iter().collect()
                } else {
                    Vec::new()
                }
            }

            WindowEvent::MouseMoved(mouse_pos) => view
                .subscriptions
                .iter()
                .filter_map(|sub| match sub {
                    Subscription::MouseMoved(callback) => Some(callback(mouse_pos)),
                    Subscription::LeftMouseUp(_) => None,
                })
                .collect(),

            WindowEvent::MouseReleased(MouseButton::Left) => view
                .subscriptions
                .iter()
                .filter_map(|sub| match sub {
                    Subscription::MouseMoved(_) => None,
                    Subscription::LeftMouseUp(callback) => Some(callback()),
                })
                .collect(),

            _ => Vec::new(),
        }
    } else {
        Vec::new()
    }
}

fn view(app: &nannou::App, logic_gates: &LogicGates) -> View<impl Drawing> {
    let (main_drawing, subscriptions) = logic_gates.view(app.window_rect());

    View { main_drawing, subscriptions }
}
