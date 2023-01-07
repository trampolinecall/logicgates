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
    pub(crate) fn iter_with_ids(&self) -> impl Iterator<Item = (Id, &T)> + '_ {
        self.0.iter().enumerate().map(|(i, thing)| (Id::make(i), thing))
    }
}

// dependant transform things {{{1
#[macro_use]
mod dependant_transform {
    use super::{ArenaId, IsArenaIdFor};

    pub(super) enum ItemState<Annotation, Id, Error> {
        Waiting,
        WaitingOn(Id),
        Ok(Annotation),
        Error(Error),
        ErrorInDep,
    }

    impl<Annotation, Id, Error> ItemState<Annotation, Id, Error> {
        pub(super) fn needs_transform(&self) -> bool {
            matches!(self, Self::Waiting | Self::WaitingOn(..))
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
pub(crate) use dependant_transform::SingleTransformResult;
impl<Original, Id: ArenaId + IsArenaIdFor<Original> + PartialEq> Arena<Original, Id> {
    // TODO: write tests for this
    pub(crate) fn transform_dependant_with_id<Annotation, New, Error>(
        self,
        mut try_annotate: impl FnMut(Id, &Original, DependancyGetter<Annotation, Original, Error, Id>) -> SingleTransformResult<Annotation, Id, Error>,
        incorporate_annotation: impl Fn(Original, Annotation) -> New,
    ) -> Result<Arena<New, Id>, (Vec<Vec<Original>>, Vec<Error>)>
    where
        Id: IsArenaIdFor<New>,
    {
        use dependant_transform::*;
        // some transformations have operations that are dependant on the results of other transformations
        // for example in name resolution, the result of a single name might be dependant on the resolution of another name
        // another example is in calculating the sizes of types, the size of a single type might be dependant on the sizes of other types (for example the size of a product type is dependant on the sizes of each of its child types)
        // this method holds the logic for allowing the operations to happen in a centralized place so that it does not need to be copied and pasted around to every part that needs it (not only because of dry but also because some of the logic, for example the loop detection logic, is annoying to constantly reimplement)

        let mut things: Vec<_> = self.0.into_iter().map(|item| (item, ItemState::Waiting::<Annotation, Id, Error>)).collect();

        let loops = loop {
            if things.iter().all(|thing| !thing.1.needs_transform()) {
                // all of the things are either done or errored
                // calling .all on an empty iterator returns true so this will be done if it is empty
                break Vec::new(); // completed successfully, return no loop errors
            }

            let mut amt_changed = 0; // TODO: test counting

            for thing_i in 0..things.len() {
                let thing = &things[thing_i];
                if thing.1.needs_transform() {
                    let annotation = try_annotate(Id::make(thing_i), &thing.0, DependancyGetter(&things));

                    let new_annotation_state = match annotation {
                        SingleTransformResult::Ok(new) => {
                            amt_changed += 1;
                            ItemState::Ok(new)
                        }
                        SingleTransformResult::Dep(DependencyError(DependencyErrorKind::WaitingOn(id))) => {
                            if let ItemState::WaitingOn(old_waiting_on) = thing.1 {
                                if id != old_waiting_on {
                                    amt_changed += 1;
                                }
                            }

                            ItemState::WaitingOn(id)
                        }
                        SingleTransformResult::Dep(DependencyError(DependencyErrorKind::ErrorInDep)) => {
                            amt_changed += 1;
                            ItemState::ErrorInDep
                        }
                        SingleTransformResult::Err(other_error) => {
                            amt_changed += 1;
                            ItemState::Error(other_error)
                        }
                    };
                    let thing_mut = things.get_mut(thing_i).expect("iterating through things by index should not be out of range");
                    thing_mut.1 = new_annotation_state;
                }
            }

            if amt_changed == 0 && things.iter().filter(|thing| thing.1.needs_transform()).count() > 0 {
                // if nothing has changed and there are still items to process
                // TODO: test this
                struct CmpByPtr<'a, T>(&'a T);

                impl<T> Copy for CmpByPtr<'_, T> {}
                impl<T> Clone for CmpByPtr<'_, T> {
                    fn clone(&self) -> Self {
                        Self(self.0)
                    }
                }
                impl<T> Eq for CmpByPtr<'_, T> {}
                impl<T> PartialEq for CmpByPtr<'_, T> {
                    fn eq(&self, other: &Self) -> bool {
                        std::ptr::eq(self.0, other.0)
                    }
                }

                let mut loops = Vec::new();
                let mut waiting_nodes = things
                    .iter()
                    .enumerate()
                    .filter_map(|(i, thing)| match thing.1 {
                        ItemState::Waiting => unreachable!("the loop above will always convert any Waiting to something else (even if just WaitingOn)"),
                        ItemState::WaitingOn(_) => Some(i),
                        ItemState::Ok(_) | ItemState::Error(_) | ItemState::ErrorInDep => None,
                    })
                    .collect::<Vec<_>>();

                'each_loop: while let Some(&(mut cur)) = waiting_nodes.first() {
                    let mut cur_loop = Vec::new();

                    // travel around the loop until a cycle is completed
                    loop {
                        cur_loop.push(cur);
                        cur = if let ItemState::WaitingOn(dep_id) = things[cur].1 {
                            if !waiting_nodes.contains(&dep_id.get()) {
                                // this node leads to the same loop as before
                                continue 'each_loop;
                            } else {
                                // travel to this nodes dependency
                                dep_id.get()
                            }
                        } else {
                            unreachable!("waiting_nodes only contains WaitingOn items")
                        };

                        if cur_loop.contains(&cur) {
                            // completed the cycle around the loop
                            break;
                        }
                    }

                    waiting_nodes.retain(|item| !cur_loop.contains(item));
                    // TODO: remove other nodes leading into loop
                    loops.push(cur_loop);
                }
                break loops; // cannot continue because if none of them are changing, they are all dependant on the loop
            }
        };

