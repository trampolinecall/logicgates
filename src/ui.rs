pub(crate) mod widgets;

pub(crate) struct UI {
    pub(crate) main_widget: widgets::simulation::SimulationWidget,
}

impl UI {
    pub(crate) fn new() -> UI {
        UI { main_widget: widgets::simulation::SimulationWidget::new() }
    }
}
