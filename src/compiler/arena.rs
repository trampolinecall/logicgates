use crate::utils::CollectAll;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Arena<T>(Vec<T>);
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Id<T>(usize, std::marker::PhantomData<T>);

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}
impl<T> Copy for Id<T> {}

impl<T> Arena<T> {
    pub(crate) fn new() -> Arena<T> {
        Arena(Vec::new())
    }

    pub(crate) fn add(&mut self, thing: T) -> Id<T> {
        self.0.push(thing);
        Id(self.0.len() - 1, std::marker::PhantomData)
    }

    pub(crate) fn get(&self, id: Id<T>) -> &T {
        self.0.get(id.0).expect("arena Id should never be invalid")
    }

    pub(crate) fn get_mut(&mut self, id: Id<T>) -> &mut T {
        self.0.get_mut(id.0).expect("arena Id should never be invalid")
    }

    fn convert_id<U>(id: Id<T>) -> Id<U> {
        Id(id.0, std::marker::PhantomData)
    }

    pub(crate) fn transform<U>(self, mut op: impl FnMut(T, fn(Id<T>) -> Id<U>) -> Option<U>) -> Option<(Arena<U>, fn(Id<T>) -> Id<U>)> {
        Some((Arena(self.0.into_iter().map(|thing| op(thing, Self::convert_id)).collect_all()?), Self::convert_id))
    }
    pub(crate) fn transform_infallible<U>(self, mut op: impl FnMut(T, fn(Id<T>) -> Id<U>) -> U) -> (Arena<U>, fn(Id<T>) -> Id<U>) {
        (Arena(self.0.into_iter().map(|thing| op(thing, Self::convert_id)).collect()), Self::convert_id)
    }

    pub(crate) fn new_item_with_id(&mut self, make: impl FnOnce(Id<T>) -> T) -> Id<T> {
        let id = Id(self.0.len(), std::marker::PhantomData);
        self.0.push(make(id));
        id
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
    pub(crate) fn iter_with_ids(&self) -> impl Iterator<Item = (Id<T>, &T)> + '_ {
        self.0.iter().enumerate().map(|(i, thing)| (Id(i, std::marker::PhantomData), thing))
    }
}

// dependant transform things {{{1
#[macro_use]
mod dependant_transform {
    use super::{Arena, Id};

    pub(super) enum ItemState<New, Old, Error> {
        Waiting,
        WaitingOn(Id<Old>),
        Ok(New),
        Error(Error),
        ErrorInDep,
    }

    impl<New, Old, Error> ItemState<New, Old, Error> {
        pub(super) fn needs_transformation(&self) -> bool {
            matches!(self, Self::Waiting | Self::WaitingOn(..))
        }

        pub(super) fn is_done(&self) -> bool {
            matches!(self, Self::Ok(..) | Self::Error(..) | Self::ErrorInDep)
        }

        pub(super) fn is_waiting_on(&self) -> bool {
            matches!(self, Self::WaitingOn(..))
        }
    }

