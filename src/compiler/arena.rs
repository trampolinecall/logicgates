use crate::utils::CollectAll;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Arena<T, Id: ArenaId + IsArenaIdFor<T>>(Vec<T>, std::marker::PhantomData<Id>);

pub(crate) trait ArenaId: Copy {
    fn make(i: usize) -> Self;
    fn get(&self) -> usize;
}

pub(crate) trait IsArenaIdFor<T>: ArenaId {}

impl<T, Id: ArenaId + IsArenaIdFor<T>> Arena<T, Id> {
    pub(crate) fn new() -> Arena<T, Id> {
        Arena(Vec::new(), std::marker::PhantomData)
    }

    pub(crate) fn add(&mut self, thing: T) -> Id {
        self.0.push(thing);
        Id::make(self.0.len() - 1)
    }

    pub(crate) fn get(&self, id: Id) -> &T {
        self.0.get(id.get()).expect("arena Id should never be invalid")
    }

    pub(crate) fn get_mut(&mut self, id: Id) -> &mut T {
        self.0.get_mut(id.get()).expect("arena Id should never be invalid")
    }

    pub(crate) fn transform<U>(self, mut op: impl FnMut(T) -> Option<U>) -> Option<Arena<U, Id>>
    where
        Id: IsArenaIdFor<U>,
    {
        Some(Arena(self.0.into_iter().map(|thing| op(thing)).collect_all()?, std::marker::PhantomData))
    }
    pub(crate) fn transform_infallible<U>(self, mut op: impl FnMut(T) -> U) -> Arena<U, Id>
    where
        Id: IsArenaIdFor<U>,
    {
        Arena(self.0.into_iter().map(|thing| op(thing)).collect(), std::marker::PhantomData)
    }

    pub(crate) fn new_item_with_id(&mut self, make: impl FnOnce(Id) -> T) -> Id {
        let id = Id::make(self.0.len());
        self.0.push(make(id));
        id
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
    pub(crate) fn iter_with_ids(&self) -> impl Iterator<Item = (Id, &T)> + '_ {
        self.0.iter().enumerate().map(|(i, thing)| (Id::make(i), thing))
    }
}

// dependant transform things {{{1
#[macro_use]
mod dependant_transform {
    use super::{ArenaId, IsArenaIdFor};

    pub(super) enum ItemState<New, Id, Error> {
        Waiting,
        WaitingOn(Id),
        Ok(New),
        Error(Error),
        ErrorInDep,
    }

    impl<New, Id, Error> ItemState<New, Id, Error> {
        pub(super) fn needs_transformation(&self) -> bool {
            matches!(self, Self::Waiting | Self::WaitingOn(..))
        }

        pub(super) fn is_waiting_on(&self) -> bool {
            matches!(self, Self::WaitingOn(..))
        }
    }

    pub(crate) struct DependancyGetter<'arena, New, Old, Error, Id>(pub(super) &'arena Vec<(Old, ItemState<New, Id, Error>)>);
    impl<New, Old, Error, Id> Copy for DependancyGetter<'_, New, Old, Error, Id> {}
    impl<New, Old, Error, Id> Clone for DependancyGetter<'_, New, Old, Error, Id> {
        fn clone(&self) -> Self {
            Self(self.0)
        }
    }
    impl<'arena, New, Old, Error, Id: ArenaId + IsArenaIdFor<Old> + IsArenaIdFor<New>> DependancyGetter<'arena, New, Old, Error, Id> {
        pub(crate) fn get_dep(&self, id: Id) -> SingleTransformResult<&'arena New, Id, Error> {
            match self.0.get(id.get()).expect("arena Id should not be invalid") {
                (_, ItemState::Ok(item)) => SingleTransformResult::Ok(item),
                (_, ItemState::Waiting) | (_, ItemState::WaitingOn(_)) => SingleTransformResult::Dep(DependencyError(DependencyErrorKind::WaitingOn(id))),
                (_, ItemState::Error(_)) | (_, ItemState::ErrorInDep) => SingleTransformResult::Dep(DependencyError(DependencyErrorKind::ErrorInDep)),
            }
        }
    }

    pub(crate) struct DependencyError<Id>(pub(super) DependencyErrorKind<Id>);
    pub(super) enum DependencyErrorKind<Id> {
        WaitingOn(Id),
        ErrorInDep,
    }
    pub(crate) enum SingleTransformResult<New, Id, Error> {
        Ok(New),
        Dep(DependencyError<Id>),
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
impl<Old, Id: ArenaId + IsArenaIdFor<Old>> Arena<Old, Id> {
    // TODO: write tests for this
    pub(crate) fn transform_dependant<New, Error>(
        self,
        // TODO: make the thing not have to take a &Old
        mut try_convert: impl FnMut(&Old, DependancyGetter<New, Old, Error, Id>) -> SingleTransformResult<New, Id, Error>,
    ) -> Result<Arena<New, Id>, (Vec<LoopError>, Vec<Error>)>
    where
        Id: IsArenaIdFor<New>,
    {
        use dependant_transform::*;
        // some transformations have operations that are dependant on the results of other transformations
        // for example in name resolution, the result of a single name might be dependant on the resolution of another name
        // another example is in calculating the sizes of types, the size of a single type might be dependant on the sizes of other types (for example the size of a product type is dependant on the sizes of each of its child types)
        // this method holds the logic for allowing the operations to happen in a centralized place so that it does not need to be copied and pasted around to every part that needs it (not only because of dry but also because some of the logic, for example the loop detection logic, is annoying to implement)

        let mut things: Vec<_> = self.0.into_iter().map(|item| (item, ItemState::Waiting::<New, Id, Error>)).collect();

        loop {
            if things.iter().all(|thing| !thing.1.needs_transformation()) {
                // all of the things are either done or errored
                break;
            }

            if things.iter().filter(|thing| thing.1.needs_transformation()).all(|thing| thing.1.is_waiting_on()) {
                // the things that are not done are all waiting on something else, which is a loop
                todo!("loop") // TODO: mark all items in loop as loop, so that they are not waiting, continue
            }

            for thing_i in 0..things.len() {
                let thing = things.get(thing_i).expect("iterating through things by index should not be out of range");
                if let ItemState::Waiting | ItemState::WaitingOn(_) = thing.1 {
                    let converted = try_convert(&thing.0, DependancyGetter(&things), );

                    let thing_mut = things.get_mut(thing_i).expect("iterating through things by index should not be out of range");
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

        for thing in things.into_iter() {
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
            Ok(Arena(final_things.expect("problem in final_things but no errors and no loops"), std::marker::PhantomData))
        }
    }
}
