use std::marker::PhantomData;

pub(crate) trait Lens<A, B> {
    fn get<'a>(&self, a: &'a A) -> &'a B;
    fn get_mut<'a>(&self, a: &'a mut A) -> &'a mut B;
}

pub(crate) fn from_closures<A, B>(immut: impl Fn(&A) -> &B, mut_: impl Fn(&mut A) -> &mut B) -> impl Lens<A, B> {
    struct Closures<A, B, I: Fn(&A) -> &B, M: Fn(&mut A) -> &mut B> {
        immut: I,
        mut_: M,

        _phantom: PhantomData<fn(&A) -> &B>,
    }
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
