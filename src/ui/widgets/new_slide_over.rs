use std::{marker::PhantomData, time::Duration};

use crate::{
    newview::{
        id::{ViewIdMaker, ViewId},
        lens::{self, Lens},
        widgets::button::ButtonState,
        View, Event, Subscription,
    },
    theme::Theme,
    ui::{
        message::{TargetedUIMessage, UIMessage},
        widgets::{Widget, WidgetId, WidgetIdMaker},
    },
    view,
};

// TODO: implement slide over from other sides
pub(crate) struct SlideOverState {
    drawer_out: bool,
    last_switch_time: Duration,

    toggle_button: ButtonState,
}

impl SlideOverState {
    pub(crate) fn new() -> Self {
        Self { drawer_out: false, last_switch_time: Duration::ZERO, toggle_button: ButtonState::new() }
    }

    /*
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
    */
}

struct SlideOverView<Data, L: Lens<Data, SlideOverState>, ButtonView: View<SlideOverState>> {
    lens: L,

    button: ButtonView,

    _phantom: PhantomData<fn(&Data) -> &SlideOverState>,
}
impl<Data, L: Lens<Data, SlideOverState>, ButtonView: View<SlideOverState>> View<Data> for SlideOverView<Data, L, ButtonView> {
    fn draw(&self, app: &nannou::App, data: &Data, draw: &nannou::Draw, hover: Option<ViewId>) {
        todo!()
    }

    fn find_hover(&self, mouse: nannou::geom::Vec2) -> Option<ViewId> {
        todo!()
    }

    fn targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: Event) {
        todo!()
    }

    fn event(&self, app: &nannou::App, data: &mut Data, event: Event) {
        todo!()
    }

    fn subscriptions(&self, data: &Data) -> Vec<Subscription<Data>> {
        todo!()
    }
}

pub(crate) fn slide_over<Data>(id_maker: &mut ViewIdMaker, lens: impl Lens<Data, SlideOverState>) -> impl View<Data> {
    SlideOverView {
        lens,
        button: crate::newview::widgets::button::button(
            id_maker,
            todo!(),
            lens::from_closures(|slide_over_state: &SlideOverState| &slide_over_state.toggle_button, |slide_over_state| &mut slide_over_state.toggle_button),
        ),
        _phantom: PhantomData,
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
