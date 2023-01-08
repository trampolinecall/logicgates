use std::collections::{BTreeMap, HashMap};

use generational_arena::Arena;

use crate::simulation::{circuit, draw};

use super::connections;

// TODO: move this into location component

pub(crate) fn calculate_locations(circuits: &mut Arena<circuit::Circuit>, gates: &mut Arena<circuit::Gate>) {
    let locations = calculate_locations_(circuits, gates);
    apply_locations(circuits, gates, locations);
}

fn calculate_locations_(circuits: &Arena<circuit::Circuit>, gates: &Arena<circuit::Gate>) -> HashMap<circuit::GateIndex, (u32, f64)> {
    /* old iterative position calculating algorithm based on a loss function and trying to find a minimum loss
    // gate position scoring; lower is better
    let score = |current_idx: usize, current_loc @ [x, y]: [f64; 2], gate: &circuit::Gate| -> f64 {
        let place_100_right_of_rightmost_input = {
            let desired_x = gate
                .inputs()
                .into_iter()
                .map(|input| match input {
                    crate::circuit::Value::Arg(_) => 0.0,
                    crate::circuit::Value::GateValue(g, _) => locations[g][0],
                })
                .reduce(f64::max)
                .unwrap_or(0.0)
                + 100.0;

            ((x - desired_x) / 10.0).powf(2.0)
        };

        let place_y_at_middle_of_inputs: f64 = {
            let input_y = |input| match input {
                circuit::Value::Arg(_) => 360.0, // TODO: dont hardcode input argument position
                circuit::Value::GateValue(g, o) => gate_output_pos(g, o)[1],
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

    // group them into columns with each one going one column right of its rightmost dependency
    let mut xs: BTreeMap<circuit::GateIndex, u32> = gates.iter().map(|(g_i, _)| (g_i, 0)).collect();
    // TODO: this has to run repeatedly in case the gates are not in topologically sorted order
    for (gate_i, gate) in gates.iter() {
        let input_producer_x = |input: connections::GateInputNodeIdx| match connections::get_receiver(circuits, gates, input.into()).producer() {
            Some(producer) => match connections::get_producer(circuits, gates, producer).gate {
                Some(producer_gate) => xs[&producer_gate], // receiver node connected to other gate output node
                None => 0,                                 // receiver node connected to circuit input node
            },
            None => 0, // receiver node not connected
        };
        xs.insert(gate_i, connections::gate_inputs(gate).map(input_producer_x).max().unwrap_or(0) + 1);
    }

    // within each column sort them by the average of their input ys
    let mut ys: BTreeMap<circuit::GateIndex, f64> = gates.iter().map(|(index, _)| (index, 0.0)).collect();
    for x in 1..=*xs.values().max().unwrap_or(&0) {
        let input_producer_y = |input: connections::GateInputNodeIdx| match connections::get_receiver(circuits, gates, input.into()).producer() {
            Some(producer) => match connections::get_producer(circuits, gates, producer).gate {
                Some(producer_gate) => ys[&producer_gate], // receiver node connected to other node
                None => 0.0,                               // receiver node connected to circuit input node
            },
            None => 0.0, // receiver node not connected
        };
        let mut on_current_column: Vec<_> = gates.iter().filter(|(gate_i, _)| xs[gate_i] == x).collect();
        on_current_column.sort_by(|(_, gate1), (_, gate2)| {
            let gate1_y = connections::gate_inputs(gate1).map(input_producer_y).sum::<f64>(); // sum can be used as average because they are only being compared to each other
            let gate2_y = connections::gate_inputs(gate2).map(input_producer_y).sum::<f64>();
            gate1_y.partial_cmp(&gate2_y).unwrap()
        });

        // set the y values
        const PADDING: f64 = 20.0;
        let all_height: f64 = on_current_column.iter().map(|(_, g)| draw::gate_display_size(g)[1]).sum::<f64>() + PADDING * (on_current_column.len() - 1) as f64;
        let mut start_y = -all_height / 2.0;
        for (gate_i, gate) in &on_current_column {
            ys.insert(*gate_i, start_y);
            start_y += draw::gate_display_size(gate)[1];
            start_y += PADDING;
        }
    }

    xs.into_iter()
        .zip(ys)
        .map(|((x_gate_index, gate_x), (y_gate_index, gate_y))| {
            assert_eq!(x_gate_index, y_gate_index); // should be the same because the maps are sorted by the key
            (x_gate_index, (gate_x, gate_y))
        })
        .collect()
}

fn apply_locations(circuits: &mut Arena<circuit::Circuit>, gates: &mut Arena<circuit::Gate>, locations: HashMap<circuit::GateIndex, (u32, f64)>) {
    for (gate_i, location) in locations {
        gates[gate_i].location = location;
    }
}
