pub struct Circuit {
    pub arity: usize,
    pub gates: Vec<Gate>,
    pub output: Vec<Value>,
}

pub enum Gate {
    // Custom(Circuit, Vec<Value>),
    And(Value, Value),
}

pub enum Value {
    Arg(usize),
    Register(usize),
}
