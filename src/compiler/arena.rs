use crate::utils::CollectAll;

#[derive(Debug)]
pub(crate) struct Arena<T>(Vec<T>);
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ArenaId<T>(usize, std::marker::PhantomData<T>);

impl<T> Clone for ArenaId<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}
impl<T> Copy for ArenaId<T> {}

impl<T> Arena<T> {
    pub(crate) fn new() -> Arena<T> {
        Arena(Vec::new())
    }

    pub(crate) fn new_item(&mut self, thing: T) -> ArenaId<T> {
        self.0.push(thing);
        ArenaId(self.0.len() - 1, std::marker::PhantomData)
    }

    pub(crate) fn get(&self, id: ArenaId<T>) -> &T {
        self.0.get(id.0).expect("ArenaId should never be invalid")
    }

    pub(crate) fn get_mut(&mut self, id: ArenaId<T>) -> &mut T {
        self.0.get_mut(id.0).expect("ArenaId should never be invalid")
    }

    pub(crate) fn convert_id<U>() -> impl Fn(ArenaId<T>) -> ArenaId<U> {
        |id| ArenaId(id.0, std::marker::PhantomData)
    }

    pub(crate) fn transform_infallible<U>(self, mut op: impl FnMut(T) -> U) -> Arena<U> {
        Arena(self.0.into_iter().map(op).collect())
    }
    pub(crate) fn transform<U>(self, mut op: impl FnMut(T) -> Option<U>) -> Option<Arena<U>> {
        Some(Arena(self.0.into_iter().map(op).collect_all()?))
    }

    pub(crate) fn new_item_with_id(&mut self, make: impl FnOnce(ArenaId<T>) -> T) -> ArenaId<T> {
        let id = ArenaId(self.0.len(), std::marker::PhantomData);
        self.0.push(make(id));
        id
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
    pub(crate) fn iter_with_ids(&self) -> impl Iterator<Item = (ArenaId<T>, &T)> + '_ {
        self.0.iter().enumerate().map(|(i, thing)| (ArenaId(i, std::marker::PhantomData), thing))
    }
}
