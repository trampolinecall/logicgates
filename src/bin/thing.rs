use logicgates::circuit;

fn main() {
    let main_circuit = circuit::Circuit { arity: 2, gates: vec![circuit::Gate::And(circuit::Value::Arg(0), circuit::Value::Arg(1))], output: vec![circuit::Value::GateValue(0, 0)] };

    println!("{:?}", main_circuit.eval(&vec![true, true]));
    println!("{:#?}", main_circuit.table());
}
