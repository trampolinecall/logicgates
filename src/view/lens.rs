use std::marker::PhantomData;

pub(crate) trait Lens<A, B> {
    fn with<'a, R: 'a, F: FnOnce(&B) -> R>(&self, a: &A, f: F) -> R;
    fn with_mut<'a, R: 'a, F: FnOnce(&mut B) -> R>(&self, a: &mut A, f: F) -> R;
}

// TODO: move all structs out and then have each function be associated on each of the struct types?

// TODO: split this into a copy version and a non-copy version
pub(crate) fn from_closures<A, B>(immut: impl Fn(&A) -> &B + Copy, mut_: impl Fn(&mut A) -> &mut B + Copy) -> impl Lens<A, B> + Copy {
    struct Closures<A, B, I: Fn(&A) -> &B, M: Fn(&mut A) -> &mut B> {
        immut: I,
        mut_: M,

        _phantom: PhantomData<fn(&A) -> &B>,
    }

    impl<A, B, I: Fn(&A) -> &B + Copy, M: Fn(&mut A) -> &mut B + Copy> Clone for Closures<A, B, I, M> {
        fn clone(&self) -> Self {
            Self { ..*self }
        }
    }
    impl<A, B, I: Fn(&A) -> &B + Copy, M: Fn(&mut A) -> &mut B + Copy> Copy for Closures<A, B, I, M> {}

    impl<A, B, I: Fn(&A) -> &B, M: Fn(&mut A) -> &mut B> Lens<A, B> for Closures<A, B, I, M> {
        fn with<'a, R: 'a, F: FnOnce(&B) -> R>(&self, a: &A, f: F) -> R {
            f((self.immut)(a))
        }

        fn with_mut<'a, R: 'a, F: FnOnce(&mut B) -> R>(&self, a: &mut A, f: F) -> R {
            f((self.mut_)(a))
        }
    }

    Closures { immut, mut_, _phantom: PhantomData }
}

pub(crate) fn compose<A, B, C>(a_b: impl Lens<A, B>, b_c: impl Lens<B, C>) -> impl Lens<A, C> {
    struct Composed<A, B, C, Lens1: Lens<A, B>, Lens2: Lens<B, C>> {
        a_b: Lens1,
        b_c: Lens2,

        _phantom: PhantomData<(fn(&A) -> &B, fn(&B) -> &C)>,
    }

    impl<A, B, C, Lens1: Lens<A, B>, Lens2: Lens<B, C>> Lens<A, C> for Composed<A, B, C, Lens1, Lens2> {
        fn with<'a, R: 'a, F: FnOnce(&C) -> R>(&self, a: &A, f: F) -> R {
            self.a_b.with(a, move |b| self.b_c.with(b, f))
        }

        fn with_mut<'a, R: 'a, F: FnOnce(&mut C) -> R>(&self, a: &mut A, f: F) -> R {
            self.a_b.with_mut(a, move |b| self.b_c.with_mut(b, f))
        }
    }

    impl<A, B, C, Lens1: Lens<A, B> + Clone, Lens2: Lens<B, C> + Clone> Clone for Composed<A, B, C, Lens1, Lens2> {
        fn clone(&self) -> Self {
            Composed { a_b: self.a_b, b_c: self.b_c, _phantom: self._phantom }
        }
    }
    impl<A, B, C, Lens1: Lens<A, B> + Copy, Lens2: Lens<B, C> + Copy> Copy for Composed<A, B, C, Lens1, Lens2> {}

    Composed { a_b, b_c, _phantom: PhantomData }
}

pub(crate) fn unit<T>() -> impl Lens<T, ()> + Copy {
    struct UnitLens<T> {
        _phantom: PhantomData<fn(&T) -> ()>,
    }

    impl<T> Clone for UnitLens<T> {
        fn clone(&self) -> Self {
            Self { ..*self }
        }
    }
    impl<T> Copy for UnitLens<T> {}

    impl<T> Lens<T, ()> for UnitLens<T> {
        fn with<'a, R: 'a, F: FnOnce(&()) -> R>(&self, _: &T, f: F) -> R {
            f(&())
        }

        fn with_mut<'a, R: 'a, F: FnOnce(&mut ()) -> R>(&self, _: &mut T, f: F) -> R {
            f(&mut ())
        }
    }

    UnitLens { _phantom: PhantomData }
}
