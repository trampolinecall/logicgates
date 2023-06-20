use std::marker::PhantomData;

use crate::{
    newview::{
        id::{ViewId, ViewIdMaker},
        lens::Lens,
        Event, Subscription, View,
    },
    theme::Theme,
};

pub(crate) struct ButtonData {
    pressed: bool,
}

impl ButtonData {
    pub(crate) fn new() -> ButtonData {
        ButtonData { pressed: false }
    }
}

struct ButtonView<Data, GetButtonData: Lens<Data, ButtonData>> {
    id: ViewId,

    rect: nannou::geom::Rect,

    button_data_lens: GetButtonData,

    _phantom: PhantomData<fn(&Data) -> &ButtonData>,
}

impl<Data, GetButtonData: Lens<Data, ButtonData>> View<Data> for ButtonView<Data, GetButtonData> {
    fn draw(&self, app: &nannou::App, data: &Data, draw: &nannou::Draw, hover: Option<ViewId>) {
        let mut rect = draw.rect().xy(self.rect.xy()).wh(self.rect.wh()).color(Theme::DEFAULT.button_normal_bg);
        if hover == Some(self.id) {
            rect = rect.color(Theme::DEFAULT.button_hover_bg);
        }
        if self.button_data_lens.get(data).pressed {
            rect = rect.color(Theme::DEFAULT.button_pressed_bg);
        }

        rect.finish();
    }

    fn find_hover(&self, mouse: nannou::geom::Vec2) -> Option<ViewId> {
        if self.rect.contains(mouse) {
            Some(self.id)
        } else {
            None
        }
    }

    fn targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: Event) {
        if target == self.id {
            self.event(app, data, event)
        }
    }
    fn event(&self, app: &nannou::App, data: &mut Data, event: Event) {
        match event {
            Event::LeftMouseDown => self.button_data_lens.get_mut(data).pressed = true,
        }
    }

    fn subscriptions(&self, data: &Data) -> Vec<Subscription<Data>> {
        if self.button_data_lens.get(data).pressed {
            // TODO: callback
            vec![Subscription::LeftMouseUp(Box::new(|app, data| self.button_data_lens.get_mut(data).pressed = false))]
        } else {
            Vec::new()
        }
    }
}

pub(crate) fn button<Data>(id_maker: &mut ViewIdMaker, rect: nannou::geom::Rect, get_button_data: impl Lens<Data, ButtonData>) -> impl View<Data> {
    // TODO: figure out how layouting is supposed to work
    ButtonView { id: id_maker.next_id(), rect, button_data_lens: get_button_data, _phantom: PhantomData }
}
