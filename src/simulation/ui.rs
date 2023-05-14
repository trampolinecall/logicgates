pub(crate) mod location;
mod render;

pub(crate) use render::render;

struct Widget {}

pub(crate) struct NodeWidget {}

pub(crate) struct GateWidget {}

impl NodeWidget {
    pub(crate) fn new() -> NodeWidget {
        NodeWidget {}
    }
}

impl GateWidget {
    pub(crate) fn new() -> GateWidget {
        GateWidget {}
    }
}

pub(crate) fn update() {
    // TODO
}
