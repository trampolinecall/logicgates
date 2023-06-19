use std::time::Duration;

use crate::{
    ui::{
        message::{TargetedUIMessage, UIMessage},
        widgets::{Widget, WidgetId, WidgetIdMaker},
    },
    view,
};

// TODO: implement slide over from other sides
pub(crate) struct SlideOver<Base: Widget, Over: Widget> {
    id: WidgetId,
    base: Base,
    over: Over,

    slide_over_out: bool,
    last_switch_time: Duration,
    toggle_pressed: bool,
}

impl<Base: Widget, Over: Widget> SlideOver<Base, Over> {
    pub(crate) fn new(id_maker: &mut WidgetIdMaker, base: Base, over: Over) -> Self {
        Self { id: id_maker.next_id(), base, over, slide_over_out: false, toggle_pressed: false, last_switch_time: Duration::ZERO }
    }
}

impl<Base: Widget, Over: Widget> Widget for SlideOver<Base, Over> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn size(&self, given: (f32, f32)) -> (f32, f32) {
        self.base.size(given)
    }

    fn view(&self, app: &nannou::App, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>) {
        struct ToggleButtonDrawing {
            slide_over_id: WidgetId,
            rect: nannou::geom::Rect,
            pressed: bool,
        }

        struct SlideOverDrawing {
            base_drawing: Box<dyn view::Drawing>,
            over_drawing: Option<Box<dyn view::Drawing>>,
            toggle_button_drawing: ToggleButtonDrawing,
        }
        impl view::Drawing for SlideOverDrawing {
            fn draw(&self, logic_gates: &crate::LogicGates, draw: &nannou::Draw, hovered: Option<&dyn view::Drawing>) {
                self.base_drawing.draw(logic_gates, draw, hovered);
                if let Some(over_drawing) = &self.over_drawing {
                    over_drawing.draw(logic_gates, draw, hovered)
                }
                self.toggle_button_drawing.draw(logic_gates, draw, hovered);
            }

            fn find_hover(&self, mouse: nannou::prelude::Vec2) -> Option<&dyn view::Drawing> {
                if let x @ Some(_) = self.toggle_button_drawing.find_hover(mouse) {
                    return x;
                }

                self.base_drawing.find_hover(mouse)
            }
        }
        impl view::Drawing for ToggleButtonDrawing {
            fn draw(&self, _: &crate::LogicGates, draw: &nannou::Draw, hovered: Option<&dyn view::Drawing>) {
                let mut rect = draw.rect().xy(self.rect.xy()).wh(self.rect.wh()).color(nannou::color::srgb(1.0, 1.0, 1.0)); // TODO: put this in theme
                if let Some(hovered) = hovered {
                    if std::ptr::eq(hovered, self) {
                        // TODO: fix clippy lint about this
                        rect = rect.color(nannou::color::srgb(0.6, 0.6, 0.6));
                    }
                }
                if self.pressed {
                    rect = rect.color(nannou::color::srgb(0.3, 0.3, 0.3));
                }
                rect.finish()
            }

            fn find_hover(&self, mouse: nannou::prelude::Vec2) -> Option<&dyn view::Drawing> {
                if self.rect.contains(mouse) {
                    Some(self)
                } else {
                    None
                }
            }

            fn left_mouse_down(&self, time: &nannou::App) -> Option<TargetedUIMessage> {
                Some(TargetedUIMessage { target: self.slide_over_id, message: UIMessage::MouseDownOnSlideOverToggleButton })
            }
        }

        let (base_drawing, mut base_subscriptions) = self.base.view(app, logic_gates, rect);
        if self.toggle_pressed {
            base_subscriptions.push(view::Subscription::LeftMouseUp(Box::new({
                let slide_over_id = self.id;
                move |_| TargetedUIMessage { target: slide_over_id, message: UIMessage::LeftMouseUp }
            })));
        }
        let (over_drawing, over_subscriptions, toggle_button_left_x) = if self.slide_over_out {
            let over_size = self.over.size(rect.w_h());
            let (over_drawing, over_subscriptions) = self.over.view(app, logic_gates, nannou::geom::Rect::from_x_y_w_h(rect.left() + over_size.0 / 2.0, rect.y(), over_size.0, over_size.1));
            (Some(over_drawing), over_subscriptions, rect.left() + over_size.0)
        } else {
            (None, Vec::new(), rect.left())
        };

        base_subscriptions.extend(over_subscriptions);

        (
            Box::new(SlideOverDrawing {
                base_drawing,
                toggle_button_drawing: ToggleButtonDrawing {
                    rect: nannou::geom::Rect::from_x_y_w_h(toggle_button_left_x + 5.0, rect.top() - 50.0, 10.0, 30.0), // TODO: make constants for toggle button rect, also TODO: make a constant for y offset
                    slide_over_id: self.id,
                    pressed: self.toggle_pressed,
                },
                over_drawing,
            }),
            base_subscriptions,
        )
    }

    fn targeted_message(&mut self, app: &nannou::App, targeted_message: TargetedUIMessage) -> Option<crate::Message> {
        if targeted_message.target == self.id {
            self.message(app, targeted_message.message)
        } else if let Some(base_response) = self.base.targeted_message(app, targeted_message) {
            Some(base_response)
        } else if let Some(over_response) = self.over.targeted_message(app, targeted_message) {
            Some(over_response)
        } else {
            None
        }
    }

    fn message(&mut self, app: &nannou::App, message: UIMessage) -> Option<crate::Message> {
        match message {
            UIMessage::MouseDownOnGate(_) => None,
            UIMessage::MouseMoved(_) => None,
            UIMessage::LeftMouseUp => {
                if self.toggle_pressed {
                    self.toggle_pressed = false;
                    self.last_switch_time = app.duration.since_start;
                    self.slide_over_out = !self.slide_over_out;
                    None
                } else {
                    None
                }
            }
            UIMessage::MouseDownOnSlideOverToggleButton => {
                self.toggle_pressed = true;
                None
            }
        }
    }
}
