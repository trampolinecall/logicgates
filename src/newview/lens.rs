use std::marker::PhantomData;

pub(crate) trait Lens<A, B> {
    fn get<'a>(&self, a: &'a A) -> &'a B;
    fn get_mut<'a>(&self, a: &'a mut A) -> &'a mut B;
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
        fn get<'a>(&self, a: &'a A) -> &'a B {
            (self.immut)(a)
        }

        fn get_mut<'a>(&self, a: &'a mut A) -> &'a mut B {
            (self.mut_)(a)
        }
    }

    Closures { immut, mut_, _phantom: PhantomData }
}
