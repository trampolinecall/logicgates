use sfml::graphics::Shape;

use crate::{view::{
    id::{ViewId, ViewIdMaker},
    GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout,
}, graphics};

struct TestRect {
    id: ViewId,
    color: graphics::Color,
    size: (f32, f32),
}
struct TestRectLayout<'test_rect> {
    test_rect: &'test_rect TestRect,
    actual_size: graphics::Vector2f,
}

impl ViewWithoutLayout<()> for TestRect {
    type WithLayout<'without_layout> = TestRectLayout<'without_layout>;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        TestRectLayout { test_rect: self, actual_size: sc.clamp_size(graphics::Vector2f::from(self.size)) }
    }
}
impl View<()> for TestRectLayout<'_> {
    fn draw_inner(&self, _: &crate::App, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, _: Option<ViewId>) {
        // TODO: use hovered?
        let rect = graphics::FloatRect::from_vecs(top_left, self.actual_size);
        let mut rect_shape = graphics::RectangleShape::from_rect(rect);
        rect_shape.set_fill_color(self.test_rect.color);
        target.draw(&rect_shape);
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<ViewId> {
        if graphics::FloatRect::from_vecs(top_left, self.actual_size).contains(mouse) {
            Some(self.test_rect.id)
        } else {
            None
        }
    }
    fn size(&self) -> graphics::Vector2f {
        self.actual_size
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut (), target: ViewId, event: TargetedEvent) {
        if target == self.test_rect.id {
            self.targeted_event(app, data, event);
        }
    }

    fn targeted_event(&self, _: &crate::App, _: &mut (), _: TargetedEvent) {}
    fn general_event(&self, _: &crate::App, _: &mut (), _: GeneralEvent) {}
}

pub(crate) fn test_rect(id_maker: &mut ViewIdMaker, color: graphics::Color, size: (f32, f32)) -> impl ViewWithoutLayout<()> {
    TestRect { id: id_maker.next_id(), color, size }
}
