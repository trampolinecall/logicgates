// TODO: there is probably a better way of doing this
pub(crate) trait CanCollectAll { // TODO: figure out how to make this private
    type Result;
    fn collect_all(iter: impl Iterator<Item = Self>) -> Self::Result;
}

pub(crate) trait CollectAll {
    type Result;
    fn collect_all(self) -> Self::Result;
}

impl<I: CanCollectAll, T: Iterator<Item = I>> CollectAll for T {
    type Result = I::Result;

    fn collect_all(self) -> Self::Result {
        CanCollectAll::collect_all(self)
    }
}

impl<T> CanCollectAll for Option<T> {
    type Result = Option<Vec<T>>;

    fn collect_all(iter: impl Iterator<Item = Self>) -> Self::Result {
        // collect into a vec first to evaluate all of the things and then collect all of the results to stop at the frist error
        iter.collect::<Vec<_>>().into_iter().collect()
    }
}

impl<R, E> CanCollectAll for Result<R, E> {
    type Result = Result<Vec<R>, Vec<E>>;

    fn collect_all(iter: impl Iterator<Item = Self>) -> Self::Result {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        for item in iter {
            match item {
                Ok(o) => results.push(o),
                Err(e) => errors.push(e),
            }
        }

        if errors.is_empty() {
            Ok(results)
        } else {
            Err(errors)
        }
    }
}

pub(crate) fn enumerate_inputs(arity: usize) -> Vec<Vec<bool>> {
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
        assert_eq!(
            super::enumerate_inputs(3),
            vec![
                vec![false, false, false],
                vec![false, false, true],
                vec![false, true, false],
                vec![false, true, true],
                vec![true, false, false],
                vec![true, false, true],
                vec![true, true, false],
                vec![true, true, true]
            ]
        );
    }
}
