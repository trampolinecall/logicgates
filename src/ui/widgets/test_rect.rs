use crate::view::{
    id::{ViewId, ViewIdMaker},
    GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout,
};

struct TestRectView {
    id: ViewId,
    color: nannou::color::Srgb,
    size: (f32, f32),
}
struct TestRectViewLayout<'test_rect> {
    test_rect: &'test_rect TestRectView,
    actual_size: nannou::geom::Vec2,
}

impl ViewWithoutLayout<()> for TestRectView {
    type WithLayout<'without_layout> = TestRectViewLayout<'without_layout>;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        TestRectViewLayout { test_rect: self, actual_size: nannou::geom::Vec2::from(self.size).clamp(sc.min, sc.max) }
    }
}
impl View<()> for TestRectViewLayout<'_> {
    fn draw(&self, _: &nannou::App, draw: &nannou::Draw, center: nannou::geom::Vec2, _: Option<ViewId>) {
        // TODO: use hovered?
        let rect = nannou::geom::Rect::from_xy_wh(center, self.actual_size);
        draw.rect().xy(rect.xy()).wh(rect.wh()).color(self.test_rect.color);
    }

    fn find_hover(&self, center: nannou::geom::Vec2, mouse: nannou::prelude::Vec2) -> Option<ViewId> {
        if nannou::geom::Rect::from_xy_wh(center, self.actual_size).contains(mouse) {
            Some(self.test_rect.id)
        } else {
            None
        }
    }
    fn size(&self) -> nannou::geom::Vec2 {
        self.actual_size
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut (), target: ViewId, event: TargetedEvent) {
        if target == self.test_rect.id {
            self.targeted_event(app, data, event);
        }
    }

    fn targeted_event(&self, _: &nannou::App, _: &mut (), _: TargetedEvent) {}
    fn general_event(&self, _: &nannou::App, _: &mut (), _: GeneralEvent) {}
}

pub(crate) fn test_rect(id_maker: &mut ViewIdMaker, color: nannou::color::Srgb, size: (f32, f32)) -> impl ViewWithoutLayout<()> {
    TestRectView { id: id_maker.next_id(), color, size }
}
