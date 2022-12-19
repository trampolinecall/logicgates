use std::collections::HashMap;

use crate::utils;

#[derive(Clone)]
pub(crate) struct Circuit {
    pub(crate) name: String,
    pub(crate) num_inputs: usize,
    pub(crate) gates: Vec<Gate>,
    pub(crate) outputs: Vec<Value>,
}

#[derive(Clone)]
pub(crate) enum Gate {
    Custom(Circuit, Vec<Value>),
    And(Vec<Value>),
    Not(Value),
    Const(bool),
}

#[derive(Copy, Clone)]
pub(crate) enum Value {
    Arg(usize),
    GateValue(usize, usize),
}

impl Circuit {
    pub(crate) fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    pub(crate) fn eval(&self, args: &[bool]) -> Vec<bool> {
        self.eval_with_results(args).0
    }

    pub(crate) fn eval_with_results(&self, inputs: &[bool]) -> (Vec<bool>, Vec<Vec<bool>>) {
        assert_eq!(inputs.len(), self.num_inputs);

        let mut results: Vec<Vec<bool>> = Vec::new();

        let get_value = |v, results: &Vec<Vec<_>>| match v {
            Value::Arg(arg_idx) => inputs[arg_idx],
            Value::GateValue(gate_idx, output_idx) => results[gate_idx][output_idx],
        };

        for gate in &self.gates {
            results.push(match gate {
                Gate::Custom(subcircuit, subinputs) => {
                    let subinputs: Vec<bool> = subinputs.iter().map(|value| get_value(*value, &results)).collect();
                    subcircuit.eval(&subinputs)
                }
                Gate::And(v) => vec![v.into_iter().all(|v| get_value(*v, &results))],
                Gate::Not(v) => vec![!get_value(*v, &results)],
                Gate::Const(b) => vec![*b],
            });
        }

        (self.outputs.iter().map(|value| get_value(*value, &results)).collect(), results)
    }

    pub(crate) fn table(&self) -> HashMap<Vec<bool>, Vec<bool>> {
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
    pub(crate) fn inputs(&self) -> Vec<Value> {
        match self {
            Gate::And(inputs) => inputs.clone(),
            Gate::Not(v) => vec![*v],
            Gate::Const(_) => Vec::new(),
            Gate::Custom(_, inputs) => inputs.clone(),
        }
    }

    pub(crate) fn num_inputs(&self) -> usize {
        match self {
            Gate::And(inputs) => inputs.len(),
            Gate::Not(_) => 1,
            Gate::Const(_) => 0,
            Gate::Custom(subcircuit, _) => subcircuit.num_inputs,
        }
    }

    pub(crate) fn num_outputs(&self) -> usize {
        match self {
            Gate::And(_) => 1,
            Gate::Const(_) => 1,
            Gate::Not(_) => 1,
            Gate::Custom(subcircuit, _) => subcircuit.outputs.len(),
        }
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            Gate::And(_) => "and",
            Gate::Not(_) => "not",
            Gate::Const(b) => {
                if *b {
                    "true"
                } else {
                    "false"
                }
            }
            Gate::Custom(subcircuit, _) => &subcircuit.name,
        }
    }
}
