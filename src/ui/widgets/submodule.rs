use std::marker::PhantomData;

use crate::view::{id::ViewId, lens::Lens, GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout};

struct SubmoduleView<Data, SubData, L: Lens<Data, SubData>, SubView: ViewWithoutLayout<SubData>> {
    lens: L,

    subview: SubView,

    _phantom: PhantomData<fn(&Data) -> &SubData>,
}
struct SubmoduleLayout<'submodule, Data, SubData, L: Lens<Data, SubData>, SubView: ViewWithoutLayout<SubData>> {
    submodule: &'submodule SubmoduleView<Data, SubData, L, SubView>,
    subview: SubView::WithLayout<'submodule>,
}
impl<Data, SubData, L: Lens<Data, SubData>, SubView: ViewWithoutLayout<SubData>> ViewWithoutLayout<Data> for SubmoduleView<Data, SubData, L, SubView> {
    type WithLayout<'without_layout> = SubmoduleLayout<'without_layout, Data, SubData, L, SubView>where Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        SubmoduleLayout { submodule: self, subview: self.subview.layout(sc) }
    }
}
impl<Data, SubData, L: Lens<Data, SubData>, SubView: ViewWithoutLayout<SubData>> View<Data> for SubmoduleLayout<'_, Data, SubData, L, SubView> {
    fn draw(&self, app: &crate::App, target: &mut dyn sfml::graphics::RenderTarget, top_left: sfml::system::Vector2f, hover: Option<ViewId>) {
        self.subview.draw(app, target, top_left, hover);
    }

    fn draw_inner(&self, app: &crate::App, target: &mut dyn sfml::graphics::RenderTarget, top_left: sfml::system::Vector2f, hover: Option<ViewId>) {
        self.subview.draw_inner(app, target, top_left, hover);
    }

    fn find_hover(&self, top_left: sfml::system::Vector2f, mouse: sfml::system::Vector2f) -> Option<ViewId> {
        self.subview.find_hover(top_left, mouse)
    }

    fn size(&self) -> sfml::system::Vector2f {
        self.subview.size()
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        self.submodule.lens.with_mut(data, |subdata| self.subview.send_targeted_event(app, subdata, target, event));
    }

    fn targeted_event(&self, app: &crate::App, data: &mut Data, event: TargetedEvent) {
        self.submodule.lens.with_mut(data, |subdata| self.subview.targeted_event(app, subdata, event));
    }

    fn general_event(&self, app: &crate::App, data: &mut Data, event: GeneralEvent) {
        self.submodule.lens.with_mut(data, |subdata| self.subview.general_event(app, subdata, event));
    }
}

pub(crate) fn submodule<Data, SubData>(lens: impl Lens<Data, SubData>, subview: impl ViewWithoutLayout<SubData>) -> impl ViewWithoutLayout<Data> {
    SubmoduleView { lens, subview, _phantom: PhantomData }
}
