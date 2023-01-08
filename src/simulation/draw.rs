use crate::simulation::{logic, CircuitIndex, CircuitMap, GateIndex, GateMap};

// TODO: consider whether to merge draw and location components

const CIRCLE_RAD: f64 = 5.0;
const CONNECTION_RAD: f64 = CIRCLE_RAD / 2.0;
const VERTICAL_VALUE_SPACING: f64 = 20.0;
const HORIZONTAL_GATE_SPACING: f64 = 100.0;

const BG: [f32; 4] = [0.172, 0.243, 0.313, 1.0];
const GATE_COLOR: [f32; 4] = [0.584, 0.647, 0.65, 1.0];
const ON_COLOR: [f32; 4] = [0.18, 0.8, 0.521, 1.0];
const OFF_COLOR: [f32; 4] = [0.498, 0.549, 0.552, 1.0];

pub(crate) fn render(circuits: &CircuitMap, gates: &GateMap, main_circuit: CircuitIndex, graphics: &mut opengl_graphics::GlGraphics, render_args: &piston::RenderArgs) {
    use graphics::{clear, ellipse, line_from_to, rectangle};

    let main_circuit = &circuits[main_circuit];
    graphics.draw(render_args.viewport(), |c, gl| {
        clear(BG, gl);

        // draw connections first
        // dont go through circuit inputs (the ones on the left edge of the screen) because those should not be drawn connected to anything
        // dont go through gate output indexes (the ones on the right edge of gates) because those are usually conteccted to some internal gates not part of the main circuit
        let circuit_outputs = logic::circuit_output_indexes(main_circuit).map(Into::into);
        let gate_inputs = main_circuit.gates.iter().flat_map(|gi| logic::gate_input_indexes(circuits, gates, *gi).map(Into::into));
        for node_idx in circuit_outputs.chain(gate_inputs) {
            if let Some(producer) = logic::get_node(circuits, gates, node_idx).producer() {
                let color = node_color(circuits, gates, node_idx);
                let cur_pos = node_pos(circuits, gates, render_args, node_idx);
                let producer_pos = node_pos(circuits, gates, render_args, producer);
                line_from_to(color, CONNECTION_RAD, producer_pos, cur_pos, c.transform, gl);
            }
        }

        // draw gate rectangles
        for gate_i in &main_circuit.gates {
            let [gate_x, gate_y, gate_width, gate_height] = gate_box(circuits, gates, render_args, *gate_i);
            rectangle(GATE_COLOR, [gate_x, gate_y, gate_width, gate_height], c.transform, gl);

            // TODO: draw gate name
            // text(BLACK, 10, gate.name(), /* character cache */, c.transform, gl);
        }

        // draw nodes
        let circuit_inputs = logic::circuit_input_indexes(main_circuit).map(Into::into);
        let circuit_outputs = logic::circuit_output_indexes(main_circuit).map(Into::into);
        let gate_inputs = main_circuit.gates.iter().flat_map(|gi| logic::gate_input_indexes(circuits, gates, *gi).map(Into::into));
        let gate_outputs = main_circuit.gates.iter().flat_map(|gi| logic::gate_output_indexes(circuits, gates, *gi).map(Into::into));
        for node_idx in circuit_inputs.chain(circuit_outputs).chain(gate_inputs).chain(gate_outputs) {
            let pos = node_pos(circuits, gates, render_args, node_idx);
            let color = node_color(circuits, gates, node_idx);
            ellipse(color, ellipse::circle(pos[0], pos[1], CIRCLE_RAD), c.transform, gl);
        }
    });
}

pub(crate) fn gate_display_size(circuits: &CircuitMap, gates: &GateMap, gate: GateIndex) -> [f64; 2] {
    const EXTRA_VERTICAL_HEIGHT: f64 = 40.0;
    const GATE_WIDTH: f64 = 50.0;

    let gate_height = (std::cmp::max(logic::gate_num_inputs(circuits, gates, gate), logic::gate_num_outputs(circuits, gates, gate)) - 1) as f64 * VERTICAL_VALUE_SPACING + EXTRA_VERTICAL_HEIGHT;
    [GATE_WIDTH, gate_height]
}

fn y_centered_around(center_y: f64, total: usize, index: usize) -> f64 {
    let args_height: f64 = ((total - 1) as f64) * VERTICAL_VALUE_SPACING;
    let args_start_y = center_y - (args_height / 2.0);
    args_start_y + (index as f64) * VERTICAL_VALUE_SPACING
}

fn gate_box(circuits: &CircuitMap, gates: &GateMap, args: &piston::RenderArgs, gate_index: GateIndex) -> [f64; 4] {
    let (gate_x, gate_y) = gates[gate_index].location.location;
    let [gate_width, gate_height] = gate_display_size(circuits, gates, gate_index);
    [gate_x as f64 * HORIZONTAL_GATE_SPACING, gate_y + args.window_size[1] / 2.0, gate_width, gate_height]
}

fn circuit_input_pos(circuits: &CircuitMap, args: &piston::RenderArgs, index: logic::CircuitInputNodeIdx) -> [f64; 2] {
    let circuit = &circuits[index.0];
    [0.0, y_centered_around(args.window_size[1] / 2.0, circuit.inputs.len(), index.1)]
}
fn circuit_output_pos(circuits: &CircuitMap, args: &piston::RenderArgs, index: logic::CircuitOutputNodeIdx) -> [f64; 2] {
    let circuit = &circuits[index.0];
    [args.window_size[0], y_centered_around(args.window_size[1] / 2.0, circuit.outputs.len(), index.1)]
}

fn gate_input_pos(circuits: &CircuitMap, gates: &GateMap, args: &piston::RenderArgs, input_idx: logic::GateInputNodeIdx) -> [f64; 2] {
    let gate_index = input_idx.0;
    let [gate_x, gate_y, _, gate_height] = gate_box(circuits, gates, args, gate_index);
    [gate_x, y_centered_around(gate_y + gate_height / 2.0, logic::gate_num_inputs(circuits, gates, gate_index), input_idx.1)]
}
fn gate_output_pos(circuits: &CircuitMap, gates: &GateMap, args: &piston::RenderArgs, output_idx: logic::GateOutputNodeIdx) -> [f64; 2] {
    let gate_index = output_idx.0;
    let [gate_x, gate_y, gate_width, gate_height] = gate_box(circuits, gates, args, gate_index);
    [gate_x + gate_width, y_centered_around(gate_y + gate_height / 2.0, logic::gate_num_outputs(circuits, gates, gate_index), output_idx.1)]
}

fn node_pos(circuits: &CircuitMap, gates: &GateMap, args: &piston::RenderArgs, node: logic::NodeIdx) -> [f64; 2] {
    match node {
        logic::NodeIdx::CI(ci) => circuit_input_pos(circuits, args, ci),
        logic::NodeIdx::CO(co) => circuit_output_pos(circuits, args, co),
        logic::NodeIdx::GI(gi) => gate_input_pos(circuits, gates, args, gi),
        logic::NodeIdx::GO(go) => gate_output_pos(circuits, gates, args, go),
    }
}

fn node_color(circuits: &CircuitMap, gates: &GateMap, node: logic::NodeIdx) -> [f32; 4] {
    if logic::get_node_value(circuits, gates, node) {
        ON_COLOR
    } else {
        OFF_COLOR
    }
}
