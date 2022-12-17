use logicgates::rep;
use logicgates::eval;

fn main () {
    let main_circuit = rep::Circuit {
        arity: 2,
        gates: vec![rep::Gate::And(rep::Value::Arg(0), rep::Value::Arg(1))],
        output: vec![rep::Value::Register(0), rep::Value::Register(0)]
    };

    println!("{:?}", eval::eval(&main_circuit, &vec![true, true]))
}
