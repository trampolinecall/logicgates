pub(crate) mod message;
pub(crate) mod widgets;

use crate::ui::widgets::Widget;

pub(crate) struct UI {
    pub(crate) main_widget: widgets::simulation::SimulationWidget,
}

impl UI {
    pub(crate) fn new() -> UI {
        let mut id_maker = widgets::WidgetIdMaker::new();
        UI { main_widget: widgets::simulation::SimulationWidget::new(&mut id_maker) }
    }
    pub(crate) fn targeted_message(&mut self, tm: message::TargetedUIMessage) -> Option<crate::Message> {
        self.main_widget.targeted_message(tm)
    }
}
