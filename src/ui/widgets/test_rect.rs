use crate::view::{
    id::{ViewId, ViewIdMaker},
    GeneralEvent, SizeConstraints, TargetedEvent, View,
};

pub(crate) struct TestRectView {
    id: ViewId,
    color: nannou::color::Srgb,
    size: (f32, f32),
}

impl View<()> for TestRectView {
    fn draw(&self, _: &nannou::App, draw: &nannou::Draw, center: nannou::geom::Vec2, sc: SizeConstraints, _: Option<ViewId>) {
        // TODO: use hovered?
        let rect = nannou::geom::Rect::from_xy_wh(center, self.size(sc));
        draw.rect().xy(rect.xy()).wh(rect.wh()).color(self.color);
    }

    fn find_hover(&self, center: nannou::geom::Vec2, sc: SizeConstraints, mouse: nannou::prelude::Vec2) -> Option<ViewId> {
        if nannou::geom::Rect::from_xy_wh(center, self.size(sc)).contains(mouse) {
            Some(self.id)
        } else {
            None
        }
    }
    fn size(&self, sc: SizeConstraints) -> nannou::geom::Vec2 {
        // TODO: clamp to given size
        nannou::geom::Vec2::from(self.size).clamp(sc.min, sc.max)
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut (), target: ViewId, event: TargetedEvent) {
        if target == self.id {
            self.targeted_event(app, data, event);
        }
    }

    fn targeted_event(&self, _: &nannou::App, _: &mut (), _: TargetedEvent) {}
    fn general_event(&self, _: &nannou::App, _: &mut (), _: GeneralEvent) {}
}

pub(crate) fn test_rect(id_maker: &mut ViewIdMaker, color: nannou::color::Srgb, size: (f32, f32)) -> impl View<()> {
    TestRectView { id: id_maker.next_id(), color, size }
}
