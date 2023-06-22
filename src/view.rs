use sfml::graphics::{RenderTarget, Transformable};

use crate::graphics;

pub(crate) mod id;
pub(crate) mod lens;

#[derive(Copy, Clone)]
pub(crate) enum TargetedEvent {
    LeftMouseDown(graphics::Vector2f),
}
#[derive(Copy, Clone)]
pub(crate) enum GeneralEvent {
    MouseMoved(graphics::Vector2f),
    LeftMouseUp,
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) struct SizeConstraints {
    pub(crate) min: graphics::Vector2f,
    pub(crate) max: graphics::Vector2f,
}
impl SizeConstraints {
    pub(crate) fn with_no_min(&self) -> SizeConstraints {
        SizeConstraints { min: graphics::Vector2f::new(0.0, 0.0), max: self.max }
    }

    pub(crate) fn clamp_size(&self, size: graphics::Vector2f) -> graphics::Vector2f {
        graphics::Vector2f::new(size.x.clamp(self.min.x, self.max.x), size.y.clamp(self.min.y, self.max.y))
    }
}

// new view system heavilty inspired by xilem
// specifically this blog post: https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html
// kind of like a merge of the old Widget and old Drawing trait
pub(crate) trait View<Data> {
    // top_left in draw_inner and draw does not necessarily correspond to window coordinates, but top_left in find_hover always does
    fn draw(&self, app: &crate::App, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<id::ViewId>) {
        let mut sub_graphics =
            graphics::RenderTexture::with_settings(self.size().x.ceil() as u32, self.size().y.ceil() as u32, &crate::App::default_render_context_settings()).expect("could not create render texture");

        sub_graphics.set_active(true);
        self.draw_inner(app, &mut sub_graphics, graphics::Vector2f::new(0.0, 0.0), hover);
        sub_graphics.set_active(false);
        sub_graphics.display();

        let mut sprite = graphics::Sprite::new();
        sprite.set_texture(sub_graphics.texture(), true);
        sprite.set_position(top_left);
        target.draw(&sprite);
    }
    fn draw_inner(&self, app: &crate::App, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<id::ViewId>);
    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<id::ViewId>;
    fn size(&self) -> graphics::Vector2f;

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: id::ViewId, event: TargetedEvent);
    fn targeted_event(&self, app: &crate::App, data: &mut Data, event: TargetedEvent);
    fn general_event(&self, app: &crate::App, data: &mut Data, event: GeneralEvent);
}
pub(crate) trait ViewWithoutLayout<Data> {
    type WithLayout<'without_layout>: View<Data>
    where
        Self: 'without_layout;
    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_>;
}

pub(crate) fn render(app: &crate::App, window: &mut graphics::RenderWindow, logic_gates: &crate::LogicGates) {
    let view_center = graphics::Vector2f::new(0.0, 0.0);
    let size_constraints = SizeConstraints { min: graphics::Vector2f::new(0.0, 0.0), max: window.size().as_other() };

    let view_without_layout = crate::view(app, logic_gates);
    let view_with_layout = view_without_layout.layout(size_constraints);

    let mouse_position = window.mouse_position().as_other();
    let hover = view_with_layout.find_hover(view_center, mouse_position);
    view_with_layout.draw(app, window, view_center, hover);
}

pub(crate) fn event(app: &crate::App, window: &graphics::RenderWindow, logic_gates: &mut crate::LogicGates, event: sfml::window::Event) {
    let view_center = graphics::Vector2f::new(0.0, 0.0);
    let size_constraints = SizeConstraints { min: graphics::Vector2f::new(0.0, 0.0), max: window.size().as_other() };

    let view_without_layout = crate::view(app, logic_gates);
    let view_with_layout = view_without_layout.layout(size_constraints);

    match event {
        sfml::window::Event::MouseButtonPressed { button: sfml::window::mouse::Button::Left, x, y } => {
            let mouse_position = graphics::Vector2f::new(x as f32, y as f32); // TODO: clean up casts (also clean up in rest of module too)
            let hovered = view_with_layout.find_hover(view_center, mouse_position);
            if let Some(hovered) = hovered {
                view_with_layout.send_targeted_event(app, logic_gates, hovered, TargetedEvent::LeftMouseDown(mouse_position));
            }
        }

        sfml::window::Event::MouseMoved { x, y } => view_with_layout.general_event(app, logic_gates, GeneralEvent::MouseMoved(graphics::Vector2f::new(x as f32, y as f32))), // TODO: change the event to accept 2 i32s

        sfml::window::Event::MouseButtonReleased { button: sfml::window::mouse::Button::Left, x: _, y: _ } => view_with_layout.general_event(app, logic_gates, GeneralEvent::LeftMouseUp),

        _ => {}
    }
}
