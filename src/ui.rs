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
        let rects =
            (0..20).map(|i| Box::new(widgets::test_rect::TestRect::new(&mut id_maker, nannou::color::srgb(i as f32 / 20.0, (20 - i) as f32 / 20.0, 0.0), (100.0, 10.0))) as Box<dyn Widget>).collect();
        let flow = widgets::flow::Flow::new(&mut id_maker, rects);
        let slide_over = widgets::slide_over::SlideOver::new(&mut id_maker, simulation_widget, flow);
        UI { main_widget: slide_over }
    }
    pub(crate) fn targeted_message(&mut self, app: &nannou::App, tm: message::TargetedUIMessage) -> Option<crate::Message> {
        self.main_widget.targeted_message(app, tm)
    }
}
