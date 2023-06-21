use std::marker::PhantomData;

use nannou::geom::{Rect, Vec2};

use crate::{
    theme::Theme,
    view::{
        id::{ViewId, ViewIdMaker},
        lens::Lens,
        GeneralEvent, TargetedEvent, View, SizeConstraints,
    },
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
    fn draw(&self, _: &nannou::App, draw: &nannou::Draw, center: Vec2, size_constraints: SizeConstraints, hover: Option<ViewId>) {
        let size = self.size(size_constraints);

        let mut rect = draw.rect().xy(center).wh(size).color(Theme::DEFAULT.button_normal_bg);
        if hover == Some(self.id) {
            rect = rect.color(Theme::DEFAULT.button_hover_bg);
        }
        if self.pressed {
            rect = rect.color(Theme::DEFAULT.button_pressed_bg);
        }
        rect = rect.stroke(nannou::color::rgb(0_u8, 0, 0));

        rect.finish();
    }

    fn find_hover(&self, center: Vec2, size_constraints: SizeConstraints, mouse: Vec2) -> Option<ViewId> {
        if Rect::from_xy_wh(center, self.size(size_constraints)).contains(mouse) {
            Some(self.id)
        } else {
            None
        }
    }

    fn size(&self, size_constraints: SizeConstraints) -> Vec2 {
        Vec2::new(150.0, 25.0).clamp(size_constraints.min, size_constraints.max) // TODO: move these to constants in the theme
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.id {
            self.targeted_event(app, data, event);
        }
    }
    fn targeted_event(&self, _: &nannou::App, data: &mut Data, event: TargetedEvent) {
        match event {
            TargetedEvent::LeftMouseDown => self.button_data_lens.with_mut(data, |button_data| button_data.pressed = true),
        }
    }
    fn general_event(&self, app: &nannou::App, data: &mut Data, event: GeneralEvent) {
        match event {
            GeneralEvent::LeftMouseUp => {
                if self.pressed {
                    self.button_data_lens.with_mut(data, |button_data| {
                        if button_data.pressed {
                            button_data.pressed = false;
                        }
                    });
                    (self.callback)(app, data);
                }
            }
            GeneralEvent::MouseMoved(_) => {}
        }
    }
}

// TODO: should this return ButtonView instead of an opaque type?
pub(crate) fn button<Data>(id_maker: &mut ViewIdMaker, data: &Data, get_button_data: impl Lens<Data, ButtonState>, callback: impl Fn(&nannou::App, &mut Data)) -> impl View<Data> {
    ButtonView { id: id_maker.next_id(), pressed: get_button_data.with(data, |button_data| button_data.pressed), button_data_lens: get_button_data, callback, _phantom: PhantomData }
}
