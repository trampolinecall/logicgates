pub(crate) mod simulation;

use crate::view;

pub(crate) trait Widget {
    type Drawing: view::Drawing;
    fn view(&self, logic_gates: &crate::LogicGates, rect: nannou::geom::Rect) -> Self::Drawing;
}
