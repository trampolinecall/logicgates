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
        self.0.get(id.0).expect("ArenaId should never be invalid")
    }

    pub(crate) fn get_mut(&mut self, id: Id<T>) -> &mut T {
        self.0.get_mut(id.0).expect("ArenaId should never be invalid")
    }

    pub(crate) fn convert_id<U>() -> impl Fn(Id<T>) -> Id<U> {
        |id| Id(id.0, std::marker::PhantomData)
    }

    pub(crate) fn transform_infallible<U>(self, mut op: impl FnMut(T) -> U) -> Arena<U> {
        Arena(self.0.into_iter().map(op).collect())
    }
    pub(crate) fn transform<U>(self, mut op: impl FnMut(T) -> Option<U>) -> Option<Arena<U>> {
        Some(Arena(self.0.into_iter().map(op).collect_all()?))
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