    pub(crate) struct DependancyGetter<'arena, New, Old, Error>(pub(super) &'arena Arena<(Old, ItemState<New, Old, Error>)>);
    impl<New, Old, Error> Copy for DependancyGetter<'_, New, Old, Error> {}
    impl<New, Old, Error> Clone for DependancyGetter<'_, New, Old, Error> {
        fn clone(&self) -> Self {
            Self(self.0)
        }
    }
    impl<'arena, New, Old, Error> DependancyGetter<'arena, New, Old, Error> {
        pub(crate) fn get_dep(&self, id: Id<Old>) -> SingleTransformResult<&'arena New, Old, Error> {
            match self.0 .0.get(id.0).expect("arena Id should not be invalid") {
                (_, ItemState::Ok(item)) => SingleTransformResult::Ok(item),
                (_, ItemState::Waiting) | (_, ItemState::WaitingOn(_)) => SingleTransformResult::Dep(DependencyError(DependencyErrorKind::WaitingOn(id))),
                (_, ItemState::Error(_)) | (_, ItemState::ErrorInDep) => SingleTransformResult::Dep(DependencyError(DependencyErrorKind::ErrorInDep)),
            }
        }
    }

    pub(crate) struct DependencyError<Old>(pub(super) DependencyErrorKind<Old>);
    pub(super) enum DependencyErrorKind<Old> {
        WaitingOn(Id<Old>),
        ErrorInDep,
    }
    pub(crate) enum SingleTransformResult<New, Old, Error> {
        Ok(New),
        Dep(DependencyError<Old>),
        Err(Error),
    }

    pub(crate) struct LoopError;

    macro_rules! try_transform_result {
        ($e:expr) => {
            match $e {
                $crate::compiler::arena::SingleTransformResult::Ok(r) => r,
                $crate::compiler::arena::SingleTransformResult::Dep(d) => return $crate::compiler::arena::SingleTransformResult::Dep(d),
                $crate::compiler::arena::SingleTransformResult::Err(e) => return $crate::compiler::arena::SingleTransformResult::Err(e),
            }
        };
    }
}
pub(crate) use dependant_transform::DependancyGetter;
pub(crate) use dependant_transform::LoopError;
pub(crate) use dependant_transform::SingleTransformResult;
impl<Old> Arena<Old> {
    // TODO: write tests for this
    pub(crate) fn transform_dependant<New, Error>(
        self,
        // TODO: make the thing not have to take a &Old
        mut try_convert: impl FnMut(&Old, DependancyGetter<New, Old, Error>, fn(Id<Old>) -> Id<New>) -> SingleTransformResult<New, Old, Error>,
    ) -> Result<(Arena<New>, fn(Id<Old>) -> Id<New>), (Vec<LoopError>, Vec<Error>)> {
        use dependant_transform::*;
        // some transformations have operations that are dependant on the results of other transformations
        // for example in name resolution, the result of a single name might be dependant on the resolution of another name
        // another example is in calculating the sizes of types, the size of a single type might be dependant on the sizes of other types (for example the size of a product type is dependant on the sizes of each of its child types)
        // this method holds the logic for allowing the operations to happen in a centralized place so that it does not need to be copied and pasted around to every part that needs it (not only because of dry but also because some of the logic, for example the loop detection logic, is annoying to implement)

        let (mut things, _) = self.transform_infallible(|item, _| (item, ItemState::Waiting::<New, Old, Error>));

        loop {
            if things.iter().all(|thing| !thing.1.needs_transformation()) {
                // all of the things are either done or errored
                break;
            }

            if things.iter().filter(|thing| thing.1.needs_transformation()).all(|thing| thing.1.is_waiting_on()) {
                // the things that are not done are all waiting on something else, which is a loop
                todo!("loop") // TODO: mark all items in loop as loop, so that they are not waiting, continue
            }

            for thing_i in 0..things.0.len() {
                let thing = things.0.get(thing_i).expect("iterating through things by index should not be out of range");
                if let ItemState::Waiting | ItemState::WaitingOn(_) = thing.1 {
                    let converted = try_convert(&thing.0, DependancyGetter(&things), Self::convert_id);

                    let thing_mut = things.0.get_mut(thing_i).expect("iterating through things by index should not be out of range");
                    thing_mut.1 = match converted {
                        SingleTransformResult::Ok(new) => ItemState::Ok(new),
                        SingleTransformResult::Dep(DependencyError(DependencyErrorKind::WaitingOn(id))) => ItemState::WaitingOn(id),
                        SingleTransformResult::Dep(DependencyError(DependencyErrorKind::ErrorInDep)) => ItemState::ErrorInDep,
                        SingleTransformResult::Err(other_error) => ItemState::Error(other_error),
                    };
                }
            }
        }

        let mut final_things = Some(Vec::new());
        let mut errors = Vec::new();
        let loops = Vec::new(); // TODO

        for thing in things.0.into_iter() {
            match thing.1 {
                ItemState::Waiting | ItemState::WaitingOn(_) => unreachable!("item waiting after main loop in dependant transform"),
                ItemState::Ok(thing) => {
                    if let Some(ref mut final_things) = final_things {
                        final_things.push(thing)
                    }
                }
                ItemState::Error(error) => {
                    errors.push(error);
                    final_things = None;
                }
                ItemState::ErrorInDep => {
                    final_things = None;
                }
            }
        }

        if !errors.is_empty() || !loops.is_empty() {
            Err((loops, errors))
        } else {
            Ok((Arena(final_things.expect("problem in final_things but no errors and no loops")), Self::convert_id))
        }
    }
}
