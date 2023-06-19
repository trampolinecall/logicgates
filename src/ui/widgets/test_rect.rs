use crate::{
    ui::{
        message::{TargetedUIMessage, UIMessage},
        widgets::{Widget, WidgetId, WidgetIdMaker},
    },
    view,
};

pub(crate) struct TestRect {
    id: WidgetId,
    color: nannou::color::Srgb,
    size: (f32, f32),
}

impl TestRect {
    pub(crate) fn new(id_maker: &mut WidgetIdMaker, color: nannou::color::Srgb, size: (f32, f32)) -> Self {
        Self { id: id_maker.next_id(), color, size }
    }
}

impl Widget for TestRect {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn size(&self, _: (f32, f32)) -> (f32, f32) {
        // TODO: clamp to given size
        self.size
    }

    fn view(&self, app: &nannou::App, _: &crate::LogicGates, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>) {
        struct TestRectDrawing(nannou::geom::Rect, nannou::color::Srgb);
        impl view::Drawing for TestRectDrawing {
            fn draw(&self, _: &crate::LogicGates, draw: &nannou::Draw, _: Option<&dyn view::Drawing>) {
                // TODO: use hovered?
                draw.rect().xy(self.0.xy()).wh(self.0.wh()).color(self.1);
            }

            fn find_hover(&self, mouse: nannou::prelude::Vec2) -> Option<&dyn view::Drawing> {
                if self.0.contains(mouse) {
                    Some(self)
                } else {
                    None
                }
            }
        }

        (Box::new(TestRectDrawing(nannou::geom::Rect::from_x_y_w_h(rect.x(), rect.y(), self.size.0, self.size.1), self.color)), Vec::new())
    }

    fn targeted_message(&mut self, app: &nannou::App, targeted_message: TargetedUIMessage) -> Option<crate::Message> {
        if targeted_message.target == self.id {
            self.message(app, targeted_message.message)
        } else {
            None
        }
    }

    fn message(&mut self, _: &nannou::App, message: UIMessage) -> Option<crate::Message> {
        match message {
            UIMessage::MouseDownOnGate(_) => None,
            UIMessage::MouseMoved(_) => None,
            UIMessage::LeftMouseUp => None,
            UIMessage::MouseDownOnSlideOverToggleButton => None,
        }
    }
}
