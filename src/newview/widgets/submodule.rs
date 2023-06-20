use std::marker::PhantomData;

use crate::newview::{id::ViewId, lens::Lens, Event, Subscription, View};

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

    fn targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: Event) {
        self.subview.targeted_event(app, self.lens.get_mut(data), target, event)
    }

    fn event(&self, app: &nannou::App, data: &mut Data, event: Event) {
        self.subview.event(app, self.lens.get_mut(data), event)
    }

    fn subscriptions(&self) -> Vec<Subscription<Data>> {
        // TODO: is there a better way that involves less boxes?
        self.subview
            .subscriptions()
            .into_iter()
            .map(|subscription| match subscription {
                Subscription::MouseMoved(callback) => Subscription::MouseMoved(Box::new(move |app, bigger_data, mouse_pos| callback(app, self.lens.get_mut(bigger_data), mouse_pos))),
                Subscription::LeftMouseUp(callback) => Subscription::LeftMouseUp(Box::new(move |app, bigger_data| callback(app, self.lens.get_mut(bigger_data)))),
            })
            .collect()
    }
}

pub(crate) fn submodule<Data, SubData>(lens: impl Lens<Data, SubData>, subview: impl View<SubData>) -> impl View<Data> {
    SubmoduleView { lens, _phantom: PhantomData, subview }
}