        let has_errors = things.iter().any(|thing| matches!(thing.1, ItemState::Error(_) | ItemState::ErrorInDep));
        if has_errors || !loops.is_empty() {
            let mut errors = Vec::new();
            let mut things: Vec<_> = things
                .into_iter()
                .map(|thing| {
                    if let ItemState::Error(err) = thing.1 {
                        errors.push(err);
                        None
                    } else {
                        Some(thing)
                    }
                })
                .collect();

            let loops = loops.into_iter().map(|loop_| loop_.into_iter().map(|loop_i| things[loop_i].take().expect("different loops should not contain the same element").0).collect()).collect();

            Err((loops, errors))
        } else {
            let final_things = things.into_iter().map(|(original, annotation)| {
                match annotation {
                    ItemState::Waiting | ItemState::WaitingOn(_) => unreachable!("this arm can only match if there is a loop that leaves items in the waiting state, but if that did happen then this else branch wouldn't run because there would be loop errors"),
                    ItemState::Ok(annotation) => {
                            incorporate_annotation(original, annotation)
                    }
                    ItemState::Error(_) | ItemState::ErrorInDep =>
                        unreachable!("errors were filtered out by the has_errors condition")
                    ,
                }
            }).collect();

            Ok(Arena(final_things, std::marker::PhantomData))
        }
    }

    pub(crate) fn transform_dependant<Annotation, New, Error>(
        self,
        mut try_convert: impl FnMut(&Original, DependancyGetter<Annotation, Original, Error, Id>) -> SingleTransformResult<Annotation, Id, Error>,
        incorporate_annotation: impl Fn(Original, Annotation) -> New,
    ) -> Result<Arena<New, Id>, (Vec<Vec<Original>>, Vec<Error>)>
    where
        Id: IsArenaIdFor<New>,
    {
        self.transform_dependant_with_id(|_, thing, dependency_getter| try_convert(thing, dependency_getter), incorporate_annotation)
    }
}
