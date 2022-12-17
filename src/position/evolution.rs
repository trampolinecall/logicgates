pub fn position_iterative(circuit: &crate::circuit::Circuit, locations: &[[f64; 2]]) -> Vec<[f64; 2]> {
    use rand::Rng;
    let mut rand = rand::thread_rng();
    let mut possiblilities: Vec<Vec<[f64; 2]>> = (0..10).map(|_| locations.iter().map(|[x, y]| [x + rand.gen_range(-1_f64..1_f64), y + rand.gen_range(-1_f64..1_f64)]).collect()).collect();
    // scoring function, lower is better
    let score = |locations: &Vec<[f64; 2]>| {
        let mut score = 0;
        for (gate_i, gate) in circuit.gates.iter().enumerate() {
            score += (locations[gate_i][0] as i32
                - (gate
                    .inputs()
                    .into_iter()
                    .map(|input| match input {
                        crate::circuit::Value::Arg(_) => 0,
                        crate::circuit::Value::GateValue(g, _) => locations[g][0] as i32,
                    })
                    .max()
                    .unwrap_or(0)
                    + 100))
                .abs();
            score += std::cmp::max((locations[gate_i][1] as i32 - 360).abs() - 100, 0);
        }
        score
    };
    possiblilities.sort_by_cached_key(score);
    possiblilities.remove(0)
}
