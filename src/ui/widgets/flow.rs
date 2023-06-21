#[macro_use]
pub(crate) mod flow_macro;
pub(crate) mod layout {
    use nannou::geom::Vec2;

    use crate::{
        ui::widgets::flow::Direction,
        view::{SizeConstraints, View},
    };

    pub(crate) fn child_sc(sc: SizeConstraints) -> SizeConstraints {
        sc.with_no_min()
    }
    pub(crate) fn find_own_size<'i, Data: 'static>(direction: Direction, sc: SizeConstraints, children: impl IntoIterator<Item = &'i dyn View<Data>>) -> Vec2 {
        let own_size: Vec2 = children
            .into_iter()
            .fold((0.0, 0.0), |(x_acc, y_acc), child| {
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
            })
            .into();
        let own_size = own_size.clamp(sc.min, sc.max);
        own_size
    }
    pub(crate) fn find_start_pos(direction: Direction, own_size: Vec2) -> f32 {
        match direction {
            Direction::Horizontal => -own_size.x / 2.0,
            Direction::Vertical => own_size.y / 2.0,
        }
    }
    pub(crate) fn layout_step<Data>(direction: Direction, cur_pos: &mut f32, child: &dyn View<Data>) -> Vec2 {
        match direction {
            Direction::Horizontal => {
                let pos = Vec2::new(*cur_pos + child.size().x / 2.0, 0.0);
                *cur_pos += child.size().x;
                pos
            }
            Direction::Vertical => {
                let pos = Vec2::new(0.0, *cur_pos - child.size().y / 2.0);
                *cur_pos -= child.size().y;
                pos
            }
        }
    }
}

use nannou::geom::Vec2;

use crate::view::{id::ViewId, GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout};

// this is kind of a hack but ViewWithoutLayout cannot be used as a trait object because it has the associated type
pub(crate) trait ViewLayoutIntoBoxView<Data> {
    fn layout(&self, sc: SizeConstraints) -> Box<dyn View<Data> + '_>;
}
impl<T: ViewWithoutLayout<Data>, Data: 'static> ViewLayoutIntoBoxView<Data> for T {
    fn layout<'a>(&'a self, sc: SizeConstraints) -> Box<dyn View<Data> + 'a> {
        Box::new(self.layout(sc)) as Box<dyn View<Data> + 'a>
    }
}

struct FlowView<Data> {
    direction: Direction,
    children: Vec<Box<dyn ViewLayoutIntoBoxView<Data>>>,
}
struct FlowLayout<'original, Data> {
    own_size: Vec2,
    children: Vec<(Vec2, Box<dyn View<Data> + 'original>)>,
}
#[derive(Copy, Clone)]
pub(crate) enum Direction {
    Horizontal,
    Vertical,
}

pub(crate) fn horizontal_flow<Data: 'static>(children: Vec<Box<dyn ViewLayoutIntoBoxView<Data>>>) -> impl ViewWithoutLayout<Data> {
    flow(Direction::Horizontal, children)
}
pub(crate) fn vertical_flow<Data: 'static>(children: Vec<Box<dyn ViewLayoutIntoBoxView<Data>>>) -> impl ViewWithoutLayout<Data> {
    flow(Direction::Vertical, children)
}
pub(crate) fn flow<Data: 'static>(direction: Direction, children: Vec<Box<dyn ViewLayoutIntoBoxView<Data>>>) -> impl ViewWithoutLayout<Data> {
    FlowView { children, direction }
}

impl<Data: 'static> ViewWithoutLayout<Data> for FlowView<Data> {
    type WithLayout<'without_layout>  = FlowLayout<'without_layout, Data> where Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        let children: Vec<_> = self.children.iter().map(|child| child.layout(layout::child_sc(sc))).collect();

        let own_size = layout::find_own_size(self.direction, sc, children.iter().map(|child| &**child));

        let mut cur_pos = layout::find_start_pos(self.direction, own_size);
        let children = children.into_iter().map(|child| (layout::layout_step(self.direction, &mut cur_pos, &*child), child)).collect();

        FlowLayout { own_size, children }
    }
}
impl<Data> View<Data> for FlowLayout<'_, Data> {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, center: Vec2, hover: Option<ViewId>) {
        for (child_offset, child) in self.children.iter() {
            child.draw(app, draw, center + *child_offset, hover);
        }
    }

    fn find_hover(&self, center: Vec2, mouse: Vec2) -> Option<ViewId> {
        for (child_offset, child) in self.children.iter() {
            if let x @ Some(_) = child.find_hover(center + *child_offset, mouse) {
                return x;
            }
        }
        None
    }

    fn size(&self) -> Vec2 {
        self.own_size
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        for (_, child) in &self.children {
            child.send_targeted_event(app, data, target, event);
        }
    }

    fn targeted_event(&self, _: &nannou::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, app: &nannou::App, data: &mut Data, event: GeneralEvent) {
        for (_, child) in &self.children {
            child.general_event(app, data, event);
        }
    }
}
