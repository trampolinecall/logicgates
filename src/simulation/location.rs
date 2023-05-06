use std::collections::{BTreeMap, HashMap};

use crate::simulation::{draw, logic, CircuitKey, GateKey, GateMap, NodeParent, Simulation};

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

    let mut locations = HashMap::new();
    for (_, circuit) in &simulation.circuits {
        // process each circuit individually because each circuit positions each of its gates independently of each other
        // also this should cover every gate only once because a gate should never be part of 2 circuits at the same time

        let subcircuits_in_cur_circuit: BTreeMap<CircuitKey, GateKey> = circuit.gates.iter().filter_map(|&gk| Some((simulation.gates[gk].logic.as_subcircuit()?, gk))).collect();
        let get_prev_gates_excluding_self = |gate: GateKey| {
            let gate_inputs = logic::gate_inputs(&simulation.circuits, &simulation.gates, gate);
            let prev_gates = gate_inputs.iter().map(|&node_k| match simulation.nodes[node_k].parent {
                NodeParent::Gate(gk) => gk,
                NodeParent::Circuit(ck) => *subcircuits_in_cur_circuit.get(&ck).expect("gate is connected to custom gate outside of current circuit"), // TODO: handle if ck == circuit, if this is connected to the current circuit's input node
            });
            prev_gates.filter(move |x| *x != gate)
        };

        // group them into columns with each one going one column right of its rightmost dependency
        // if a gate is connected to itself, it will just ignore that it is its own dependency
        let mut xs: BTreeMap<GateKey, Option<u32>> = circuit.gates.iter().copied().map(|g_i| (g_i, None)).collect();
        while xs.values().any(Option::is_none) {
            for &gate_i in &circuit.gates {
                let prev_gates_excluding_self = get_prev_gates_excluding_self(gate_i);
                let max_prev_gate_x: Option<u32> = prev_gates_excluding_self
                    .map(|prev| xs.get(&prev).expect("gate is connected to prev gate outisde of current circuit"))
                    .copied()
                    .try_fold(0, |last_max, cur_x| cur_x.map(|cur_x| std::cmp::max(last_max, cur_x)));
                if let Some(max_prev_gate_x) = max_prev_gate_x {
                    xs.insert(gate_i, Some(max_prev_gate_x + 1));
                }
            }
        }
        let xs: BTreeMap<_, _> = xs.into_iter().map(|(k, v)| (k, v.unwrap())).collect();

        // within each column sort them by the average of their input ys
        let mut ys: BTreeMap<GateKey, f32> = BTreeMap::new();
        for cur_col in 0..=*xs.values().max().unwrap_or(&0) {
            let get_gate_relative_y = |gate: GateKey| -> f32 {
                let prev_gates_excluding_self = get_prev_gates_excluding_self(gate);
                let prev_gate_ys = prev_gates_excluding_self.map(|prev| ys[&prev]);

                prev_gate_ys.sum() // sum can be used as average because they are only being compared to each other
            };

            let mut on_current_column: Vec<_> = xs.iter().filter_map(|(&gate_i, &gate_x)| if gate_x == cur_col { Some(gate_i) } else { None }).collect();
            if on_current_column.is_empty() {
                continue;
            }
            on_current_column.sort_by(|&gate1_i, &gate2_i| {
                let gate1_y = get_gate_relative_y(gate1_i);
                let gate2_y = get_gate_relative_y(gate2_i);
                gate1_y.partial_cmp(&gate2_y).unwrap()
            });

            // set the y values
            const PADDING: f32 = 20.0;
            let all_height: f32 = on_current_column.iter().map(|&gate| draw::gate_display_size(simulation, gate)[1]).sum::<f32>() + PADDING * (on_current_column.len() - 1) as f32; // TODO: remove dependency on draw system
            let mut start_y = -all_height / 2.0;
            for &gate_i in &on_current_column {
                ys.insert(gate_i, start_y);
                start_y += draw::gate_display_size(simulation, gate_i)[1];
                start_y += PADDING;
            }
        }

        locations.extend(xs.into_iter().zip(ys).map(|((x_gate_index, x), (y_gate_index, y))| {
            assert_eq!(x_gate_index, y_gate_index); // should be the same because the maps are sorted by the key
            (x_gate_index, GateLocation { x, y })
        }))
    }
    locations
}

fn apply_locations(gates: &mut GateMap, locations: HashMap<GateKey, GateLocation>) {
    for (gate_i, location) in locations {
        gates[gate_i].location = location;
    }
}
