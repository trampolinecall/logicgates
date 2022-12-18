use std::collections::HashMap;

use crate::utils;

pub struct Circuit {
    pub num_inputs: usize,
    pub gates: Vec<Gate>,
    pub outputs: Vec<Value>,
}

pub enum Gate {
    // Custom(Circuit, Vec<Value>),
    And(Value, Value),
    Not(Value),
    Const(bool),
}

#[derive(Copy, Clone)]
pub enum Value {
    Arg(usize),
    GateValue(usize, usize),
}

impl Circuit {
    pub fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    pub fn eval(&self, args: &[bool]) -> Vec<bool> {
        self.eval_with_results(args).0
    }

    pub fn eval_with_results(&self, inputs: &[bool]) -> (Vec<bool>, Vec<Vec<bool>>) {
        assert_eq!(inputs.len(), self.num_inputs);

        let mut results: Vec<Vec<bool>> = Vec::new();

        let get_value = |v, results: &Vec<Vec<_>>| match v {
            Value::Arg(arg_idx) => inputs[arg_idx],
            Value::GateValue(gate_idx, output_idx) => results[gate_idx][output_idx],
        };

        for gate in &self.gates {
            results.push(match gate {
                /*
                rep::Gate::Custom(subcircuit, subinputs) => {
                    let subinputs: Vec<bool> = subinputs.iter().map(|value| get_value(value, inputs, &registers)).collect();
                    registers.extend(eval(subcircuit, &subinputs))
                }
                */
                Gate::And(a, b) => vec![get_value(*a, &results) && get_value(*b, &results)],
                Gate::Not(v) => vec![!get_value(*v, &results)],
                Gate::Const(b) => vec![*b],
            });
        }

        (self.outputs.iter().map(|value| get_value(*value, &results)).collect(), results)
    }

    pub fn table(&self) -> HashMap<Vec<bool>, Vec<bool>> {
        utils::enumerate_inputs(self.num_inputs)
            .into_iter()
            .map(|input| {
                let res = self.eval(&input);
                (input, res)
            })
            .collect()
    }
}

impl Gate {
    pub fn inputs(&self) -> Vec<Value> {
        match self {
            Gate::And(a, b) => vec![*a, *b],
            Gate::Not(v) => vec![*v],
            Gate::Const(_) => Vec::new(),
        }
    }

    pub fn num_inputs(&self) -> usize {
        match self {
            Gate::And(_, _) => 2,
            Gate::Not(_) => 1,
            Gate::Const(_) => 0,
        }
    }

    pub fn num_outputs(&self) -> usize {
        match self {
            Gate::And(_, _) => 1,
            Gate::Const(_) => 1,
            Gate::Not(_) => 1,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Gate::And(_, _) => "and",
            Gate::Not(_) => "not",
            Gate::Const(b) => {
                if *b {
                    "true"
                } else {
                    "false"
                }
            }
        }
    }
}
