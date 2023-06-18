use crate::LogicGates;

use nannou::prelude::*;

pub(crate) struct View<D: Drawing, MouseMovedCallback: Fn(Vec2) -> Option<crate::Message>> {
    main_drawing: D,
    subscriptions: Subscriptions<MouseMovedCallback>,
}

struct Subscriptions<MouseMovedCallback: Fn(Vec2) -> Option<crate::Message>> {
    mouse_moved: Option<MouseMovedCallback>,
    left_mouse_up: Option<crate::Message>,
}

pub(crate) trait Drawing {
    fn draw(&self, logic_gates: &LogicGates, draw: &nannou::Draw, hovered: Option<&dyn Drawing>);
    // iterate through this and child widgets in z order to check which one the mouse is currently over
    fn find_hover(&self, mouse: Vec2) -> Option<&dyn Drawing>;

    // TODO: reconsider whether or not to use listeners
    fn left_mouse_down(&self) -> Option<crate::Message> {
        None
    }
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, logic_gates: &LogicGates) {
    let view = view(app, logic_gates);
    let hover = view.main_drawing.find_hover(app.mouse.position());
    view.main_drawing.draw(logic_gates, draw, hover);
}

pub(crate) fn event(app: &nannou::App, logic_gates: &LogicGates, event: nannou::Event) -> Option<crate::Message> {
    let view = view(app, logic_gates);
    if let nannou::Event::WindowEvent { id: _, simple: Some(event) } = event {
        match event {
            WindowEvent::MousePressed(MouseButton::Left) => {
                let hovered = view.main_drawing.find_hover(app.mouse.position());
                if let Some(hovered) = hovered {
                    hovered.left_mouse_down()
                } else {
                    None
                }
            }

            WindowEvent::MouseMoved(mouse_pos) => {
                if let Some(mouse_moved_callback) = view.subscriptions.mouse_moved {
                    mouse_moved_callback(mouse_pos)
                } else {
                    None
                }
            }

            WindowEvent::MouseReleased(MouseButton::Left) => view.subscriptions.left_mouse_up,

            _ => None,
        }
    } else {
        None
    }
}

fn view(app: &nannou::App, logic_gates: &LogicGates) -> View<impl Drawing, impl Fn(Vec2) -> Option<crate::Message>> {
    let main_drawing = logic_gates.view(app.window_rect());

    /* TODO
    let subscriptions = Subscriptions {
        mouse_moved: if logic_gates.ui.main_widget.cur_gate_drag.is_some() { Some(|mouse_pos| Some(crate::Message::MouseMoved(mouse_pos))) } else { None },
        left_mouse_up: if logic_gates.ui.main_widget.cur_gate_drag.is_some() { Some(crate::Message::MouseUp) } else { None },
    };
    */
    let subscriptions = Subscriptions { mouse_moved: None::<fn(_) -> _>, left_mouse_up: None };

    View { main_drawing, subscriptions }
}
