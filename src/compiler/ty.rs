use symtern::prelude::*;

pub(crate) struct Types {
    types: symtern::Pool<Type>, // ideally, i would use a interner crate that doesnt use ids to access types but they dont handle cyclic references nicely
}
pub(crate) type TypeSym = symtern::Sym<usize>;
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) enum Type {
    Bit,
    Product(Vec<(String, TypeSym)>),
}
impl Types {
    pub(crate) fn new() -> Self {
        Self { types: symtern::Pool::new() }
    }

    pub(crate) fn get(&self, sym: TypeSym) -> &Type {
        self.types.resolve(sym).expect("symtern resolution error")
    }

    pub(crate) fn intern(&mut self, ty: Type) -> TypeSym {
        self.types.intern(&ty).expect("symtern interning error")
    }
}
impl Type {
    pub(crate) fn size(&self, types: &Types) -> usize {
        match self {
            Type::Bit => 1,
            Type::Product(fields) => fields.iter().map(|(_, tyi)| types.get(*tyi).size(types)).sum(),
        }
    }

    // TODO: there is probably a better solution to this
    pub(crate) fn fmt(&self, types: &Types) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        match self {
            Type::Bit => write!(s, "'").unwrap(),
            Type::Product(items) => {
                write!(s, "[named ").unwrap();
                if let Some(((first_name, first), more)) = items.split_first() {
                    write!(s, "{}; {}", first_name, types.get(*first).fmt(types)).unwrap();
                    for (more_name, more) in more {
                        write!(s, ", {}; {}", more_name, types.get(*more).fmt(types)).unwrap();
                    }
                }
                write!(s, "]").unwrap();
            }
        };

        s
    }
}
