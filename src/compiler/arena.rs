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

    /* (unused)
    pub(crate) fn get_mut(&mut self, id: Id) -> &mut T {
        self.0.get_mut(id.get()).expect("arena Id should never be invalid")
    }
    */

    pub(crate) fn transform<U>(self, op: impl FnMut(T) -> Option<U>) -> Option<Arena<U, Id>>
    where
        Id: IsArenaIdFor<U>,
    {
        Some(Arena(self.0.into_iter().map(op).collect_all()?, std::marker::PhantomData))
    }
    pub(crate) fn transform_infallible<U>(self, op: impl FnMut(T) -> U) -> Arena<U, Id>
    where
        Id: IsArenaIdFor<U>,
    {
        Arena(self.0.into_iter().map(op).collect(), std::marker::PhantomData)
    }

    /* (unused)
    pub(crate) fn new_item_with_id(&mut self, make: impl FnOnce(Id) -> T) -> Id {
        let id = Id::make(self.0.len());
        self.0.push(make(id));
        id
    }
    */

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
    /* (unused)
    pub(crate) fn iter_with_ids(&self) -> impl Iterator<Item = (Id, &T)> + '_ {
        self.0.iter().enumerate().map(|(i, thing)| (Id::make(i), thing))
    }
    */
}

// dependant annotation things {{{1
#[macro_use]
mod dependant_annotation {
    use super::{ArenaId, IsArenaIdFor};

    pub(super) enum ItemState<Annotation, Id, Error> {
        Waiting,
        WaitingOn(Id),
        Ok(Annotation),
        Error(Error),
        ErrorInDep,
    }

    impl<Annotation, Id, Error> ItemState<Annotation, Id, Error> {
        pub(super) fn needs_annotation(&self) -> bool {
            matches!(self, Self::Waiting | Self::WaitingOn(..))
        }

        pub(super) fn is_waiting_on(&self) -> bool {
            matches!(self, Self::WaitingOn(..))
        }
    }

    pub(crate) struct DependancyGetter<'arena, Annotation, Original, Error, Id>(pub(super) &'arena Vec<(Original, ItemState<Annotation, Id, Error>)>);
    impl<Annotation, Original, Error, Id> Copy for DependancyGetter<'_, Annotation, Original, Error, Id> {}
    impl<Annotation, Original, Error, Id> Clone for DependancyGetter<'_, Annotation, Original, Error, Id> {
        fn clone(&self) -> Self {
            Self(self.0)
        }
    }
    impl<'arena, Annotation, Original, Error, Id: ArenaId + IsArenaIdFor<Original>> DependancyGetter<'arena, Annotation, Original, Error, Id> {
        pub(crate) fn get(&self, id: Id) -> SingleTransformResult<(&'arena Original, &'arena Annotation), Id, Error> {
            match self.0.get(id.get()).expect("arena Id should not be invalid") {
                (original, ItemState::Ok(item)) => SingleTransformResult::Ok((original, item)),
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
    pub(crate) enum SingleTransformResult<Annotation, Id, Error> {
        Ok(Annotation),
        Dep(DependencyError<Id>),
        Err(Error),
    }

    pub(crate) struct LoopError;

    macro_rules! try_annotation_result {
        ($e:expr) => {
            match $e {
                $crate::compiler::arena::SingleTransformResult::Ok(r) => r,
                $crate::compiler::arena::SingleTransformResult::Dep(d) => return $crate::compiler::arena::SingleTransformResult::Dep(d),
                $crate::compiler::arena::SingleTransformResult::Err(e) => return $crate::compiler::arena::SingleTransformResult::Err(e),
            }
        };
    }
}
pub(crate) use dependant_annotation::DependancyGetter;
pub(crate) use dependant_annotation::LoopError;
pub(crate) use dependant_annotation::SingleTransformResult;
impl<Original, Id: ArenaId + IsArenaIdFor<Original>> Arena<Original, Id> {
    // TODO: write tests for this
    pub(crate) fn annotate_dependant<Annotation, New, Error>(
        self,
        mut try_convert: impl FnMut(&Original, DependancyGetter<Annotation, Original, Error, Id>) -> SingleTransformResult<Annotation, Id, Error>,
        incorporate_annotation: impl Fn(Original, Annotation) -> New,
    ) -> Result<Arena<New, Id>, (Vec<LoopError>, Vec<Error>)>
    where
        Id: IsArenaIdFor<New>,
    {
        use dependant_annotation::*;
        // some transformations / annotations have operations that are dependant on the results of other annotations
        // for example in name resolution, the result of a single name might be dependant on the resolution of another name
        // another example is in calculating the sizes of types, the size of a single type might be dependant on the sizes of other types (for example the size of a product type is dependant on the sizes of each of its child types)
        // this method holds the logic for allowing the operations to happen in a centralized place so that it does not need to be copied and pasted around to every part that needs it (not only because of dry but also because some of the logic, for example the loop detection logic, is annoying to constantly reimplement)

        let mut things: Vec<_> = self.0.into_iter().map(|item| (item, ItemState::Waiting::<Annotation, Id, Error>)).collect();

        loop {
            if things.iter().all(|thing| !thing.1.needs_annotation()) {
                // all of the things are either done or errored
                break;
            }

            if things.iter().filter(|thing| thing.1.needs_annotation()).all(|thing| thing.1.is_waiting_on()) {
                // the things that are not done are all waiting on something else, which is a loop
                todo!("loop") // TODO: mark all items in loop as loop, so that they are not waiting, continue
            }

            for thing_i in 0..things.len() {
                let thing = things.get(thing_i).expect("iterating through things by index should not be out of range");
                if let ItemState::Waiting | ItemState::WaitingOn(_) = thing.1 {
                    let converted = try_convert(&thing.0, DependancyGetter(&things));

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

        for (original, annotation) in things.into_iter() {
            match annotation {
                ItemState::Waiting | ItemState::WaitingOn(_) => unreachable!("item waiting after main loop in dependant annotation"),
                ItemState::Ok(annotation) => {
                    if let Some(ref mut final_things) = final_things {
                        final_things.push(incorporate_annotation(original, annotation))
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
