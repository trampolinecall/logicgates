#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]
#![warn(clippy::semicolon_if_nothing_returned)]

use std::rc::Rc;

pub(crate) mod import;
pub(crate) mod simulation;
pub(crate) mod theme;
#[macro_use]
pub(crate) mod ui;
pub(crate) mod graphics;
pub(crate) mod view;

struct App {
    start_time: std::time::Instant,
    last_update: std::time::Instant,
}

impl App {
    fn new() -> Self {
        Self { start_time: std::time::Instant::now(), last_update: std::time::Instant::now() }
    }

    fn time_since_start(&self) -> std::time::Duration {
        std::time::Instant::now() - self.start_time
    }

    fn default_render_context_settings() -> sfml::window::ContextSettings {
        sfml::window::ContextSettings { antialiasing_level: 0, ..Default::default() }
    }
}

// TODO: find a better place to put this and reorganize everything
struct LogicGates {
    simulation: simulation::Simulation,
    ticks_per_second: isize,
    ui: ui::UI,
    font: Rc<sfml::SfBox<graphics::Font>>, // not ideal but
}

impl LogicGates {
    fn new() -> LogicGates {
        // TODO: convert panics to Result?
        let font_handle = font_kit::source::SystemSource::new()
            .select_best_match(&[font_kit::family_name::FamilyName::SansSerif, font_kit::family_name::FamilyName::Serif], &font_kit::properties::Properties::new())
            .expect("could not find appropriate font");
        let font = match font_handle {
            font_kit::handle::Handle::Path { path, font_index: _ } => graphics::Font::from_file(&path.to_string_lossy()).expect("could not load font"), // TODO: figure out how to handle font_index
            font_kit::handle::Handle::Memory { bytes: _, font_index: _ } => unimplemented!("loading font from memory"),
        };
        LogicGates { simulation: import::import(&std::env::args().nth(1).expect("expected input file")).unwrap(), ticks_per_second: 20, ui: ui::UI::new(), font: Rc::new(font) }
    }
}

fn main() {
    use sfml::{
        graphics::{RenderTarget, RenderWindow},
        window::{Event, Style},
    };

    let mut app = App::new();
    let mut logic_gates = LogicGates::new();
    let mut window = RenderWindow::new((800, 600), "logic gates", Style::DEFAULT, &App::default_render_context_settings());
    window.set_vertical_sync_enabled(true);

    while window.is_open() {
        // events
        while let Some(event) = window.poll_event() {
            match event {
                // TODO: put these in the event handler with everything else
                Event::Closed => window.close(),
                Event::Resized { width, height } => {
                    // update the view to the new size of the window
                    let visible_area = graphics::FloatRect::new(0.0, 0.0, width as f32, height as f32);
                    window.set_view(&sfml::graphics::View::from_rect(visible_area));
                }
                _ => view::event(&app, &window, &mut logic_gates, event),
            }
        }

        // update
        let mut time_since_last_update = std::time::Instant::now() - app.last_update;
        let time_between_updates = std::time::Duration::from_secs(1) / logic_gates.ticks_per_second as u32;
        while time_since_last_update > time_between_updates {
            simulation::logic::update(&mut logic_gates.simulation.gates, &mut logic_gates.simulation.nodes);
            time_since_last_update -= time_between_updates;
            app.last_update = std::time::Instant::now();
        }

        // draw
        window.set_active(true);
        view::render(&app, &mut window, &logic_gates);
        window.display();
    }
}

fn view(app: &App, logic_gates: &LogicGates) -> impl view::ViewWithoutLayout<LogicGates> {
    let mut id_maker = view::id::ViewIdMaker::new();

    let simulation_view = ui::widgets::simulation::simulation(
        &mut id_maker,
        view::lens::from_closures(|logic_gates: &LogicGates| &logic_gates.ui.main_simulation_state, |logic_gates| &mut logic_gates.ui.main_simulation_state),
        view::lens::from_closures(|logic_gates: &LogicGates| &logic_gates.simulation, |logic_gates| &mut logic_gates.simulation),
        &logic_gates.font,
        logic_gates,
    );

    let mut rects: [_; 20] = (0..20)
        .map(|i| {
            Some(ui::widgets::submodule::submodule(
                view::lens::unit(),
                ui::widgets::test_rect::test_rect(&mut id_maker, graphics::Color::rgb((i as f32 / 20.0 * 255.0) as u8, (((20 - i) as f32 / 20.0) * 255.0) as u8, 0), ((i * 5 + 20) as f32, 10.0)), // TODO: clean up this math
            ))
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap_or_else(|_| unreachable!());

    let subticks_slider = ui::widgets::slider::slider(
        &mut id_maker,
        Some(1),
        Some(1000),
        view::lens::from_closures(|logic_gates: &LogicGates| &logic_gates.ui.tps_slider_state, |logic_gates| &mut logic_gates.ui.tps_slider_state),
        view::lens::from_closures(|logic_gates: &LogicGates| &logic_gates.ticks_per_second, |logic_gates| &mut logic_gates.ticks_per_second),
        |mouse_diff| (mouse_diff / 10.0) as isize,
        &logic_gates.font,
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
