use std::{cell::RefCell, marker::PhantomData, time::Duration};

use crate::{
    newview::{
        id::{ViewId, ViewIdMaker},
        lens::{self, Lens},
        widgets::button::ButtonState,
        Event, Subscription, View,
    },
    theme::Theme,
    ui::{
        message::{TargetedUIMessage, UIMessage},
        widgets::WidgetId,
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

struct SlideOverView<Data, L: Lens<Data, SlideOverState>, ButtonView: View<Data>, BaseView: View<Data>, OverView: View<Data>> {
    lens: L,

    base: BaseView,
    over: OverView,
    button: ButtonView,

    drawer_openness: f32,

    layout: RefCell<Option<(nannou::geom::Rect, SlideOverLayout)>>, // cache layout

    _phantom: PhantomData<fn(&Data) -> &SlideOverState>,
}
struct SlideOverLayout {
    base_rect: nannou::geom::Rect,
    over_rect: Option<nannou::geom::Rect>,
    toggle_button_rect: nannou::geom::Rect,
}

impl<Data, L: Lens<Data, SlideOverState>, ButtonView: View<Data>, BaseView: View<Data>, OverView: View<Data>> SlideOverView<Data, L, ButtonView, BaseView, OverView> {
    fn layout<'layout>(&self, given_rect: nannou::geom::Rect, layout_field: &'layout mut Option<(nannou::geom::Rect, SlideOverLayout)>) -> &'layout SlideOverLayout {
        // the layout should really never need to be computed more than once because the view tree is supposed to only be used for one frame so the layout should really never change
        // but conceptually the layout is supposed to be computed for whatever rect is passed into draw() or find_hover() and this only caches whatever layout was computed last

        let needs_recompute = match layout_field {
            None => true,
            Some((old_given_rect, _)) if *old_given_rect != given_rect => true,

            _ => false,
        };
        if needs_recompute {
            let over_size = self.over.size(given_rect.w_h());
            let over_rect = nannou::geom::Rect::from_wh(over_size.into()).align_y_of(nannou::geom::Align::End, given_rect).left_of(given_rect).shift_x(over_size.0 * self.drawer_openness);
            let toggle_button_rect = nannou::geom::Rect::from_wh(Theme::DEFAULT.slide_out_size.into()).right_of(over_rect).align_top_of(given_rect).shift_y(-Theme::DEFAULT.slide_out_toggle_y_offset);
            let over_rect_needed = self.drawer_openness != 0.0;

            *layout_field = Some((given_rect, SlideOverLayout { base_rect: given_rect, over_rect: if over_rect_needed { Some(over_rect) } else { None }, toggle_button_rect }));
        }

        &layout_field.as_ref().expect("layout was either already computed or just computed").1
    }
}

impl<Data, L: Lens<Data, SlideOverState>, ButtonView: View<Data>, BaseView: View<Data>, OverView: View<Data>> View<Data> for SlideOverView<Data, L, ButtonView, BaseView, OverView> {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, rect: nannou::geom::Rect, hover: Option<ViewId>) {
        let mut layout_borrow = self.layout.borrow_mut();
        let layout = self.layout(rect, &mut layout_borrow);

        self.base.draw(app, draw, layout.base_rect, hover);
        if let Some(over_rect) = layout.over_rect {
            self.over.draw(app, draw, over_rect, hover);
        }
        self.button.draw(app, draw, layout.toggle_button_rect, hover);
    }

    fn find_hover(&self, rect: nannou::geom::Rect, mouse: nannou::geom::Vec2) -> Option<ViewId> {
        let mut layout_borrow = self.layout.borrow_mut();
        let layout = self.layout(rect, &mut layout_borrow);

        // go in z order from top to bottom
        if let x @ Some(_) = self.button.find_hover(layout.toggle_button_rect, mouse) {
            return x;
        }

        if let Some(over_rect) = layout.over_rect {
            if let x @ Some(_) = self.over.find_hover(over_rect, mouse) {
                return x;
            }
        }

        if let x @ Some(_) = self.base.find_hover(layout.base_rect, mouse) {
            return x;
        }

        None
    }

    fn size(&self, given: (f32, f32)) -> (f32, f32) {
        self.base.size(given)
    }

    fn targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: Event) {
        // only one of these will respond
        self.button.targeted_event(app, data, target, event);
        self.over.targeted_event(app, data, target, event);
        self.base.targeted_event(app, data, target, event);
    }

    fn event(&self, app: &nannou::App, data: &mut Data, event: Event) {
        match event {
            Event::LeftMouseDown => {}
        }
    }

    fn subscriptions(&self) -> Vec<Subscription<Data>> {
        [self.base.subscriptions(), self.over.subscriptions(), self.button.subscriptions()].into_iter().flatten().collect()
    }
}

pub(crate) fn slide_over<Data>(
    app: &nannou::App,
    id_maker: &mut ViewIdMaker,
    data: &Data,
    get_slide_over_data: impl Lens<Data, SlideOverState> + Copy,
    base: impl View<Data>,
    over: impl View<Data>,
) -> impl View<Data> {
    let slide_over_data = get_slide_over_data.get(data);

    let time_since_switch = app.duration.since_start - slide_over_data.last_switch_time;
    let time_interp = (Theme::DEFAULT.animation_ease)((time_since_switch.as_secs_f32() / Theme::DEFAULT.animation_time).clamp(0.0, 1.0));
    let drawer_openness = if slide_over_data.drawer_out { time_interp } else { 1.0 - time_interp };

    let button = crate::newview::widgets::button::button(
        id_maker,
        data,
        lens::from_closures(move |data| &get_slide_over_data.get(data).toggle_button, move |data| &mut get_slide_over_data.get_mut(data).toggle_button),
    );
    SlideOverView { lens: get_slide_over_data, base, over, button, drawer_openness, layout: RefCell::new(None), _phantom: PhantomData }
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
