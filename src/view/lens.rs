use std::marker::PhantomData;

pub(crate) trait Lens<A, B> {
    fn with<'a, R: 'a, F: FnOnce(&B) -> R>(&self, a: &A, f: F) -> R;
    fn with_mut<'a, R: 'a, F: FnOnce(&mut B) -> R>(&self, a: &mut A, f: F) -> R;
}

pub(crate) struct Closures<A, B, I: Fn(&A) -> &B, M: Fn(&mut A) -> &mut B> {
    immut: I,
    mut_: M,

    _phantom: PhantomData<fn(&A) -> &B>,
}

impl<A, B, I: Fn(&A) -> &B, M: Fn(&mut A) -> &mut B> Closures<A, B, I, M> {
    pub(crate) fn new(immut: I, mut_: M) -> Closures<A, B, I, M> {
        Closures { immut, mut_, _phantom: PhantomData }
    }
}
impl<A, B, I: Fn(&A) -> &B, M: Fn(&mut A) -> &mut B> Lens<A, B> for Closures<A, B, I, M> {
    fn with<'a, R: 'a, F: FnOnce(&B) -> R>(&self, a: &A, f: F) -> R {
        f((self.immut)(a))
    }

    fn with_mut<'a, R: 'a, F: FnOnce(&mut B) -> R>(&self, a: &mut A, f: F) -> R {
        f((self.mut_)(a))
    }
}
impl<A, B, I: Fn(&A) -> &B + Copy, M: Fn(&mut A) -> &mut B + Copy> Clone for Closures<A, B, I, M> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<A, B, I: Fn(&A) -> &B + Copy, M: Fn(&mut A) -> &mut B + Copy> Copy for Closures<A, B, I, M> {}

pub(crate) struct Compose<A, B, C, Lens1: Lens<A, B>, Lens2: Lens<B, C>> {
    a_b: Lens1,
    b_c: Lens2,

    _phantom: PhantomData<(fn(&A) -> &B, fn(&B) -> &C)>,
}
impl<A, B, C, Lens1: Lens<A, B>, Lens2: Lens<B, C>> Compose<A, B, C, Lens1, Lens2> {
    pub(crate) fn new(a_b: Lens1, b_c: Lens2) -> Compose<A, B, C, Lens1, Lens2> {
        Compose { a_b, b_c, _phantom: PhantomData }
    }
}
impl<A, B, C, Lens1: Lens<A, B>, Lens2: Lens<B, C>> Lens<A, C> for Compose<A, B, C, Lens1, Lens2> {
    fn with<'a, R: 'a, F: FnOnce(&C) -> R>(&self, a: &A, f: F) -> R {
        self.a_b.with(a, move |b| self.b_c.with(b, f))
    }

    fn with_mut<'a, R: 'a, F: FnOnce(&mut C) -> R>(&self, a: &mut A, f: F) -> R {
        self.a_b.with_mut(a, move |b| self.b_c.with_mut(b, f))
    }
}
impl<A, B, C, Lens1: Lens<A, B> + Clone, Lens2: Lens<B, C> + Clone> Clone for Compose<A, B, C, Lens1, Lens2> {
    fn clone(&self) -> Self {
        Compose { a_b: self.a_b.clone(), b_c: self.b_c.clone(), _phantom: self._phantom }
    }
}
impl<A, B, C, Lens1: Lens<A, B> + Copy, Lens2: Lens<B, C> + Copy> Copy for Compose<A, B, C, Lens1, Lens2> {}

pub(crate) struct Unit<T> {
    _phantom: PhantomData<fn(&T) -> ()>,
}

impl<T> Unit<T> {
    pub(crate) fn new() -> Unit<T> {
        Unit { _phantom: PhantomData }
    }
}
impl<T> Lens<T, ()> for Unit<T> {
    fn with<'a, R: 'a, F: FnOnce(&()) -> R>(&self, _: &T, f: F) -> R {
        f(&())
    }

    fn with_mut<'a, R: 'a, F: FnOnce(&mut ()) -> R>(&self, _: &mut T, f: F) -> R {
        f(&mut ())
    }
}
impl<T> Clone for Unit<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Unit<T> {}
