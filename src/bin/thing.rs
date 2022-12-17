use logicgates::circuit;
use logicgates::eval;

fn main() {
    let main_circuit = circuit::Circuit { arity: 2, gates: vec![circuit::Gate::And(circuit::Value::Arg(0), circuit::Value::Arg(1))], output: vec![circuit::Value::GateValue(0, 0)] };

    println!("{:?}", eval::eval(&main_circuit, &vec![true, true]));
    println!("{:#?}", eval::table(&main_circuit));
}
