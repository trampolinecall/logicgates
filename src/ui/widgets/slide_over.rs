use std::time::Duration;

use crate::{
    theme::Theme,
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

    drawer_out: bool,
    last_switch_time: Duration,
    toggle_pressed: bool,
}

impl<Base: Widget, Over: Widget> SlideOver<Base, Over> {
    pub(crate) fn new(id_maker: &mut WidgetIdMaker, base: Base, over: Over) -> Self {
        Self { id: id_maker.next_id(), base, over, drawer_out: false, toggle_pressed: false, last_switch_time: Duration::ZERO }
    }

    fn calculate_slide_over_rects(&self, app: &nannou::App, rect: nannou::geom::Rect) -> (Option<nannou::geom::Rect>, nannou::geom::Rect) {
        let time_since_switch = app.duration.since_start - self.last_switch_time;
        let time_interp = (Theme::DEFAULT.animation_ease)((time_since_switch.as_secs_f32() / Theme::DEFAULT.animation_time).clamp(0.0, 1.0));
        let x_interp = if self.drawer_out { time_interp } else { 1.0 - time_interp };

        let over_size = self.over.size(rect.w_h());
        let over_rect = nannou::geom::Rect::from_wh(over_size.into()).align_y_of(nannou::geom::Align::End, rect).left_of(rect).shift_x(over_size.0 * x_interp);
        let toggle_button_rect = nannou::geom::Rect::from_wh(Theme::DEFAULT.slide_out_size.into()).right_of(over_rect).align_top_of(rect).shift_y(-Theme::DEFAULT.slide_out_toggle_y_offset);
        let over_rect_needed = self.drawer_out || time_interp != 1.0; // drawer is out or we are in the middle of an animation
        (if over_rect_needed { Some(over_rect) } else { None }, toggle_button_rect)
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
        let (base_drawing, mut subscriptions) = self.base.view(app, logic_gates, rect);

        let (over_rect, toggle_button_rect) = self.calculate_slide_over_rects(app, rect);
        let over_drawing = over_rect.map(|over_rect| {
            let (over_drawing, over_subscriptions) = self.over.view(app, logic_gates, over_rect);
            subscriptions.extend(over_subscriptions);
            over_drawing
        });
        if self.toggle_pressed {
            subscriptions.push(view::Subscription::LeftMouseUp(Box::new({
                let slide_over_id = self.id;
                move |_| TargetedUIMessage { target: slide_over_id, message: UIMessage::LeftMouseUp }
            })));
        }

        (
            Box::new(SlideOverDrawing { base_drawing, toggle_button_drawing: ToggleButtonDrawing { rect: toggle_button_rect, slide_over_id: self.id, pressed: self.toggle_pressed }, over_drawing }),
            subscriptions,
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
                    self.drawer_out = !self.drawer_out;
                    None
                } else {
                    None
                }
            }
            UIMessage::MouseDownOnSlideOverToggleButton => {
                self.toggle_pressed = true;
                None
            }
            UIMessage::MouseDownOnSlider(_, _) => None,
        }
    }
}

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
        if let Some(over_drawing) = &self.over_drawing {
            if let x @ Some(_) = over_drawing.find_hover(mouse) {
                return x;
            }
        }

        self.base_drawing.find_hover(mouse)
    }
}
impl view::Drawing for ToggleButtonDrawing {
    fn draw(&self, _: &crate::LogicGates, draw: &nannou::Draw, hovered: Option<&dyn view::Drawing>) {
        let mut rect = draw.rect().xy(self.rect.xy()).wh(self.rect.wh()).color(Theme::DEFAULT.button_normal_bg);
        if let Some(hovered) = hovered {
            if std::ptr::eq(hovered, self) {
                // TODO: fix clippy lint about this
                rect = rect.color(Theme::DEFAULT.button_hover_bg);
            }
        }
        if self.pressed {
            rect = rect.color(Theme::DEFAULT.button_pressed_bg);
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

    fn left_mouse_down(&self, _: &nannou::App) -> Option<TargetedUIMessage> {
        Some(TargetedUIMessage { target: self.slide_over_id, message: UIMessage::MouseDownOnSlideOverToggleButton })
    }
}
