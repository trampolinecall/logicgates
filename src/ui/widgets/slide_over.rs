use std::{marker::PhantomData, time::Duration};

use nannou::geom::Vec2;

use crate::{
    theme::Theme,
    ui::widgets::button::ButtonState,
    view::{
        id::{ViewId, ViewIdMaker},
        lens::{self, Lens},
        GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout,
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

struct SlideOverView<Data, ButtonView: ViewWithoutLayout<Data>, BaseView: ViewWithoutLayout<Data>, OverView: ViewWithoutLayout<Data>> {
    base: BaseView,
    over: OverView,
    button: ButtonView,

    drawer_openness: f32,

    _phantom: PhantomData<fn(&Data) -> &SlideOverState>,
}
struct SlideOverLayout<'original, Data, ButtonView: ViewWithoutLayout<Data> + 'original, BaseView: ViewWithoutLayout<Data> + 'original, OverView: ViewWithoutLayout<Data> + 'original> {
    base: BaseView::WithLayout<'original>,
    over: OverView::WithLayout<'original>,
    button: ButtonView::WithLayout<'original>,

    over_shift: Option<f32>,
    toggle_button_offset: Vec2,
}

impl<Data, ButtonView: ViewWithoutLayout<Data>, BaseView: ViewWithoutLayout<Data>, OverView: ViewWithoutLayout<Data>> ViewWithoutLayout<Data> for SlideOverView<Data, ButtonView, BaseView, OverView> {
    type WithLayout<'without_layout>  = SlideOverLayout<'without_layout, Data, ButtonView, BaseView, OverView> where Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        let base_sc = sc;
        let over_sc = SizeConstraints { min: sc.min, max: Vec2::new(sc.max.x - Theme::DEFAULT.slide_out_size.0, sc.max.y) };
        let button_sc = SizeConstraints { min: Theme::DEFAULT.slide_out_size.into(), max: Theme::DEFAULT.slide_out_size.into() };

        let base = self.base.layout(base_sc);
        let over = self.over.layout(over_sc);
        let button = self.button.layout(button_sc);

        let base_size = base.size();
        let over_size = over.size();
        let button_size = button.size();

        // let over_rect = nannou::geom::Rect::from_wh(over_size.into()).align_y_of(nannou::geom::Align::End, given_rect).left_of(given_rect).shift_x(over_size.0 * self.drawer_openness);
        // let toggle_button_rect = nannou::geom::Rect::from_wh(Theme::DEFAULT.slide_out_size.into()).right_of(over_rect).align_top_of(given_rect).shift_y(-Theme::DEFAULT.slide_out_toggle_y_offset);

        let over_rect_needed = self.drawer_openness != 0.0;
        let over_right_edge = -base_size.x / 2.0 + over_size.x * self.drawer_openness;
        let over_shift = over_right_edge - over_size.x / 2.0;

        let button_offset = Vec2::new(over_right_edge + button_size.x / 2.0, base_size.y / 2.0 - Theme::DEFAULT.slide_out_toggle_y_offset);

        SlideOverLayout { base, over, button, over_shift: if over_rect_needed { Some(over_shift) } else { None }, toggle_button_offset: button_offset }
    }
}
impl<Data, ButtonView: ViewWithoutLayout<Data>, BaseView: ViewWithoutLayout<Data>, OverView: ViewWithoutLayout<Data>> View<Data> for SlideOverLayout<'_, Data, ButtonView, BaseView, OverView> {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, center: nannou::geom::Vec2, hover: Option<ViewId>) {
        self.base.draw(app, draw, center, hover);
        if let Some(over_shift) = self.over_shift {
            self.over.draw(app, draw, center + Vec2::new(over_shift, 0.0), hover);
        }
        self.button.draw(app, draw, center + self.toggle_button_offset, hover);
    }

    fn find_hover(&self, center: nannou::geom::Vec2, mouse: Vec2) -> Option<ViewId> {
        // go in z order from top to bottom
        if let x @ Some(_) = self.button.find_hover(center + self.toggle_button_offset, mouse) {
            return x;
        }

        if let Some(over_shift) = self.over_shift {
            if let x @ Some(_) = self.over.find_hover(center + Vec2::new(over_shift, 0.0), mouse) {
                return x;
            }
        }

        if let x @ Some(_) = self.base.find_hover(center, mouse) {
            return x;
        }

        None
    }

    fn size(&self) -> Vec2 {
        self.base.size()
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
    base: impl ViewWithoutLayout<Data>,
    over: impl ViewWithoutLayout<Data>,
) -> impl ViewWithoutLayout<Data> {
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
    SlideOverView { base, over, button, drawer_openness, _phantom: PhantomData }
}
