// TODO: there is probably a better way of doing this
pub(crate) trait CanCollectAll {
    // TODO: figure out how to make this private
    type Output<Collection>;
    type Item;
    fn collect_all<F: FromIterator<Self::Item>>(iter: impl Iterator<Item = Self>) -> Self::Output<F>;
}

pub(crate) trait CollectAll {
    type Item;
    type Output<Collection>;
    fn collect_all<F: FromIterator<Self::Item>>(self) -> Self::Output<F>;
}

impl<I: CanCollectAll, T: Iterator<Item = I>> CollectAll for T {
    type Item = <I as CanCollectAll>::Item;
    type Output<Collection> = I::Output<Collection>;

    fn collect_all<F: FromIterator<Self::Item>>(self) -> Self::Output<F> {
        CanCollectAll::collect_all(self)
    }
}

impl<T> CanCollectAll for Option<T> {
    type Output<Collection> = Option<Collection>;
    type Item = T;

    fn collect_all<F: FromIterator<Self::Item>>(iter: impl Iterator<Item = Self>) -> Self::Output<F> {
        // collect into a vec first to evaluate all of the things and then collect all of the results to stop at the frist error
        iter.collect::<Vec<_>>().into_iter().collect()
    }
}

impl<R, E> CanCollectAll for Result<R, Vec<E>> {
    type Output<Collection> = Result<Collection, Vec<E>>;

    type Item = R;

    fn collect_all<F: std::iter::FromIterator<R>>(iter: impl Iterator<Item = Self>) -> Self::Output<F> {
        // TODO: make a better implementation of this
        let mut results = Vec::new();
        let mut errors = Vec::new();
        for item in iter {
            match item {
                Ok(o) => results.push(o),
                Err(e) => errors.extend(e),
            }
        }

        if errors.is_empty() {
            Ok(results.into_iter().collect())
        } else {
            Err(errors)
        }
    }
}
