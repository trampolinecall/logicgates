#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]
#![warn(clippy::semicolon_if_nothing_returned)]

#[macro_use]
pub(crate) mod utils;
pub(crate) mod compiler;
pub(crate) mod simulation;
pub(crate) mod theme;
#[macro_use]
pub(crate) mod ui;
pub(crate) mod view;
pub(crate) mod draw;

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
    let draw = draw::Draw::new(app.draw(), app.window_rect());
    view::render(app, &draw, logic_gates);
    draw.to_frame(app, &frame).unwrap();
}

fn view(app: &nannou::App, logic_gates: &LogicGates) -> impl view::ViewWithoutLayout<LogicGates> {
    let mut id_maker = view::id::ViewIdMaker::new();

    let simulation_view = ui::widgets::simulation::simulation(
        &mut id_maker,
        view::lens::from_closures(|logic_gates: &LogicGates| &logic_gates.ui.main_simulation_state, |logic_gates| &mut logic_gates.ui.main_simulation_state),
        view::lens::from_closures(|logic_gates: &LogicGates| &logic_gates.simulation, |logic_gates| &mut logic_gates.simulation),
        logic_gates,
    );

    let mut rects: [_; 20] = (0..20)
        .map(|i| {
            Some(ui::widgets::submodule::submodule(
                view::lens::unit(),
                ui::widgets::test_rect::test_rect(&mut id_maker, nannou::color::srgb(i as f32 / 20.0, (20 - i) as f32 / 20.0, 0.0), ((i * 5 + 20) as f32, 10.0)),
            ))
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap_or_else(|_| unreachable!());

    let subticks_slider = ui::widgets::slider::slider(
        &mut id_maker,
        Some(1),
        Some(20),
        view::lens::from_closures(|logic_gates: &LogicGates| &logic_gates.ui.subticks_slider_state, |logic_gates| &mut logic_gates.ui.subticks_slider_state),
        view::lens::from_closures(|logic_gates: &LogicGates| &logic_gates.subticks_per_update, |logic_gates| &mut logic_gates.subticks_per_update),
        |mouse_diff| (mouse_diff / 10.0) as isize,
        logic_gates,
    );

    let flow_view = flow! {
        vertical

        rect0: rects[0].take().unwrap(),
        rect1: rects[1].take().unwrap(),
        rect2: rects[2].take().unwrap(),
        rect3: rects[3].take().unwrap(),
        rect4: rects[4].take().unwrap(),
        rect5: rects[5].take().unwrap(),
        rect6: rects[6].take().unwrap(),
        rect7: rects[7].take().unwrap(),
        rect8: rects[8].take().unwrap(),
        rect9: rects[9].take().unwrap(),
        rect10: rects[10].take().unwrap(),
        rect11: rects[11].take().unwrap(),
        rect12: rects[12].take().unwrap(),
        rect13: rects[13].take().unwrap(),
        rect14: rects[14].take().unwrap(),
        rect15: rects[15].take().unwrap(),
        rect16: rects[16].take().unwrap(),
        rect17: rects[17].take().unwrap(),
        rect18: rects[18].take().unwrap(),
        rect19: rects[19].take().unwrap(),
        slider: subticks_slider,
    };

    ui::widgets::slide_over::slide_over(
        app,
        &mut id_maker,
        logic_gates,
        view::lens::from_closures(|logic_gates: &LogicGates| &logic_gates.ui.new_slide_over, |logic_gates: &mut LogicGates| &mut logic_gates.ui.new_slide_over),
        simulation_view,
        flow_view,
    )
}
