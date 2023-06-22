use crate::view::{
    id::{ViewId, ViewIdMaker},
    GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout,
};

struct TestRect {
    id: ViewId,
    color: sfml::graphics::Color,
    size: (f32, f32),
}
struct TestRectLayout<'test_rect> {
    test_rect: &'test_rect TestRect,
    actual_size: sfml::system::Vector2f,
}

impl ViewWithoutLayout<()> for TestRect {
    type WithLayout<'without_layout> = TestRectLayout<'without_layout>;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        TestRectLayout { test_rect: self, actual_size: sc.clamp_size(sfml::system::Vector2f::from(self.size)) }
    }
}
impl View<()> for TestRectLayout<'_> {
    fn draw_inner(&self, _: &crate::App, target: &mut dyn sfml::graphics::RenderTarget, top_left: sfml::system::Vector2f, _: Option<ViewId>) {
        use sfml::graphics::Shape;
        // TODO: use hovered?
        let rect = sfml::graphics::FloatRect::from_vecs(top_left, self.actual_size);
        let mut rect_shape = sfml::graphics::RectangleShape::from_rect(rect);
        rect_shape.set_fill_color(self.test_rect.color);
        target.draw(&rect_shape);
    }

    fn find_hover(&self, top_left: sfml::system::Vector2f, mouse: sfml::system::Vector2f) -> Option<ViewId> {
        if sfml::graphics::FloatRect::from_vecs(top_left, self.actual_size).contains(mouse) {
            Some(self.test_rect.id)
        } else {
            None
        }
    }
    fn size(&self) -> sfml::system::Vector2f {
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

pub(crate) fn test_rect(id_maker: &mut ViewIdMaker, color: sfml::graphics::Color, size: (f32, f32)) -> impl ViewWithoutLayout<()> {
    TestRect { id: id_maker.next_id(), color, size }
}
