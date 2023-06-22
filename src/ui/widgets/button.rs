use std::marker::PhantomData;

use sfml::graphics::Shape;

use crate::{
    graphics,
    theme::Theme,
    view::{
        id::{ViewId, ViewIdMaker},
        lens::Lens,
        GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout,
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

struct ButtonView<Data, GetButtonData: Lens<Data, ButtonState>, Callback: Fn(&crate::App, &mut Data)> {
    id: ViewId,
    pressed: bool,
    button_data_lens: GetButtonData,
    callback: Callback,

    _phantom: PhantomData<fn(&Data) -> &ButtonState>,
}
struct ButtonViewLayout<'button, Data, GetButtonData: Lens<Data, ButtonState>, Callback: Fn(&crate::App, &mut Data)> {
    view: &'button ButtonView<Data, GetButtonData, Callback>,
    size: graphics::Vector2f,
}

impl<Data, GetButtonData: Lens<Data, ButtonState>, Callback: Fn(&crate::App, &mut Data)> ViewWithoutLayout<Data> for ButtonView<Data, GetButtonData, Callback> {
    type WithLayout<'without_layout> = ButtonViewLayout<'without_layout, Data, GetButtonData, Callback> where Data: 'without_layout, GetButtonData: 'without_layout, Callback: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        // TODO: move size to constants in the theme
        ButtonViewLayout { view: self, size: sc.clamp_size(graphics::Vector2f::new(150.0, 25.0)) }
    }
}
impl<Data, GetButtonData: Lens<Data, ButtonState>, Callback: Fn(&crate::App, &mut Data)> View<Data> for ButtonViewLayout<'_, Data, GetButtonData, Callback> {
    fn draw_inner(&self, _: &crate::App, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<ViewId>) {
        let mut rect_shape = graphics::RectangleShape::from_rect(graphics::FloatRect::from_vecs(top_left, self.size));
        rect_shape.set_fill_color(Theme::DEFAULT.button_normal_bg);

        if hover == Some(self.view.id) {
            rect_shape.set_fill_color(Theme::DEFAULT.button_hover_bg);
        }
        if self.view.pressed {
            rect_shape.set_fill_color(Theme::DEFAULT.button_pressed_bg);
        }

        target.draw(&rect_shape);
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<ViewId> {
        if graphics::FloatRect::from_vecs(top_left, self.size).contains(mouse) {
            Some(self.view.id)
        } else {
            None
        }
    }

    fn size(&self) -> graphics::Vector2f {
        self.size
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.view.id {
            self.targeted_event(app, data, event);
        }
    }
    fn targeted_event(&self, _: &crate::App, data: &mut Data, event: TargetedEvent) {
        match event {
            TargetedEvent::LeftMouseDown(_) => self.view.button_data_lens.with_mut(data, |button_data| button_data.pressed = true),
        }
    }
    fn general_event(&self, app: &crate::App, data: &mut Data, event: GeneralEvent) {
        match event {
            GeneralEvent::LeftMouseUp => {
                if self.view.pressed {
                    self.view.button_data_lens.with_mut(data, |button_data| {
                        if button_data.pressed {
                            button_data.pressed = false;
                        }
                    });
                    (self.view.callback)(app, data);
                }
            }
            GeneralEvent::MouseMoved(_) => {}
        }
    }
}

// TODO: should this return ButtonView instead of an opaque type?
pub(crate) fn button<Data>(id_maker: &mut ViewIdMaker, data: &Data, get_button_data: impl Lens<Data, ButtonState>, callback: impl Fn(&crate::App, &mut Data)) -> impl ViewWithoutLayout<Data> {
    ButtonView { id: id_maker.next_id(), pressed: get_button_data.with(data, |button_data| button_data.pressed), button_data_lens: get_button_data, callback, _phantom: PhantomData }
}
