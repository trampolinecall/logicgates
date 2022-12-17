pub struct Circuit {
    pub arity: usize,
    pub gates: Vec<Gate>,
    pub output: Vec<Value>,
}

pub enum Gate {
    // Custom(Circuit, Vec<Value>),
    And(Value, Value),
}

impl Gate {
    pub fn inputs(&self) -> Vec<Value> {
        match self {
            Gate::And(a, b) => vec![*a, *b],
        }
    }

    pub fn num_inputs(&self) -> usize {
        match self {
            Gate::And(_, _) => 2,
        }
    }

    pub fn num_outputs(&self) -> usize {
        match self {
            Gate::And(_, _) => 1,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Gate::And(_, _) => "and",
        }
    }
}

#[derive(Copy, Clone)]
pub enum Value {
    Arg(usize),
    GateValue(usize, usize),
}
