#[macro_use]
pub(crate) mod widgets;

pub(crate) struct UI {
    pub(crate) new_slide_over: widgets::slide_over::SlideOverState,
    pub(crate) btree: widgets::btree::BTree<widgets::simulation::SimulationWidgetState>,
    pub(crate) tps_slider_state: widgets::slider::SliderState<isize>,
}

impl UI {
    pub(crate) fn new() -> UI {
        UI {
            new_slide_over: widgets::slide_over::SlideOverState::new(),
            btree: widgets::btree::BTree::new_single(widgets::simulation::SimulationWidgetState::new()),
            tps_slider_state: widgets::slider::SliderState::new(),
        }
    }
}
