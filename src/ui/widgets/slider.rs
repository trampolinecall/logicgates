use std::{fmt::Display, marker::PhantomData, ops::Add};

use crate::{
    theme::Theme,
    view::{
        id::{ViewId, ViewIdMaker},
        lens::Lens,
        GeneralEvent, TargetedEvent, View,
    },
};

pub(crate) struct SliderState<Value: Display> {
    drag_start: Option<(nannou::geom::Vec2, Value)>,
}

struct SliderView<Data, Value: Display + Copy + Add<Value, Output = Value> + Ord, StateLens: Lens<Data, SliderState<Value>>, DataLens: Lens<Data, Value>, ConvertMousePosition: Fn(f32) -> Value> {
    id: ViewId,

    min: Option<Value>,
    max: Option<Value>,
    value: Value,

    pressed: bool,

    state_lens: StateLens,
    value_lens: DataLens,

    convert_mouse_position: ConvertMousePosition,

    _phantom: PhantomData<(fn(&Data) -> &SliderState<Value>, fn(&Data) -> &Value)>,
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
    data: &Data,
) -> impl View<Data> {
    let pressed = state_lens.with(data, |slider_state| slider_state.drag_start.is_some());
    let value = value_lens.with(data, |v| *v);
    SliderView { id: id_maker.next_id(), min, max, value, pressed, state_lens, value_lens, convert_mouse_position, _phantom: PhantomData }
}

impl<Data, Value: Display + Copy + Add<Value, Output = Value> + Ord, StateLens: Lens<Data, SliderState<Value>>, DataLens: Lens<Data, Value>, ConvertMousePosition: Fn(f32) -> Value> View<Data>
    for SliderView<Data, Value, StateLens, DataLens, ConvertMousePosition>
{
    fn draw(&self, _: &nannou::App, draw: &nannou::Draw, rect: nannou::geom::Rect, hover: Option<ViewId>) {
        // TODO: show as progress bar if both min and max
        let mut background_rect = draw.rect().xy(rect.xy()).wh(rect.wh()).color(Theme::DEFAULT.button_normal_bg);
        let mut text = draw.text(&self.value.to_string()).xy(rect.xy()).wh(rect.wh()).center_justify().align_text_middle_y().color(Theme::DEFAULT.button_normal_fg);
        if Some(self.id) == hover {
            background_rect = background_rect.color(Theme::DEFAULT.button_hover_bg);
            text = text.color(Theme::DEFAULT.button_hover_fg);
        }
        if self.pressed {
            background_rect = background_rect.color(Theme::DEFAULT.button_pressed_bg);
            text = text.color(Theme::DEFAULT.button_pressed_fg);
        }

        background_rect.finish();
        text.finish();
    }

    fn find_hover(&self, rect: nannou::geom::Rect, mouse: nannou::geom::Vec2) -> Option<ViewId> {
        if rect.contains(mouse) {
            Some(self.id)
        } else {
            None
        }
    }

    fn size(&self, _: (f32, f32)) -> (f32, f32) {
        (100.0, 15.0) // TODO: put this in theme?, also TODO: clamp to given space
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.id {
            self.targeted_event(app, data, event);
        }
    }

    fn targeted_event(&self, app: &nannou::App, data: &mut Data, event: TargetedEvent) {
        match event {
            TargetedEvent::LeftMouseDown => {
                let mouse_pos = app.mouse.position();
                let cur_value = self.value_lens.with(data, |value| *value);
                self.state_lens.with_mut(data, |state| state.drag_start = Some((mouse_pos, cur_value)));
            }
        }
    }
    fn general_event(&self, _: &nannou::App, data: &mut Data, event: GeneralEvent) {
        if self.pressed {
            match event {
                GeneralEvent::MouseMoved(new_mouse_pos) => {
                    let new_value = self.state_lens.with_mut(data, |state| {
                        if let Some((drag_start_mouse_pos, start_value)) = state.drag_start {
                            let diff = (self.convert_mouse_position)(new_mouse_pos.x - drag_start_mouse_pos.x); // TODO: scale this, also with modifier keys
                            let mut new_value = start_value + diff;
                            if let Some(min) = self.min {
                                new_value = new_value.max(min);
                            }
                            if let Some(max) = self.max {
                                new_value = new_value.min(max);
                            }
                            Some(new_value)
                        } else {
                            None
                        }
                    });
                    if let Some(new_value) = new_value {
                        self.value_lens.with_mut(data, |value| *value = new_value);
                    }
                }
                GeneralEvent::LeftMouseUp => self.state_lens.with_mut(data, |state| {
                    if state.drag_start.is_some() {
                        state.drag_start = None;
                    }
                }),
            }
        }
    }
}
