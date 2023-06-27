use std::collections::HashMap;

use json::JsonValue;

use crate::simulation;

// TODO: clean this all up, esp repetitive code to get a field from an object

pub(crate) fn import(filename: &str) -> Result<simulation::Simulation, Box<dyn std::error::Error>> {
    let mut simulation = simulation::Simulation::new();
    let mut node_mapping = HashMap::new();

    let project = std::fs::read_to_string(filename)?;
    let project = json::parse(&project)?;

    let JsonValue::Object(mut project) = project else {
        return Err("toplevel json must be object".into());
    };

    let (connections, toplevel_gates) =
        (project.remove("connections").ok_or("toplevel object must contain key \"connections\"")?, project.remove("toplevel_gates").ok_or("toplevel object must contain key \"toplevel_gates\"")?);
    let JsonValue::Array(connections) = connections else { return Err("connections must be array".into()) };
    let JsonValue::Array(toplevel_gates) = toplevel_gates else { return Err("toplevel_gates must be array".into()) };

    for gate in toplevel_gates {
        let gate = parse_gate(&mut simulation.circuits, &mut simulation.gates, &mut simulation.nodes, &mut node_mapping, gate)?;
        simulation.toplevel_gates.add_gate(gate);
    }

    for connection in connections {
        let JsonValue::Array(connection) = connection else { return Err("connection must be array".into()); };
        let [JsonValue::Number(a), JsonValue::Number(b)] = &connection[..] else { return Err("connection must have 2 numbers".into()) };
        let a = f64::from(*a) as usize;
        let b = f64::from(*b) as usize;
        let node_a = node_mapping[&a];
        let node_b = node_mapping[&b];
        simulation::connections::connect(&mut simulation.connections, &mut simulation.nodes, node_a, node_b);
    }

    Ok(simulation)
}

fn parse_gate(
    circuit_map: &mut simulation::CircuitMap,
    gate_map: &mut simulation::GateMap,
    node_map: &mut simulation::NodeMap,
    node_mapping: &mut HashMap<usize, simulation::NodeKey>,
    gate: JsonValue,
) -> Result<simulation::GateKey, String> {
    let JsonValue::Object(mut gate) = gate else { return Err("gate must be object".to_string()); };
    let gate_type = gate.remove("type").ok_or("gate must have field 'type'")?.take_string().ok_or("gate type must be string")?;
    let JsonValue::Array(gate_inputs) = gate.remove("inputs").ok_or("gate must have field 'inputs'")? else { return Err("gate inputs must be array".to_string()); };
    let JsonValue::Array(gate_outputs) = gate.remove("outputs").ok_or("gate must have field 'outputs'")? else { return Err("gate outputs must be array".to_string()); };

    match &*gate_type {
        "nand" => Ok(gate_map.insert_with_key(|gk| {
            let logic = simulation::logic::NandLogic::new(node_map, gk);
            assign_node_mapping(node_mapping, logic.nodes.inputs(), &gate_inputs);
            assign_node_mapping(node_mapping, logic.nodes.outputs(), &gate_outputs);
            simulation::Gate::Nand { logic, location: simulation::location::GateLocation::new(), direction: simulation::GateDirection::LTR }
        })),

        "true" => Ok(gate_map.insert_with_key(|gk| {
            let logic = simulation::logic::ConstLogic::new(node_map, gk, true);
            assign_node_mapping(node_mapping, logic.nodes.inputs(), &gate_inputs);
            assign_node_mapping(node_mapping, logic.nodes.outputs(), &gate_outputs);
            simulation::Gate::Const { logic, location: simulation::location::GateLocation::new(), direction: simulation::GateDirection::LTR }
        })),

        "false" => Ok(gate_map.insert_with_key(|gk| {
            let logic = simulation::logic::ConstLogic::new(node_map, gk, false);
            assign_node_mapping(node_mapping, logic.nodes.inputs(), &gate_inputs);
            assign_node_mapping(node_mapping, logic.nodes.outputs(), &gate_outputs);
            simulation::Gate::Const { logic, location: simulation::location::GateLocation::new(), direction: simulation::GateDirection::LTR }
        })),

        "unerror" => Ok(gate_map.insert_with_key(|gk| {
            let logic = simulation::logic::UnerrorLogic::new(node_map, gk);
            assign_node_mapping(node_mapping, logic.nodes.inputs(), &gate_inputs);
            assign_node_mapping(node_mapping, logic.nodes.outputs(), &gate_outputs);
            simulation::Gate::Unerror { logic, location: simulation::location::GateLocation::new(), direction: simulation::GateDirection::LTR }
        })),

        "subcircuit" => {
            let ck = circuit_map.insert_with_key(|ck| simulation::Circuit::new(ck, node_map, "unnamed".to_string(), simulation::GateDirection::LTR, gate_inputs.len(), gate_outputs.len())); // TODO: names
            let JsonValue::Array(subgates) = gate.remove("gates").ok_or("subcircuit gate must have field 'gate'")? else { return Err("gate subgates must be array".to_string()); };

            assign_node_mapping(node_mapping, circuit_map[ck].nodes.inputs(), &gate_inputs);
            assign_node_mapping(node_mapping, circuit_map[ck].nodes.outputs(), &gate_outputs);

            for gate in subgates {
                let gate = parse_gate(circuit_map, gate_map, node_map, node_mapping, gate)?;
                circuit_map[ck].gates.add_gate(gate);
            }

            Ok(gate_map.insert(simulation::Gate::Custom(ck)))
        }

        _ => Err(format!("invalid gate type {}", gate_type)),
    }
}

// TODO: figure out a better way than to panic
fn assign_node_mapping(node_mapping: &mut HashMap<usize, simulation::NodeKey>, nodes: &[simulation::NodeKey], numbers: &[JsonValue]) {
    assert_eq!(nodes.len(), numbers.len());
    for (node, num) in nodes.iter().zip(numbers.iter()) {
        let JsonValue::Number(num) = num else { panic!("gate node must be number"); };
        let previous = node_mapping.insert(f64::from(*num) as usize, *node);
        assert!(previous.is_none());
    }
}
