use crate::{ui::widgets::Widget, view};

pub(crate) mod widgets;

pub(crate) struct UI {
    pub(crate) main_widget: widgets::simulation::SimulationWidget,
}

impl UI {
    pub(crate) fn new() -> UI {
        UI { main_widget: widgets::simulation::SimulationWidget::new() }
    }

    pub(crate) fn view(&self, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> impl view::Drawing {
        self.main_widget.view(logic_gates, rect)
    }
}
