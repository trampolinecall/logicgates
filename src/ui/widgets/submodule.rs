use std::marker::PhantomData;

use crate::view::{id::ViewId, lens::Lens, GeneralEvent, TargetedEvent, View};

pub(crate) struct SubmoduleView<Data, SubData, L: Lens<Data, SubData>, SubView: View<SubData>> {
    lens: L,

    subview: SubView,

    _phantom: PhantomData<fn(&Data) -> &SubData>,
}
impl<Data, SubData, L: Lens<Data, SubData>, SubView: View<SubData>> View<Data> for SubmoduleView<Data, SubData, L, SubView> {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, rect: nannou::geom::Rect, hover: Option<ViewId>) {
        self.subview.draw(app, draw, rect, hover)
    }

    fn find_hover(&self, rect: nannou::geom::Rect, mouse: nannou::geom::Vec2) -> Option<ViewId> {
        self.subview.find_hover(rect, mouse)
    }

    fn size(&self, given: (f32, f32)) -> (f32, f32) {
        self.subview.size(given)
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        self.lens.with_mut(data, |subdata| self.subview.send_targeted_event(app, subdata, target, event))
    }

    fn targeted_event(&self, app: &nannou::App, data: &mut Data, event: TargetedEvent) {
        self.lens.with_mut(data, |subdata| self.subview.targeted_event(app, subdata, event))
    }

    fn general_event(&self, app: &nannou::App, data: &mut Data, event: GeneralEvent) {
        self.lens.with_mut(data, |subdata| self.subview.general_event(app, subdata, event))
    }
}

pub(crate) fn submodule<Data, SubData>(lens: impl Lens<Data, SubData>, subview: impl View<SubData>) -> impl View<Data> {
    SubmoduleView { lens, _phantom: PhantomData, subview }
}
