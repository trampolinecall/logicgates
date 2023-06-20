pub(crate) mod widgets;

pub(crate) struct UI {
    pub(crate) new_slide_over: widgets::slide_over::SlideOverState,
    pub(crate) main_simulation_state: widgets::simulation::SimulationWidgetState,
    pub(crate) subticks_slider_state: widgets::slider::SliderState<isize>,
}

impl UI {
    pub(crate) fn new() -> UI {
        UI {
            new_slide_over: widgets::slide_over::SlideOverState::new(),
            main_simulation_state: widgets::simulation::SimulationWidgetState::new(),
            subticks_slider_state: widgets::slider::SliderState::new(),
        }
    }
}
