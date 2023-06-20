use std::marker::PhantomData;

use crate::{
    view::{
        id::{ViewId, ViewIdMaker},
        lens::Lens,
        Event, Subscription, View,
    },
    theme::Theme,
};

pub(crate) struct ButtonState {
    pressed: bool,
}

impl ButtonState {
    pub(crate) fn new() -> ButtonState {
        ButtonState { pressed: false }
    }
}

struct ButtonView<Data, GetButtonData: Lens<Data, ButtonState>, Callback: Fn(&nannou::App, &mut Data)> {
    id: ViewId,
    pressed: bool,
    button_data_lens: GetButtonData,
    callback: Callback,

    _phantom: PhantomData<fn(&Data) -> &ButtonState>,
}

impl<Data, GetButtonData: Lens<Data, ButtonState>, Callback: Fn(&nannou::App, &mut Data)> View<Data> for ButtonView<Data, GetButtonData, Callback> {
    fn draw(&self, _: &nannou::App, draw: &nannou::Draw, rect: nannou::geom::Rect, hover: Option<ViewId>) {
        let mut rect = draw.rect().xy(rect.xy()).wh(rect.wh()).color(Theme::DEFAULT.button_normal_bg);
        if hover == Some(self.id) {
            rect = rect.color(Theme::DEFAULT.button_hover_bg);
        }
        if self.pressed {
            rect = rect.color(Theme::DEFAULT.button_pressed_bg);
        }
        rect = rect.stroke(nannou::color::rgb(0_u8, 0, 0));

        rect.finish();
    }

    fn find_hover(&self, rect: nannou::geom::Rect, mouse: nannou::geom::Vec2) -> Option<ViewId> {
        if rect.contains(mouse) {
            Some(self.id)
        } else {
            None
        }
    }

    fn size(&self, given: (f32, f32)) -> (f32, f32) {
        (150.0, 25.0) // TODO: adapt to given size
    }

    fn targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: Event) {
        if target == self.id {
            self.event(app, data, event)
        }
    }
    fn event(&self, _: &nannou::App, data: &mut Data, event: Event) {
        match event {
            Event::LeftMouseDown => self.button_data_lens.with_mut(data, |button_data| button_data.pressed = true),
        }
    }

    fn subscriptions(&self) -> Vec<Subscription<Data>> {
        if self.pressed {
            vec![Subscription::LeftMouseUp(Box::new(|app, data| {
                self.button_data_lens.with_mut(data, |button_data| {
                    if button_data.pressed {
                        button_data.pressed = false;
                    }
                });
                (self.callback)(app, data);
            }))]
        } else {
            Vec::new()
        }
    }
}

// TODO: should this return ButtonView instead of an opaque type?
pub(crate) fn button<Data>(id_maker: &mut ViewIdMaker, data: &Data, get_button_data: impl Lens<Data, ButtonState>, callback: impl Fn(&nannou::App, &mut Data)) -> impl View<Data> {
    ButtonView { id: id_maker.next_id(), pressed: get_button_data.with(data, |button_data| button_data.pressed), button_data_lens: get_button_data, callback, _phantom: PhantomData }
}
