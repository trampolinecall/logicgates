use crate::{
    theme::Theme,
    ui::{
        message::{TargetedUIMessage, UIMessage},
        widgets::{Widget, WidgetId, WidgetIdMaker},
    },
    view,
};

pub(crate) struct Slider<Getter: Fn(&crate::LogicGates) -> f32, Changer: Fn(f32) -> crate::Message> {
    id: WidgetId,
    min: Option<f32>,
    max: Option<f32>,

    getter: Getter,
    change: Changer,

    drag_start: Option<(nannou::geom::Vec2, f32)>,
}

impl<Getter: Fn(&crate::LogicGates) -> f32, Changer: Fn(f32) -> crate::Message> Slider<Getter, Changer> {
    pub(crate) fn new(id_maker: &mut WidgetIdMaker, min: Option<f32>, max: Option<f32>, getter: Getter, change: Changer) -> Self {
        Self { id: id_maker.next_id(), min, max, getter, change, drag_start: None }
    }
}

impl<Getter: Fn(&crate::LogicGates) -> f32, Changer: Fn(f32) -> crate::Message> Widget for Slider<Getter, Changer> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn size(&self, _: (f32, f32)) -> (f32, f32) {
        (150.0, 25.0) // TODO: put this in theme?, also TODO: clamp to given space
    }

    fn view(&self, _: &nannou::App, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>) {
        // TODO: show as progressbar if both min and max
        let drawing = SliderDrawing { slider_id: self.id, rect, value: (self.getter)(logic_gates), pressed: self.drag_start.is_some() };

        (
            Box::new(drawing),
            if self.drag_start.is_some() {
                vec![
                    view::Subscription::MouseMoved(Box::new({
                        let slider_id = self.id;
                        move |_, mouse_pos| TargetedUIMessage { target: slider_id, message: UIMessage::MouseMoved(mouse_pos) }
                    })),
                    view::Subscription::LeftMouseUp(Box::new({
                        let slider_id = self.id;
                        move |_| TargetedUIMessage { target: slider_id, message: UIMessage::LeftMouseUp }
                    })),
                ]
            } else {
                Vec::new()
            },
        )
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
            UIMessage::MouseMoved(new_mouse_pos) => {
                if let Some((drag_start_mouse_pos, start_value)) = self.drag_start {
                    let diff = new_mouse_pos.x - drag_start_mouse_pos.x; // TODO: scale this
                    let mut new_value = start_value + diff;
                    if let Some(min) = self.min {
                        new_value = new_value.max(min);
                    }
                    if let Some(max) = self.max {
                        new_value = new_value.min(max);
                    }
                    Some((self.change)(new_value))
                } else {
                    None
                }
            }
            UIMessage::LeftMouseUp => {
                if self.drag_start.is_some() {
                    self.drag_start = None;
                    None
                } else {
                    None
                }
            }
            UIMessage::MouseDownOnSlideOverToggleButton => None,
            UIMessage::MouseDownOnSlider(mouse_pos, cur_value) => {
                self.drag_start = Some((mouse_pos, cur_value));
                None
            }
        }
    }
}

struct SliderDrawing {
    slider_id: WidgetId,
    rect: nannou::geom::Rect,
    value: f32,
    pressed: bool,
}

impl view::Drawing for SliderDrawing {
    fn draw(&self, _: &crate::LogicGates, draw: &nannou::Draw, hovered: Option<&dyn view::Drawing>) {
        let mut background_rect = draw.rect().xy(self.rect.xy()).wh(self.rect.wh()).color(Theme::DEFAULT.button_normal_bg);
        let mut text = draw.text(&self.value.to_string()).xy(self.rect.xy()).wh(self.rect.wh()).center_justify().align_text_middle_y().color(Theme::DEFAULT.button_normal_fg);
        if let Some(hovered) = hovered {
            if std::ptr::eq(hovered, self) {
                // TODO: fix clippy lint about this
                background_rect = background_rect.color(Theme::DEFAULT.button_hover_bg);
                text = text.color(Theme::DEFAULT.button_hover_fg);
            }
        }
        if self.pressed {
            background_rect = background_rect.color(Theme::DEFAULT.button_pressed_bg);
            text = text.color(Theme::DEFAULT.button_pressed_fg);
        }

        background_rect.finish();
        text.finish();
    }

    fn find_hover(&self, mouse: nannou::prelude::Vec2) -> Option<&dyn view::Drawing> {
        if self.rect.contains(mouse) {
            Some(self)
        } else {
            None
        }
    }

    fn left_mouse_down(&self, app: &nannou::App) -> Option<TargetedUIMessage> {
        Some(TargetedUIMessage { target: self.slider_id, message: UIMessage::MouseDownOnSlider(app.mouse.position(), self.value) })
    }
}
