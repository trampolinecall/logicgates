use std::{marker::PhantomData, time::Duration};

use crate::{
    graphics,
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
    toggle_button_offset: graphics::Vector2f,
}

impl<Data, ButtonView: ViewWithoutLayout<Data>, BaseView: ViewWithoutLayout<Data>, OverView: ViewWithoutLayout<Data>> ViewWithoutLayout<Data> for SlideOverView<Data, ButtonView, BaseView, OverView> {
    type WithLayout<'without_layout>  = SlideOverLayout<'without_layout, Data, ButtonView, BaseView, OverView> where Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        let base_sc = sc;
        let over_sc = SizeConstraints { min: sc.min, max: graphics::Vector2f::new(sc.max.x - Theme::DEFAULT.modify_ui_button_size.0, sc.max.y) };
        let button_sc = SizeConstraints { min: Theme::DEFAULT.modify_ui_button_size.into(), max: Theme::DEFAULT.modify_ui_button_size.into() };

        let base = self.base.layout(base_sc);
        let over = self.over.layout(over_sc);
        let button = self.button.layout(button_sc);

        let over_size = over.size();

        let over_rect_needed = self.drawer_openness != 0.0;
        let button_shift = over_size.x * self.drawer_openness;
        let over_shift = -over_size.x + over_size.x * self.drawer_openness;

        let button_offset = graphics::Vector2f::new(button_shift, Theme::DEFAULT.slide_out_toggle_y_offset);

        SlideOverLayout { base, over, button, over_shift: if over_rect_needed { Some(over_shift) } else { None }, toggle_button_offset: button_offset }
    }
}
impl<Data, ButtonView: ViewWithoutLayout<Data>, BaseView: ViewWithoutLayout<Data>, OverView: ViewWithoutLayout<Data>> View<Data> for SlideOverLayout<'_, Data, ButtonView, BaseView, OverView> {
    fn draw_inner(&self, app: &crate::App, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<ViewId>) {
        self.base.draw(app, target, top_left, hover);
        if let Some(over_shift) = self.over_shift {
            self.over.draw(app, target, top_left + graphics::Vector2f::new(over_shift, 0.0), hover);
        }
        self.button.draw(app, target, top_left + self.toggle_button_offset, hover);
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<ViewId> {
        // go in z order from top to bottom
        self.button
            .find_hover(top_left + self.toggle_button_offset, mouse)
            .or(self.over_shift.and_then(|over_shift| self.over.find_hover(top_left + graphics::Vector2f::new(over_shift, 0.0), mouse)))
            .or(self.base.find_hover(top_left, mouse))
    }

    fn size(&self) -> graphics::Vector2f {
        self.base.size()
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        // only one of these will respond
        self.button.send_targeted_event(app, data, target, event);
        self.over.send_targeted_event(app, data, target, event);
        self.base.send_targeted_event(app, data, target, event);
    }

    fn targeted_event(&self, _: &crate::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, app: &crate::App, data: &mut Data, event: GeneralEvent) {
        self.base.general_event(app, data, event);
        self.over.general_event(app, data, event);
        self.button.general_event(app, data, event);
    }
}

pub(crate) fn slide_over<Data>(
    app: &crate::App,
    id_maker: &mut ViewIdMaker,
    data: &Data,
    get_slide_over_data: impl Lens<Data, SlideOverState> + Copy,
    base: impl ViewWithoutLayout<Data>,
    over: impl ViewWithoutLayout<Data>,
) -> impl ViewWithoutLayout<Data> {
    let drawer_openness = get_slide_over_data.with(data, |slide_over_data| {
        let time_since_switch = app.time_since_start() - slide_over_data.last_switch_time;
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
                slide_over_data.last_switch_time = app.time_since_start();
            });
        },
    );
    SlideOverView { base, over, button, drawer_openness, _phantom: PhantomData }
}
