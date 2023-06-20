use crate::ui::widgets;

pub(crate) struct UI {
    pub(crate) new_slide_over: widgets::new_slide_over::SlideOverState,
    pub(crate) main_simulation_state: widgets::new_simulation::SimulationWidgetState,
    pub(crate) subticks_slider_state: widgets::new_slider::SliderState<isize>,
}

impl UI {
    pub(crate) fn new() -> UI {
        UI {
            new_slide_over: widgets::new_slide_over::SlideOverState::new(),
            main_simulation_state: widgets::new_simulation::SimulationWidgetState::new(),
            subticks_slider_state: widgets::new_slider::SliderState::new(),
        }
    }
}
