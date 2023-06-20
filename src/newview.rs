pub(crate) mod id;
pub(crate) mod lens;
pub(crate) mod widgets;

use nannou::prelude::*;

#[derive(Copy, Clone)]
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
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, rect: nannou::geom::Rect, hover: Option<id::ViewId>);
    fn find_hover(&self, rect: nannou::geom::Rect, mouse: nannou::geom::Vec2) -> Option<id::ViewId>;
    fn size(&self, given: (f32, f32)) -> (f32, f32); // TODO: this should eventually take some kind of constraint type instead of just a given size

    fn targeted_event(&self, app: &nannou::App, data: &mut Data, target: id::ViewId, event: Event);
    fn event(&self, app: &nannou::App, data: &mut Data, event: Event);
    fn subscriptions(&self) -> Vec<Subscription<Data>>;
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, logic_gates: &crate::LogicGates) {
    let view = view(app, logic_gates);
    let hover = view.find_hover(app.window_rect(), app.mouse.position());
    view.draw(app, draw, app.window_rect(), hover);
}

pub(crate) fn event(app: &nannou::App, logic_gates: &mut crate::LogicGates, event: nannou::Event) {
    let view = view(app, logic_gates);
    if let nannou::Event::WindowEvent { id: _, simple: Some(event) } = event {
        match event {
            WindowEvent::MousePressed(MouseButton::Left) => {
                let hovered = view.find_hover(app.window_rect(), app.mouse.position());
                if let Some(hovered) = hovered {
                    view.targeted_event(app, logic_gates, hovered, Event::LeftMouseDown);
                }
            }

            WindowEvent::MouseMoved(mouse_pos) => {
                for sub in view.subscriptions() {
                    match sub {
                        Subscription::MouseMoved(callback) => callback(app, logic_gates, mouse_pos),
                        Subscription::LeftMouseUp(_) => {}
                    }
                }
            }

            WindowEvent::MouseReleased(MouseButton::Left) => {
                for sub in view.subscriptions() {
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

fn view(app: &nannou::App, logic_gates: &crate::LogicGates) -> impl View<crate::LogicGates> {
    let mut id_maker = id::ViewIdMaker::new();

    let simulation_view = crate::ui::widgets::new_simulation::simulation(
        &mut id_maker,
        lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.newui.main_simulation_state, |logic_gates| &mut logic_gates.newui.main_simulation_state),
        lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.simulation, |logic_gates| &mut logic_gates.simulation),
        logic_gates,
    );

    let mut rects: Vec<_> = (0..20)
        .map(|i| {
            Box::new(widgets::submodule::submodule(
                lens::unit(),
                crate::ui::widgets::new_test_rect::test_rect(&mut id_maker, nannou::color::srgb(i as f32 / 20.0, (20 - i) as f32 / 20.0, 0.0), ((i * 5 + 20) as f32, 10.0)),
            )) as Box<dyn View<_>>
        })
        .collect();
    rects.push(Box::new(crate::ui::widgets::new_slider::slider(
        &mut id_maker,
        Some(1.0),
        Some(20.0),
        lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.newui.subticks_slider_state, |logic_gates| &mut logic_gates.newui.subticks_slider_state),
        lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.subticks_per_update, |logic_gates| &mut logic_gates.subticks_per_update),
        logic_gates,
    )));

    let flow_view = crate::ui::widgets::new_flow::vertical_flow(&mut id_maker, rects);

    crate::ui::widgets::new_slide_over::slide_over(
        app,
        &mut id_maker,
        logic_gates,
        lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.newui.new_slide_over, |logic_gates: &mut crate::LogicGates| &mut logic_gates.newui.new_slide_over),
        simulation_view,
        flow_view,
    )
}
