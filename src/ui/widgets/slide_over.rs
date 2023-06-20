use std::{marker::PhantomData, time::Duration};

use crate::{
    theme::Theme,
    ui::widgets::button::ButtonState,
    view::{
        id::{ViewId, ViewIdMaker},
        layout_cache::LayoutCache,
        lens::{self, Lens},
        GeneralEvent, TargetedEvent, View,
    },
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
}

struct SlideOverView<Data, ButtonView: View<Data>, BaseView: View<Data>, OverView: View<Data>> {
    base: BaseView,
    over: OverView,
    button: ButtonView,

    drawer_openness: f32,

    layout: LayoutCache<SlideOverLayout>,

    _phantom: PhantomData<fn(&Data) -> &SlideOverState>,
}
struct SlideOverLayout {
    base_rect: nannou::geom::Rect,
    over_rect: Option<nannou::geom::Rect>,
    toggle_button_rect: nannou::geom::Rect,
}

impl<Data, ButtonView: View<Data>, BaseView: View<Data>, OverView: View<Data>> SlideOverView<Data, ButtonView, BaseView, OverView> {
    fn layout(&self, given_rect: nannou::geom::Rect) -> SlideOverLayout {
        let over_size = self.over.size(given_rect.w_h());
        let over_rect = nannou::geom::Rect::from_wh(over_size.into()).align_y_of(nannou::geom::Align::End, given_rect).left_of(given_rect).shift_x(over_size.0 * self.drawer_openness);
        let toggle_button_rect = nannou::geom::Rect::from_wh(Theme::DEFAULT.slide_out_size.into()).right_of(over_rect).align_top_of(given_rect).shift_y(-Theme::DEFAULT.slide_out_toggle_y_offset);
        let over_rect_needed = self.drawer_openness != 0.0;

        SlideOverLayout { base_rect: given_rect, over_rect: if over_rect_needed { Some(over_rect) } else { None }, toggle_button_rect }
    }
}

impl<Data, ButtonView: View<Data>, BaseView: View<Data>, OverView: View<Data>> View<Data> for SlideOverView<Data, ButtonView, BaseView, OverView> {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, rect: nannou::geom::Rect, hover: Option<ViewId>) {
        self.layout.with_layout(
            rect,
            |given_rect| self.layout(given_rect),
            |layout| {
                self.base.draw(app, draw, layout.base_rect, hover);
                if let Some(over_rect) = layout.over_rect {
                    self.over.draw(app, draw, over_rect, hover);
                }
                self.button.draw(app, draw, layout.toggle_button_rect, hover);
            },
        )
    }

    fn find_hover(&self, rect: nannou::geom::Rect, mouse: nannou::geom::Vec2) -> Option<ViewId> {
        self.layout.with_layout(
            rect,
            |given_rect| self.layout(given_rect),
            |layout| {
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
            },
        )
    }

    fn size(&self, given: (f32, f32)) -> (f32, f32) {
        self.base.size(given)
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        // only one of these will respond
        self.button.send_targeted_event(app, data, target, event);
        self.over.send_targeted_event(app, data, target, event);
        self.base.send_targeted_event(app, data, target, event);
    }

    fn targeted_event(&self, _: &nannou::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, app: &nannou::App, data: &mut Data, event: GeneralEvent) {
        self.base.general_event(app, data, event);
        self.over.general_event(app, data, event);
        self.button.general_event(app, data, event);
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
    let drawer_openness = get_slide_over_data.with(data, |slide_over_data| {
        let time_since_switch = app.duration.since_start - slide_over_data.last_switch_time;
        let time_interp = (Theme::DEFAULT.animation_ease)((time_since_switch.as_secs_f32() / Theme::DEFAULT.animation_time).clamp(0.0, 1.0));
        if slide_over_data.drawer_out {
            time_interp
        } else {
            1.0 - time_interp
        }
    });

    let button = crate::ui::widgets::button::button(
        id_maker,
        data,
        lens::compose(get_slide_over_data, lens::from_closures(move |slide_over_data: &SlideOverState| &slide_over_data.toggle_button, move |slide_over_data| &mut slide_over_data.toggle_button)),
        move |app, data| {
            get_slide_over_data.with_mut(data, |slide_over_data| {
                slide_over_data.drawer_out = !slide_over_data.drawer_out;
                slide_over_data.last_switch_time = app.duration.since_start;
            });
        },
    );
    SlideOverView { base, over, button, drawer_openness, layout: LayoutCache::new(), _phantom: PhantomData }
}
