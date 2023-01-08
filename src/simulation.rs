use generational_arena::Arena;

pub(crate) mod circuit;
pub(crate) mod components;
pub(crate) mod position;

pub(crate) type GateIndex = generational_arena::Index;

pub(crate) struct Circuit {
    pub(crate) name: String,
    pub(crate) gates: Vec<GateIndex>,
    pub(crate) inputs: components::connection::ProducersComponent, // usually inputs are receivers but in circuits, if this is the main circuit, the inputs are producers because they are user controlled, and inputs of subcircuits are just passthrough producer nodes
    pub(crate) outputs: components::connection::ReceiversComponent,
}

pub(crate) struct Gate {
    pub(crate) index: GateIndex,
    pub(crate) calculation: components::calculator::CalculationComponent,
    pub(crate) draw_component: components::draw::DrawComponent,
}

pub(crate) struct Simulation {
    pub(crate) gates: Arena<Gate>,

    pub(crate) main_circuit: GateIndex, // TODO: disallow the main circuit being a nand or const
}

impl Simulation {
    pub(crate) fn render(&self, gl: &mut opengl_graphics::GlGraphics, render_args: &piston::RenderArgs) {
        components::draw::render(&self.gates, self.main_circuit, gl, render_args);
    }

    pub(crate) fn update(&mut self) {
        components::calculator::update(&mut self.gates);
    }
}
