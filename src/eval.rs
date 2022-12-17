use std::collections::HashMap;

use crate::circuit;

pub fn table(circuit: &circuit::Circuit) -> HashMap<Vec<bool>, Vec<bool>> {
    enumerate_inputs(circuit.arity)
        .into_iter()
        .map(|input| {
            let res = eval(circuit, &input);
            (input, res)
        })
        .collect()
}

pub fn enumerate_inputs(arity: usize) -> Vec<Vec<bool>> {
    let mut inputs = vec![vec![false], vec![true]];
    for _ in 0..(arity - 1) {
        let mut inputs_false = inputs.clone();
        let mut inputs_true = inputs;

        inputs_false.iter_mut().for_each(|i| i.insert(0, false));
        inputs_true.iter_mut().for_each(|i| i.insert(0, true));

        inputs = inputs_false;
        inputs.extend(inputs_true);
    }
    inputs
}

pub fn eval(circuit: &circuit::Circuit, args: &[bool]) -> Vec<bool> {
    eval_with_results(circuit, args).0
}

pub fn eval_with_results(circuit: &circuit::Circuit, args: &[bool]) -> (Vec<bool>, Vec<Vec<bool>>) {
    assert_eq!(args.len(), circuit.arity);

    let mut registers = Vec::new();

    for gate in &circuit.gates {
        registers.push(match gate {
            /*
            rep::Gate::Custom(subcircuit, sub_args) => {
                let sub_args: Vec<bool> = sub_args.iter().map(|value| get_value(value, args, &registers)).collect();
                registers.extend(eval(subcircuit, &sub_args))
            }
            */
            circuit::Gate::And(a, b) => vec![get_value(a, &args, &registers) && get_value(b, &args, &registers)],
        });
    }

    (circuit.output.iter().map(|value| get_value(value, &args, &registers)).collect(), registers)
}

fn get_value(v: &circuit::Value, args: &[bool], gate_values: &[Vec<bool>]) -> bool {
    match v {
        circuit::Value::Arg(arg_idx) => args[*arg_idx],
        circuit::Value::GateValue(gate_idx, output_idx) => gate_values[*gate_idx][*output_idx],
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn enumerate_inputs() {
        assert_eq!(super::enumerate_inputs(2), vec![vec![false, false], vec![false, true], vec![true, false], vec![true, true]]);
    }
}
