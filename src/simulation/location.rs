use std::collections::{BTreeMap, HashMap};

use crate::simulation::{draw, logic, GateKey, GateMap, NodeKey, Simulation};

pub(crate) struct GateLocation {
    pub(crate) x: u32,
    pub(crate) y: f32,
}

impl GateLocation {
    pub(crate) fn new() -> Self {
        Self { x: 0, y: 0.0 }
    }
}

pub(crate) fn calculate_locations(simulation: &mut Simulation) {
    let locations = calculate_locations_(simulation);
    apply_locations(&mut simulation.gates, locations);
}

fn calculate_locations_(simulation: &Simulation) -> HashMap<GateKey, GateLocation> {
    /* old iterative position calculating algorithm based on a loss function and trying to find a minimum loss
    // gate position scoring; lower is better
    let score = |current_idx: usize, current_loc @ [x, y]: [f64; 2], gate: &simulation::Gate| -> f64 {
        let place_100_right_of_rightmost_input = {
            let desired_x = gate
                .inputs()
                .into_iter()
                .map(|input| match input {
                    crate::simulation::Value::Arg(_) => 0.0,
                    crate::simulation::Value::GateValue(g, _) => locations[g][0],
                })
                .reduce(f64::max)
                .unwrap_or(0.0)
                + 100.0;

            ((x - desired_x) / 10.0).powf(2.0)
        };

        let place_y_at_middle_of_inputs: f64 = {
            let input_y = |input| match input {
                simulation::Value::Arg(_) => 360.0, // TODO: dont hardcode input argument position
                simulation::Value::GateValue(g, o) => gate_output_pos(g, o)[1],
            };
            let desired_y = (gate.inputs().into_iter().map(input_y).sum::<f64>()) / (gate.num_inputs() as f64);

            ((y - desired_y) / 10.0).powf(2.0)
        };

        let space_from_others: f64 = {
            let dist = |[x1, y1]: [f64; 2], [x2, y2]: [f64; 2]| ((x1 - x2).powf(2.0) + (y1 - y2).powf(2.0)).sqrt();
            let min_dist = self
                .locations
                .iter()
                .copied()
                .enumerate()
                .map(|(loc_idx, loc)| if loc_idx != current_idx && (loc[0] - current_loc[0]).abs() < 200.0 { dist(loc, current_loc) } else { f64::MAX })
                .reduce(f64::min);

            match min_dist {
                Some(min_dist) if min_dist < 100.0 => 10000.0 / min_dist,
                _ => 0.0,
            }
        };

        place_100_right_of_rightmost_input + place_y_at_middle_of_inputs + space_from_others
    };

    let new_locations: Vec<[f64; 2]> = self
        .locations
        .iter()
        .zip(circuit.gates.iter())
        .enumerate()
        .map(|(idx, (location, gate))| {
            const DELTA: f64 = 0.0001;
            let x_deriv = (score(idx, [location[0] + DELTA, location[1]], gate) - score(idx, *location, gate)) / DELTA;
            let y_deriv = (score(idx, [location[0], location[1] + DELTA], gate) - score(idx, *location, gate)) / DELTA;

            [location[0] - x_deriv.clamp(-100.0, 100.0), location[1] - y_deriv.clamp(-100.0, 100.0)]
        })
        .collect();

    locations = new_locations;
    */

    // TODO: test this

    // gates in subcircuits just get processed based on the other gates they are connected to, meaning that their positions are independent of the positions of the gates in the supercircuits
    // TODO: it actually does not work properly as described in the line above so fix this

    // group them into columns with each one going one column right of its rightmost dependency
    let mut xs: BTreeMap<GateKey, u32> = simulation.gates.iter().map(|(g_i, _)| (g_i, 0)).collect();
    // TODO: this has to run repeatedly in case the gates are not in topologically sorted order
    for (gate_i, _) in simulation.gates.iter() {
        let input_producer_x = |input: NodeKey| match simulation.nodes[input.into()].value.producer() {
            Some(producer) => 0, /* TODO: match simulation.nodes[producer].gate {
            Some(producer_gate) => xs[&producer_gate], // receiver node connected to other gate output node
            None => 0,                                 // receiver node connected to circuit input node
            },
             */
            None => 0, // receiver node not connected
        };
        xs.insert(gate_i, logic::gate_inputs(&simulation.circuits, &simulation.gates, gate_i).iter().copied().map(input_producer_x).max().unwrap_or(0) + 1);
    }

    // within each column sort them by the average of their input ys
    let mut ys: BTreeMap<GateKey, f32> = simulation.gates.iter().map(|(index, _)| (index, 0.0)).collect();
    for x in 1..=*xs.values().max().unwrap_or(&0) {
        let input_producer_y = |input: NodeKey| match simulation.nodes[input.into()].value.producer() {
            Some(producer) => 0.0, /* TODO: match simulation.nodes[producer].value.gate {
            Some(producer_gate) => ys[&producer_gate], // receiver node connected to other node
            None => 0.0,                               // receiver node connected to circuit input node
            },
             */
            None => 0.0, // receiver node not connected
        };
        let mut on_current_column: Vec<_> = simulation.gates.iter().filter(|(gate_i, _)| xs[gate_i] == x).collect();
        on_current_column.sort_by(|(gate1_i, _), (gate2_i, _)| {
            let gate1_y = logic::gate_inputs(&simulation.circuits, &simulation.gates, *gate1_i).iter().copied().map(input_producer_y).sum::<f32>(); // sum can be used as average because they are only being compared to each other
            let gate2_y = logic::gate_inputs(&simulation.circuits, &simulation.gates, *gate2_i).iter().copied().map(input_producer_y).sum::<f32>();
            gate1_y.partial_cmp(&gate2_y).unwrap()
        });

        // set the y values
        const PADDING: f32 = 20.0;
        let all_height: f32 = on_current_column.iter().map(|(g_i, _)| draw::gate_display_size(simulation, *g_i)[1]).sum::<f32>() + PADDING * (on_current_column.len() - 1) as f32;
        let mut start_y = -all_height / 2.0;
        for (gate_i, _) in &on_current_column {
            ys.insert(*gate_i, start_y);
            start_y += draw::gate_display_size(simulation, *gate_i)[1];
            start_y += PADDING;
        }
    }

    xs.into_iter()
        .zip(ys)
        .map(|((x_gate_index, x), (y_gate_index, y))| {
            assert_eq!(x_gate_index, y_gate_index); // should be the same because the maps are sorted by the key
            (x_gate_index, GateLocation { x, y })
        })
        .collect()
}

fn apply_locations(gates: &mut GateMap, locations: HashMap<GateKey, GateLocation>) {
    for (gate_i, location) in locations {
        gates[gate_i].location = location;
    }
}
