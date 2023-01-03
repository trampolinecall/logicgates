use symtern::prelude::*;

pub(crate) struct Types {
    types: symtern::Pool<Type>, // ideally, i would use a interner crate that doesnt use ids to access types but they dont handle cyclic references nicely
    named_types: Vec<(String, TypeSym)>, // this stores all the named types, one for each named type definition ast
                                // this needs to be a vec as an arena and not an interner because every named type definition ast makes a unique type
                                // these are used through the Type::Named constructor which is compared based off of its index into this array, meaning that named types will not be equal unless they point to the same item in this array
}

pub(crate) type TypeSym = symtern::Sym<usize>;
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) enum Type {
    Bit,
    Product(Vec<(String, TypeSym)>),
    Named(usize),
}

impl Types {
    pub(crate) fn new() -> Self {
        Self { types: symtern::Pool::new(), named_types: Vec::new() }
    }

    pub(crate) fn get(&self, sym: TypeSym) -> &Type {
        self.types.resolve(sym).expect("symtern resolution error")
    }

    pub(crate) fn intern(&mut self, ty: Type) -> TypeSym {
        self.types.intern(&ty).expect("symtern interning error")
    }
    pub(crate) fn new_named(&mut self, name: String, ty: TypeSym) -> TypeSym {
        self.named_types.push((name, ty));
        self.intern(Type::Named(self.named_types.len() - 1))
    }
    pub(crate) fn get_named(&self, index: usize) -> &(String, TypeSym) {
        &self.named_types[index]
    }
}
impl Type {
    pub(crate) fn size(&self, types: &Types) -> usize {
        match self {
            Type::Bit => 1,
            Type::Product(fields) => fields.iter().map(|(_, tyi)| types.get(*tyi).size(types)).sum(),
            Type::Named(named_index) => types.get(types.named_types[*named_index].1).size(types),
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
            Type::Named(index) => write!(s, "{}", types.get_named(*index).0).unwrap(),
        };

        s
    }

    pub(crate) fn has_field(&self, types: &Types, name: &str) -> bool {
        match self {
            Type::Bit => false,
            Type::Product(fields) => fields.iter().find(|(field_name, _)| field_name == name).is_some(),
            Type::Named(named_index) => types.get(types.get_named(*named_index).1).has_field(types, name),
        }
    }
}
