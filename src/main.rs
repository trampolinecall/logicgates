#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]

#[macro_use]
pub(crate) mod utils;
pub(crate) mod compiler;
pub(crate) mod simulation;
pub(crate) mod theme;
pub(crate) mod ui;
pub(crate) mod view;

use nannou::prelude::*;

// TODO: find a better place to put this and reorganize everything
struct LogicGates {
    simulation: simulation::Simulation,
    subticks_per_update: isize,
    ui: ui::UI,
}

impl LogicGates {
    fn new(_: &App) -> LogicGates {
        LogicGates { simulation: compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap(), subticks_per_update: 1, ui: ui::UI::new() }
    }
}

fn main() {
    nannou::app(LogicGates::new).event(event).update(update).simple_window(draw).run();
}

fn event(app: &App, logic_gates: &mut LogicGates, event: Event) {
    view::event(app, logic_gates, event);
}

fn update(_: &App, logic_gates: &mut LogicGates, _: Update) {
    // TODO: adjust number of ticks for time since last update
    simulation::logic::update(&mut logic_gates.simulation.gates, &mut logic_gates.simulation.nodes, logic_gates.subticks_per_update as usize);
}

fn draw(app: &App, logic_gates: &LogicGates, frame: Frame) {
    let draw = app.draw();
    view::render(app, &draw, logic_gates);
    draw.to_frame(app, &frame).unwrap();
}

fn view(app: &nannou::App, logic_gates: &crate::LogicGates) -> impl view::View<crate::LogicGates> {
    let mut id_maker = view::id::ViewIdMaker::new();

    let simulation_view = ui::widgets::simulation::simulation(
        &mut id_maker,
        view::lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.ui.main_simulation_state, |logic_gates| &mut logic_gates.ui.main_simulation_state),
        view::lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.simulation, |logic_gates| &mut logic_gates.simulation),
        logic_gates,
    );

    let mut rects: Vec<_> = (0..20)
        .map(|i| {
            Box::new(ui::widgets::submodule::submodule(
                view::lens::unit(),
                ui::widgets::test_rect::test_rect(&mut id_maker, nannou::color::srgb(i as f32 / 20.0, (20 - i) as f32 / 20.0, 0.0), ((i * 5 + 20) as f32, 10.0)),
            )) as Box<dyn view::View<_>>
        })
        .collect();
    rects.push(Box::new(ui::widgets::slider::slider(
        &mut id_maker,
        Some(1),
        Some(20),
        view::lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.ui.subticks_slider_state, |logic_gates| &mut logic_gates.ui.subticks_slider_state),
        view::lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.subticks_per_update, |logic_gates| &mut logic_gates.subticks_per_update),
        |mouse_diff| (mouse_diff / 10.0) as isize,
        logic_gates,
    )));

    let flow_view = ui::widgets::flow::vertical_flow(&mut id_maker, rects);

    ui::widgets::slide_over::slide_over(
        app,
        &mut id_maker,
        logic_gates,
        view::lens::from_closures(|logic_gates: &crate::LogicGates| &logic_gates.ui.new_slide_over, |logic_gates: &mut crate::LogicGates| &mut logic_gates.ui.new_slide_over),
        simulation_view,
        flow_view,
    )
}
