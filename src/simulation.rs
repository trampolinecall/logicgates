use generational_arena::Arena;

pub(crate) mod circuit;
pub(crate) mod components;
pub(crate) mod position;

pub(crate) type GateIndex = generational_arena::Index;

pub(crate) struct Circuit {
    pub(crate) name: String,
    pub(crate) gates: Vec<GateIndex>,
    pub(crate) inputs: Vec<components::connection::Producer>,
    pub(crate) outputs: Vec<components::connection::Receiver>,
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
