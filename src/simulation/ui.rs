pub(crate) mod location;
mod render;

pub(crate) use render::render;

#[derive(Copy, Clone)]
struct DrawShape {}

pub(crate) struct NodeWidget {}

pub(crate) struct GateWidget {}

impl NodeWidget {
    pub(crate) fn new() -> NodeWidget {
        NodeWidget {}
    }

    fn draw(&self) -> DrawShape {
        todo!()
    }
}

impl GateWidget {
    pub(crate) fn new() -> GateWidget {
        GateWidget {}
    }

    fn draw(&self) -> DrawShape {
        todo!()
    }
}

pub(crate) fn update() {
    // TODO
}

impl DrawShape {
    fn new_for_connection(adj_pos: nannou::prelude::Vec2, cur_pos: nannou::prelude::Vec2) -> DrawShape {
        todo!()
    }

    fn render(&self, app: &nannou::App, draw: &nannou::Draw) {
        todo!()
    }
}
