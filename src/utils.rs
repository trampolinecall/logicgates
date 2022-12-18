pub fn enumerate_inputs(arity: usize) -> Vec<Vec<bool>> {
    let mut inputs = vec![vec![]];
    for _ in 0..arity {
        let mut inputs_false = inputs.clone();
        let mut inputs_true = inputs;

        inputs_false.iter_mut().for_each(|i| i.insert(0, false));
        inputs_true.iter_mut().for_each(|i| i.insert(0, true));

        inputs = inputs_false;
        inputs.extend(inputs_true);
    }
    inputs
}

#[cfg(test)]
mod test {
    #[test]
    fn enumerate_inputs() {
        assert_eq!(super::enumerate_inputs(2), vec![vec![false, false], vec![false, true], vec![true, false], vec![true, true]]);
    }
}
