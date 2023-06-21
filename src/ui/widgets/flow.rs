use nannou::geom::Vec2;

use crate::view::{id::ViewId, layout_cache::LayoutCache, GeneralEvent, SizeConstraints, TargetedEvent, View};

// TODO: turn this into a macro?
struct FlowView<Data> {
    direction: Direction,
    children: Vec<Box<dyn View<Data>>>,
    layout: LayoutCache<FlowLayout>,
}
struct FlowLayout {
    own_size: Vec2,
    child_layouts: Vec<(Vec2, SizeConstraints)>,
}
enum Direction {
    Horizontal,
    Vertical,
}

impl<Data> FlowView<Data> {
    fn layout(&self, sc: SizeConstraints) -> FlowLayout {
        let child_sc = sc.with_no_min();

        let child_sizes = self.children.iter().map(|child| child.size(child_sc));

        let own_size: Vec2 = child_sizes
            .clone()
            .fold((0.0, 0.0), |(x_acc, y_acc), cur_size| {
                match self.direction {
                    Direction::Horizontal => {
                        // sum x, take max of y
                        let x_sum = x_acc + cur_size.x;
                        let max_y = if cur_size.y > y_acc { cur_size.y } else { y_acc };
                        (x_sum, max_y)
                    }
                    Direction::Vertical => {
                        // take max of x, sum y
                        let max_x = if cur_size.x > x_acc { cur_size.x } else { x_acc };
                        let y_sum = y_acc + cur_size.y;
                        (max_x, y_sum)
                    }
                }
            })
            .into();
        let own_size = own_size.clamp(sc.min, sc.max);

        let mut cur_pos = match self.direction {
            Direction::Horizontal => -own_size.x / 2.0,
            Direction::Vertical => own_size.y / 2.0,
        };

        let child_layouts = child_sizes
            .map(|child_size| {
                let child_xy = match self.direction {
                    Direction::Horizontal => {
                        let pos = Vec2::new(cur_pos + child_size.x / 2.0, 0.0);
                        cur_pos += child_size.x;
                        pos
                    }
                    Direction::Vertical => {
                        let pos = Vec2::new(0.0, cur_pos - child_size.y / 2.0);
                        cur_pos -= child_size.y;
                        pos
                    }
                };
                (child_xy, child_sc)
            })
            .collect();

        FlowLayout { child_layouts, own_size }
    }
}

impl<Data> View<Data> for FlowView<Data> {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, center: Vec2, size_constraints: SizeConstraints, hover: Option<ViewId>) {
        self.layout.with_layout(
            size_constraints,
            |sc| self.layout(sc),
            |layout| {
                assert_eq!(layout.child_layouts.len(), self.children.len());

                for ((child_offset, child_sc), child) in layout.child_layouts.iter().zip(&self.children) {
                    child.draw(app, draw, center + *child_offset, *child_sc, hover);
                }
            },
        );
    }

    fn find_hover(&self, center: Vec2, sc: SizeConstraints, mouse: Vec2) -> Option<ViewId> {
        self.layout.with_layout(
            sc,
            |sc| self.layout(sc),
            |layout| {
                assert_eq!(layout.child_layouts.len(), self.children.len());

                for ((child_offset, child_sc), child) in layout.child_layouts.iter().zip(&self.children) {
                    if let x @ Some(_) = child.find_hover(center + *child_offset, *child_sc, mouse) {
                        return x;
                    }
                }
                None
            },
        )
    }

    fn size(&self, given: SizeConstraints) -> Vec2 {
        self.layout.with_layout(given, |sc| self.layout(sc), |layout| layout.own_size)
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        for child in &self.children {
            child.send_targeted_event(app, data, target, event);
        }
    }

    fn targeted_event(&self, _: &nannou::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, app: &nannou::App, data: &mut Data, event: GeneralEvent) {
        for child in &self.children {
            child.general_event(app, data, event);
        }
    }
}

pub(crate) fn horizontal_flow<Data>(children: Vec<Box<dyn View<Data>>>) -> impl View<Data> {
    flow(Direction::Horizontal, children)
}
pub(crate) fn vertical_flow<Data>(children: Vec<Box<dyn View<Data>>>) -> impl View<Data> {
    flow(Direction::Vertical, children)
}
fn flow<Data>(direction: Direction, children: Vec<Box<dyn View<Data>>>) -> impl View<Data> {
    FlowView { children, direction, layout: LayoutCache::new() }
}
