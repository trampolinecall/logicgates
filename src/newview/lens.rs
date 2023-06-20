use std::marker::PhantomData;

pub(crate) trait Lens<A, B> {
    fn with<R, F: FnOnce(&B) -> R>(&self, a: &A, f: F) -> R;
    fn with_mut<R, F: FnOnce(&mut B) -> R>(&self, a: &mut A, f: F) -> R;
}

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
        fn with<R, F: FnOnce(&B) -> R>(&self, a: &A, f: F) -> R {
            f((self.immut)(a))
        }

        fn with_mut<R, F: FnOnce(&mut B) -> R>(&self, a: &mut A, f: F) -> R {
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
        fn with<R, F: FnOnce(&C) -> R>(&self, a: &A, f: F) -> R {
            self.a_b.with(a, move |b| self.b_c.with(b, f))
        }

        fn with_mut<R, F: FnOnce(&mut C) -> R>(&self, a: &mut A, f: F) -> R {
            self.a_b.with_mut(a, move |b| self.b_c.with_mut(b, f))
        }
    }

    Composed { a_b, b_c, _phantom: PhantomData }
}
