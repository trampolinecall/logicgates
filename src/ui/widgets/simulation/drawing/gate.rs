use crate::{
    simulation::GateKey,
    theme::Theme,
    ui::{
        message::{TargetedUIMessage, UIMessage},
        widgets::WidgetId,
    },
    view::Drawing,
    LogicGates,
};

pub(crate) struct GateDrawing {
    pub(crate) simulation_widget_id: WidgetId,
    pub(crate) key: GateKey,
    pub(crate) rect: nannou::geom::Rect,
}

impl Drawing for GateDrawing {
    fn draw(&self, simulation: &LogicGates, draw: &nannou::Draw, hovered: Option<&dyn Drawing>) {
        if let Some(hovered) = hovered {
            if std::ptr::eq(hovered, self) {
                // TODO: fix clippy lint about this
                let hover_rect = self.rect.pad_left(-Theme::DEFAULT.gate_hover_dist).pad_top(-Theme::DEFAULT.gate_hover_dist).pad_right(-Theme::DEFAULT.gate_hover_dist).pad_bottom(-Theme::DEFAULT.gate_hover_dist); // expand by hover distance, this is the "stroke weight"
                draw.rect().xy(hover_rect.xy()).wh(hover_rect.wh()).color(Theme::DEFAULT.gate_hover_color);
            }
        }

        draw.rect().xy(self.rect.xy()).wh(self.rect.wh()).color(Theme::DEFAULT.gate_color);

        draw.text(simulation.simulation.gates[self.key].name(&simulation.simulation.circuits))
            .xy(self.rect.xy())
            .wh(self.rect.wh())
            .center_justify()
            .align_text_middle_y()
            .color(Theme::DEFAULT.gate_text_color);
    }

    fn find_hover(&self, mouse_pos: nannou::geom::Vec2) -> Option<&dyn Drawing> {
        if self.rect.contains(mouse_pos) {
            // TODO: hover distance
            return Some(self);
        }
        None
    }

    fn left_mouse_down(&self) -> Option<TargetedUIMessage> {
        Some(TargetedUIMessage { target: self.simulation_widget_id, message: UIMessage::MouseDownOnGate(self.key) })
    }
}
