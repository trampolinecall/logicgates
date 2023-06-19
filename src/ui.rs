pub(crate) mod message;
pub(crate) mod widgets;

use crate::ui::widgets::Widget;

pub(crate) struct UI {
    pub(crate) main_widget: widgets::slide_over::SlideOver<widgets::simulation::SimulationWidget, widgets::flow::Flow>,
}

impl UI {
    pub(crate) fn new() -> UI {
        let mut id_maker = widgets::WidgetIdMaker::new();
        let simulation_widget = widgets::simulation::SimulationWidget::new(&mut id_maker);
        let panel = widgets::flow::Flow::new(&mut id_maker, vec![]);
        let slide_over = widgets::slide_over::SlideOver::new(&mut id_maker, simulation_widget, panel);
        UI { main_widget: slide_over }
    }
    pub(crate) fn targeted_message(&mut self, tm: message::TargetedUIMessage) -> Option<crate::Message> {
        self.main_widget.targeted_message(tm)
    }
}
