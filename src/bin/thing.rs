use logicgates::circuit;

fn main() {
    let main_circuit = circuit::Circuit { num_inputs: 2, gates: vec![circuit::Gate::And(circuit::Value::Arg(0), circuit::Value::Arg(1))], outputs: vec![circuit::Value::GateValue(0, 0)] };

    println!("{:?}", main_circuit.eval(&[true, true]));
    println!("{:#?}", main_circuit.table());
}
