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
    children: Vec<Box<dyn Widget>>,
}

impl Flow {
    pub(crate) fn new(id_maker: &mut WidgetIdMaker, children: Vec<Box<dyn Widget>>) -> Self {
        Self { id: id_maker.next_id(), children }
    }
}

impl Widget for Flow {
    fn id(&self) -> super::WidgetId {
        self.id
    }

    fn size(&self, given: (f32, f32)) -> (f32, f32) {
        // TODO: stay within bounds of given
        self.children.iter().map(|child| child.size(given)).fold((0.0, 0.0), |(last_max_x, last_y_sum), (cur_size_x, cur_size_y)| {
            let max_x = if cur_size_x > last_max_x { cur_size_x } else { last_max_x };
            let y_sum = last_y_sum + cur_size_y;
            (max_x, y_sum)
        })
    }

    fn view(&self, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>) {
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
        let mut cur_y = rect.top();

        let (children_drawings, all_subscriptions): (Vec<_>, Vec<_>) = self
            .children
            .iter()
            .map(|child| {
                let child_size = child.size(rect.w_h());
                let child_drawing = child.view(logic_gates, nannou::geom::Rect::from_x_y_w_h(rect.x(), cur_y - child_size.1 / 2.0, child_size.0, child_size.1));
                cur_y -= child_size.1;
                child_drawing
            })
            .unzip();
        (Box::new(FlowDrawing { children: children_drawings }), all_subscriptions.into_iter().flatten().collect())
    }

    fn targeted_message(&mut self, targeted_message: TargetedUIMessage) -> Option<crate::Message> {
        if targeted_message.target == self.id {
            self.message(targeted_message.message)
        } else {
            for child in &mut self.children {
                if let Some(child_response) = child.targeted_message(targeted_message) {
                    return Some(child_response);
                }
            }

            None
        }
    }

    fn message(&mut self, message: UIMessage) -> Option<crate::Message> {
        match message {
            UIMessage::MouseDownOnGate(_) => None,
            UIMessage::MouseMoved(_) => None,
            UIMessage::LeftMouseUp => None,
            UIMessage::MouseDownOnSlideOverToggleButton => None,
        }
    }
}
