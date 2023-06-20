use std::cell::RefCell;

use crate::view::{
    id::{ViewId, ViewIdMaker},
    Event, Subscription, View,
};

// TODO: turn this into a macro?
struct FlowView<Data> {
    direction: Direction,
    children: Vec<Box<dyn View<Data>>>,
    layout: RefCell<Option<(nannou::geom::Rect, FlowLayout)>>, // TODO: factor out refcell into LayoutCache struct so that the caching logic can be shared between all the views that need it
}
struct FlowLayout {
    child_rects: Vec<nannou::geom::Rect>,
}
enum Direction {
    Horizontal,
    Vertical,
}

impl<Data> FlowView<Data> {
    fn layout<'layout>(&self, given_rect: nannou::geom::Rect, layout_field: &'layout mut Option<(nannou::geom::Rect, FlowLayout)>) -> &'layout FlowLayout {
        let needs_recompute = match layout_field {
            None => true,
            Some((old_given_rect, _)) if *old_given_rect != given_rect => true,
            _ => false,
        };

        if needs_recompute {
            // TODO: stay within bounds of rect
            let mut cur_pos = match self.direction {
                Direction::Horizontal => given_rect.left(),
                Direction::Vertical => given_rect.top(),
            };

            let child_rects = self
                .children
                .iter()
                .map(|child| {
                    let child_size = child.size(given_rect.w_h());
                    let child_xy = match self.direction {
                        Direction::Horizontal => {
                            let pos = nannou::geom::vec2(cur_pos + child_size.0 / 2.0, given_rect.y());
                            cur_pos += child_size.0;
                            pos
                        }
                        Direction::Vertical => {
                            let pos = nannou::geom::vec2(given_rect.x(), cur_pos - child_size.1 / 2.0);
                            cur_pos -= child_size.1;
                            pos
                        }
                    };
                    nannou::geom::Rect::from_xy_wh(child_xy, child_size.into())
                })
                .collect();
            *layout_field = Some((given_rect, FlowLayout { child_rects }));
        }

        &layout_field.as_ref().expect("layout was either already computed or just computed").1
    }
}

impl<Data> View<Data> for FlowView<Data> {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, rect: nannou::geom::Rect, hover: Option<ViewId>) {
        let mut layout_ref = self.layout.borrow_mut();
        let layout = self.layout(rect, &mut layout_ref);
        assert_eq!(layout.child_rects.len(), self.children.len());

        for (child_rect, child) in layout.child_rects.iter().zip(&self.children) {
            child.draw(app, draw, *child_rect, hover);
        }
    }

    fn find_hover(&self, rect: nannou::geom::Rect, mouse: nannou::geom::Vec2) -> Option<ViewId> {
        let mut layout_ref = self.layout.borrow_mut();
        let layout = self.layout(rect, &mut layout_ref);
        assert_eq!(layout.child_rects.len(), self.children.len());

        for (child_rect, child) in layout.child_rects.iter().zip(&self.children) {
            if let x @ Some(_) = child.find_hover(*child_rect, mouse) {
                return x;
            }
        }
        None
    }

    fn size(&self, given: (f32, f32)) -> (f32, f32) {
        // TODO: stay within bounds of given
        self.children.iter().map(|child| child.size(given)).fold((0.0, 0.0), |(x_acc, y_acc), (cur_size_x, cur_size_y)| {
            match self.direction {
                Direction::Horizontal => {
                    // sum x, take max of y
                    let x_sum = x_acc + cur_size_x;
                    let max_y = if cur_size_y > y_acc { cur_size_y } else { y_acc };
                    (x_sum, max_y)
                }
                Direction::Vertical => {
                    // take max of x, sum y
                    let max_x = if cur_size_x > x_acc { cur_size_x } else { x_acc };
                    let y_sum = y_acc + cur_size_y;
                    (max_x, y_sum)
                }
            }
        })
    }

    fn targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: Event) {
        for child in &self.children {
            child.targeted_event(app, data, target, event);
        }
    }

    fn event(&self, _: &nannou::App, _: &mut Data, event: Event) {
        match event {
            Event::LeftMouseDown => {}
        }
    }

    fn subscriptions(&self) -> Vec<Subscription<Data>> {
        self.children.iter().flat_map(|c| c.subscriptions()).collect()
    }
}

pub(crate) fn horizontal_flow<Data>(id_maker: &mut ViewIdMaker, children: Vec<Box<dyn View<Data>>>) -> impl View<Data> {
    flow(Direction::Horizontal, children)
}
pub(crate) fn vertical_flow<Data>(id_maker: &mut ViewIdMaker, children: Vec<Box<dyn View<Data>>>) -> impl View<Data> {
    flow(Direction::Vertical, children)
}
fn flow<Data>(direction: Direction, children: Vec<Box<dyn View<Data>>>) -> impl View<Data> {
    FlowView { children, direction, layout: RefCell::new(None) }
}
