use crate::{
    ui::{
        message::{TargetedUIMessage, UIMessage},
        widgets::{Widget, WidgetId, WidgetIdMaker},
    },
    view,
};

// TODO: allow for multiple directions
pub(crate) struct Flow {
    id: WidgetId,
    direction: FlowDirection,
    children: Vec<Box<dyn Widget>>,
}
pub(crate) enum FlowDirection {
    Horizontal,
    Vertical,
}

impl Flow {
    pub(crate) fn new(id_maker: &mut WidgetIdMaker, direction: FlowDirection, children: Vec<Box<dyn Widget>>) -> Self {
        Self { id: id_maker.next_id(), children, direction }
    }
}

impl Widget for Flow {
    fn id(&self) -> super::WidgetId {
        self.id
    }

    fn size(&self, given: (f32, f32)) -> (f32, f32) {
        // TODO: stay within bounds of given
        self.children.iter().map(|child| child.size(given)).fold((0.0, 0.0), |(x_acc, y_acc), (cur_size_x, cur_size_y)| {
            match self.direction {
                FlowDirection::Horizontal => {
                    // sum x, take max of y
                    let x_sum = x_acc + cur_size_x;
                    let max_y = if cur_size_y > y_acc { cur_size_y } else { y_acc };
                    (x_sum, max_y)
                }
                FlowDirection::Vertical => {
                    // take max of x, sum y
                    let max_x = if cur_size_x > x_acc { cur_size_x } else { x_acc };
                    let y_sum = y_acc + cur_size_y;
                    (max_x, y_sum)
                }
            }
        })
    }

    fn view(&self, app: &nannou::App, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>) {
        struct FlowDrawing {
            children: Vec<Box<dyn view::Drawing>>,
        }
        impl view::Drawing for FlowDrawing {
            fn draw(&self, logic_gates: &crate::LogicGates, draw: &nannou::Draw, hovered: Option<&dyn view::Drawing>) {
                for child in &self.children {
                    child.draw(logic_gates, draw, hovered);
                }
            }

            fn find_hover(&self, mouse: nannou::prelude::Vec2) -> Option<&dyn view::Drawing> {
                for child in &self.children {
                    if let x @ Some(_) = child.find_hover(mouse) {
                        return x;
                    }
                }
                None
            }
        }

        // TODO: stay within bounds of rect
        let mut cur_pos = match self.direction {
            FlowDirection::Horizontal => rect.left(),
            FlowDirection::Vertical => rect.top(),
        };

        let (children_drawings, all_subscriptions): (Vec<_>, Vec<_>) = self
            .children
            .iter()
            .map(|child| {
                let child_size = child.size(rect.w_h());
                let child_xy = match self.direction {
                    FlowDirection::Horizontal => {
                        let pos = nannou::geom::vec2(cur_pos + child_size.0 / 2.0, rect.y());
                        cur_pos += child_size.0;
                        pos
                    }
                    FlowDirection::Vertical => {
                        let pos = nannou::geom::vec2(rect.x(), cur_pos - child_size.1 / 2.0);
                        cur_pos -= child_size.1;
                        pos
                    }
                };
                let child_rect = nannou::geom::Rect::from_xy_wh(child_xy, child_size.into());
                let child_drawing = child.view(app, logic_gates, child_rect);
                child_drawing
            })
            .unzip();

        (Box::new(FlowDrawing { children: children_drawings }), all_subscriptions.into_iter().flatten().collect())
    }

    fn targeted_message(&mut self, app: &nannou::App, targeted_message: TargetedUIMessage) -> Option<crate::Message> {
        if targeted_message.target == self.id {
            self.message(app, targeted_message.message)
        } else {
            for child in &mut self.children {
                if let Some(child_response) = child.targeted_message(app, targeted_message) {
                    return Some(child_response);
                }
            }

            None
        }
    }

    fn message(&mut self, _: &nannou::App, message: UIMessage) -> Option<crate::Message> {
        match message {
            UIMessage::MouseDownOnGate(_) => None,
            UIMessage::MouseMoved(_) => None,
            UIMessage::LeftMouseUp => None,
            UIMessage::MouseDownOnSlideOverToggleButton => None,
            UIMessage::MouseDownOnSlider(_, _) => None,
        }
    }
}
