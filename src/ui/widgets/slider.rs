use std::{fmt::Display, marker::PhantomData, ops::Add, rc::Rc};

use sfml::graphics::{Shape, Transformable};

use crate::{
    graphics::{self, CenterText, RectCenter},
    theme::Theme,
    view::{
        id::{ViewId, ViewIdMaker},
        lens::Lens,
        GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout,
    },
};

pub(crate) struct SliderState<Value: Display> {
    drag_start: Option<(graphics::Vector2f, Value)>,
}

// TODO: type in values
struct SliderView<Data, Value: Display + Copy + Add<Value, Output = Value> + Ord, StateLens: Lens<Data, SliderState<Value>>, DataLens: Lens<Data, Value>, ConvertMousePosition: Fn(f32) -> Value> {
    id: ViewId,

    min: Option<Value>,
    max: Option<Value>,
    value: Value,

    pressed: bool,

    state_lens: StateLens,
    value_lens: DataLens,

    convert_mouse_position: ConvertMousePosition,

    font: Rc<sfml::SfBox<graphics::Font>>,

    _phantom: PhantomData<(fn(&Data) -> &SliderState<Value>, fn(&Data) -> &Value)>,
}
struct SliderLayout<
    'slider,
    Data,
    Value: Display + Copy + Add<Value, Output = Value> + Ord,
    StateLens: Lens<Data, SliderState<Value>>,
    DataLens: Lens<Data, Value>,
    ConvertMousePosition: Fn(f32) -> Value,
> {
    slider: &'slider SliderView<Data, Value, StateLens, DataLens, ConvertMousePosition>,
    size: graphics::Vector2f,
}

impl<Value: Display> SliderState<Value> {
    pub(crate) fn new() -> SliderState<Value> {
        SliderState { drag_start: None }
    }
}

// TODO: find a more consistent order for the arguments of all of these view creating functions
pub(crate) fn slider<Data, Value: Display + Copy + Add<Value, Output = Value> + Ord>(
    id_maker: &mut ViewIdMaker,
    min: Option<Value>,
    max: Option<Value>,
    state_lens: impl Lens<Data, SliderState<Value>>,
    value_lens: impl Lens<Data, Value>,
    convert_mouse_position: impl Fn(f32) -> Value,
    font: &Rc<sfml::SfBox<graphics::Font>>,
    data: &Data,
) -> impl ViewWithoutLayout<Data> {
    let pressed = state_lens.with(data, |slider_state| slider_state.drag_start.is_some());
    let value = value_lens.with(data, |v| *v);
    SliderView { id: id_maker.next_id(), min, max, value, pressed, state_lens, value_lens, convert_mouse_position, _phantom: PhantomData, font: font.clone() }
}

impl<Data, Value: Display + Copy + Add<Value, Output = Value> + Ord, StateLens: Lens<Data, SliderState<Value>>, DataLens: Lens<Data, Value>, ConvertMousePosition: Fn(f32) -> Value>
    ViewWithoutLayout<Data> for SliderView<Data, Value, StateLens, DataLens, ConvertMousePosition>
{
    type WithLayout<'without_layout> = SliderLayout<'without_layout, Data, Value, StateLens, DataLens, ConvertMousePosition>where Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        SliderLayout {
            slider: self,
            size: sc.clamp_size(graphics::Vector2f::new(100.0, 15.0)), // TODO: put this in theme?
        }
    }
}
impl<Data, Value: Display + Copy + Add<Value, Output = Value> + Ord, StateLens: Lens<Data, SliderState<Value>>, DataLens: Lens<Data, Value>, ConvertMousePosition: Fn(f32) -> Value> View<Data>
    for SliderLayout<'_, Data, Value, StateLens, DataLens, ConvertMousePosition>
{
    fn draw_inner(&self, _: &crate::App, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<ViewId>) {
        // TODO: show as progress bar if both min and max
        let rect = graphics::FloatRect::from_vecs(top_left, self.size);

        let mut background_rect = graphics::RectangleShape::from_rect(rect);
        let mut text = graphics::Text::new(&self.slider.value.to_string(), &self.slider.font, 10); // TODO: also put this font size into the theme as well

        text.center();
        text.set_position(rect.center());

        if self.slider.pressed {
            background_rect.set_fill_color(Theme::DEFAULT.button_pressed_bg);
            text.set_fill_color(Theme::DEFAULT.button_pressed_fg);
        } else if Some(self.slider.id) == hover {
            background_rect.set_fill_color(Theme::DEFAULT.button_hover_bg);
            text.set_fill_color(Theme::DEFAULT.button_hover_fg);
        } else {
            background_rect.set_fill_color(Theme::DEFAULT.button_normal_bg);
            text.set_fill_color(Theme::DEFAULT.button_normal_fg);
        }

        target.draw(&background_rect);
        target.draw(&text);
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<ViewId> {
        if graphics::FloatRect::from_vecs(top_left, self.size).contains(mouse) {
            Some(self.slider.id)
        } else {
            None
        }
    }

    fn size(&self) -> graphics::Vector2f {
        self.size
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.slider.id {
            self.targeted_event(app, data, event);
        }
    }

    fn targeted_event(&self, _: &crate::App, data: &mut Data, event: TargetedEvent) {
        match event {
            TargetedEvent::LeftMouseDown(mouse_pos) => {
                let cur_value = self.slider.value_lens.with(data, |value| *value);
                self.slider.state_lens.with_mut(data, |state| state.drag_start = Some((mouse_pos, cur_value)));
            }
            TargetedEvent::RightMouseDown(_) => {}
        }
    }
    fn general_event(&self, _: &crate::App, data: &mut Data, event: GeneralEvent) {
        if self.slider.pressed {
            match event {
                GeneralEvent::MouseMoved(new_mouse_pos) => {
                    let new_value = self.slider.state_lens.with_mut(data, |state| {
                        if let Some((drag_start_mouse_pos, start_value)) = state.drag_start {
                            let diff = (self.slider.convert_mouse_position)(new_mouse_pos.x - drag_start_mouse_pos.x); // TODO: scale this, also with modifier keys
                            let mut new_value = start_value + diff;
                            if let Some(min) = self.slider.min {
                                new_value = new_value.max(min);
                            }
                            if let Some(max) = self.slider.max {
                                new_value = new_value.min(max);
                            }
                            Some(new_value)
                        } else {
                            None
                        }
                    });
                    if let Some(new_value) = new_value {
                        self.slider.value_lens.with_mut(data, |value| *value = new_value);
                    }
                }
                GeneralEvent::LeftMouseUp => self.slider.state_lens.with_mut(data, |state| {
                    if state.drag_start.is_some() {
                        state.drag_start = None;
                    }
                }),
            }
        }
    }
}
