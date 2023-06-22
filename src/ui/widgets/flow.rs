#[macro_use]
pub(crate) mod flow_macro;
pub(crate) mod layout {
    use crate::{
        graphics,
        ui::widgets::flow::Direction,
        view::{SizeConstraints, View},
    };

    pub(crate) fn child_sc(sc: SizeConstraints) -> SizeConstraints {
        sc.with_no_min()
    }
    pub(crate) fn find_own_size<'i, Data: 'i>(direction: Direction, sc: SizeConstraints, children: impl IntoIterator<Item = &'i (dyn View<Data> + 'i)>) -> graphics::Vector2f {
        {
            sc.clamp_size(graphics::Vector2f::from(children.into_iter().fold((0.0, 0.0), |(x_acc, y_acc), child| {
                match direction {
                    Direction::Horizontal => {
                        // sum x, take max of y
                        let x_sum = x_acc + child.size().x;
                        let max_y = if child.size().y > y_acc { child.size().y } else { y_acc };
                        (x_sum, max_y)
                    }
                    Direction::Vertical => {
                        // take max of x, sum y
                        let max_x = if child.size().x > x_acc { child.size().x } else { x_acc };
                        let y_sum = y_acc + child.size().y;
                        (max_x, y_sum)
                    }
                }
            })))
        }
    }
    pub(crate) fn layout_step<Data>(direction: Direction, cur_pos: &mut f32, child: &dyn View<Data>) -> graphics::Vector2f {
        match direction {
            Direction::Horizontal => {
                let pos = graphics::Vector2f::new(*cur_pos, 0.0);
                *cur_pos += child.size().x;
                pos
            }
            Direction::Vertical => {
                let pos = graphics::Vector2f::new(0.0, *cur_pos);
                *cur_pos += child.size().y;
                pos
            }
        }
    }
}

use crate::{
    graphics,
    view::{id::ViewId, GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout},
};

// this is kind of a hack but ViewWithoutLayout cannot be used as a trait object because it has the associated type
pub(crate) trait ViewLayoutIntoBoxView<'s, Data> {
    fn layout(&'s self, sc: SizeConstraints) -> Box<dyn View<Data> + 's>;
}
impl<'s, T: ViewWithoutLayout<Data>, Data: 's> ViewLayoutIntoBoxView<'s, Data> for T {
    fn layout(&'s self, sc: SizeConstraints) -> Box<dyn View<Data> + 's> {
        Box::new(self.layout(sc)) as Box<dyn View<_>>
    }
}

struct FlowView<Data> {
    direction: Direction,
    children: Vec<Box<dyn for<'a> ViewLayoutIntoBoxView<'a, Data>>>,
}
struct FlowLayout<'original, Data> {
    own_size: graphics::Vector2f,
    children: Vec<(graphics::Vector2f, Box<dyn View<Data> + 'original>)>,
}
#[derive(Copy, Clone)]
pub(crate) enum Direction {
    Horizontal,
    Vertical,
}

pub(crate) fn horizontal_flow<Data>(children: Vec<Box<dyn for<'s> ViewLayoutIntoBoxView<'s, Data>>>) -> impl ViewWithoutLayout<Data> {
    flow(Direction::Horizontal, children)
}
pub(crate) fn vertical_flow<Data>(children: Vec<Box<dyn for<'s> ViewLayoutIntoBoxView<'s, Data>>>) -> impl ViewWithoutLayout<Data> {
    flow(Direction::Vertical, children)
}
pub(crate) fn flow<Data>(direction: Direction, children: Vec<Box<dyn for<'s> ViewLayoutIntoBoxView<'s, Data>>>) -> impl ViewWithoutLayout<Data> {
    FlowView { children, direction }
}

impl<Data> ViewWithoutLayout<Data> for FlowView<Data> {
    type WithLayout<'without_layout>  = FlowLayout<'without_layout, Data> where Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        let children: Vec<_> = self.children.iter().map(|child| child.layout(layout::child_sc(sc))).collect();

        let own_size = layout::find_own_size(self.direction, sc, children.iter().map(|child| &**child));

        let mut cur_pos = 0.0;
        let children = children.into_iter().map(|child| (layout::layout_step(self.direction, &mut cur_pos, &*child), child)).collect();

        FlowLayout { own_size, children }
    }
}
impl<Data> View<Data> for FlowLayout<'_, Data> {
    fn draw_inner(&self, app: &crate::App, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<ViewId>) {
        for (child_offset, child) in self.children.iter() {
            child.draw(app, target, top_left + *child_offset, hover);
        }
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<ViewId> {
        for (child_offset, child) in self.children.iter() {
            if let x @ Some(_) = child.find_hover(top_left + *child_offset, mouse) {
                return x;
            }
        }
        None
    }

    fn size(&self) -> graphics::Vector2f {
        self.own_size
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        for (_, child) in &self.children {
            child.send_targeted_event(app, data, target, event);
        }
    }

    fn targeted_event(&self, _: &crate::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, app: &crate::App, data: &mut Data, event: GeneralEvent) {
        for (_, child) in &self.children {
            child.general_event(app, data, event);
        }
    }
}
