use crate::rep;

pub fn eval(circuit: &rep::Circuit, args: &[bool]) -> Vec<bool> {
    assert!(args.len() == circuit.arity);

    let mut registers = Vec::new();

    for gate in &circuit.gates {
        match gate {
            /*
            rep::Gate::Custom(subcircuit, sub_args) => {
                let sub_args: Vec<bool> = sub_args.iter().map(|value| get_value(value, args, &registers)).collect();
                registers.extend(eval(subcircuit, &sub_args))
            }
            */

            rep::Gate::And(a, b) => registers.push(get_value(a, &args, &registers) && get_value(b, &args, &registers)),
        };
    }

    circuit.output.iter().map(|value| get_value(value, &args, &registers)).collect()
}

fn get_value(v: &rep::Value, args: &[bool], registers: &[bool]) -> bool {
    match v {
        rep::Value::Arg(arg_idx) => args[*arg_idx],
        rep::Value::Register(reg_idx) => registers[*reg_idx],
    }
}
