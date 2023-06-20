use crate::view::{
    id::{ViewId, ViewIdMaker},
    GeneralEvent, TargetedEvent, View,
};

pub(crate) struct TestRectView {
    id: ViewId,
    color: nannou::color::Srgb,
    size: (f32, f32),
}

impl View<()> for TestRectView {
    fn draw(&self, _: &nannou::App, draw: &nannou::Draw, rect: nannou::geom::Rect, _: Option<ViewId>) {
        // TODO: use hovered?
        draw.rect().xy(rect.xy()).wh(rect.wh()).color(self.color);
    }

    fn find_hover(&self, rect: nannou::geom::Rect, mouse: nannou::prelude::Vec2) -> Option<ViewId> {
        if rect.contains(mouse) {
            Some(self.id)
        } else {
            None
        }
    }
    fn size(&self, _: (f32, f32)) -> (f32, f32) {
        // TODO: clamp to given size
        self.size
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
