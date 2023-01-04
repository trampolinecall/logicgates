use symtern::prelude::*;

pub(crate) struct TypeContext {
    pool: symtern::Pool<Type>, // ideally, i would use a interner crate that doesnt use ids to access types but they dont handle cyclic references nicely
    named: Vec<(String, TypeSym)>, // this stores all the named types, one for each named type definition ast
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

impl TypeContext {
    pub(crate) fn new() -> Self {
        Self { pool: symtern::Pool::new(), named: Vec::new() }
    }

    pub(crate) fn get(&self, sym: TypeSym) -> &Type {
        self.pool.resolve(sym).expect("symtern resolution error")
    }

    pub(crate) fn intern(&mut self, ty: Type) -> TypeSym {
        self.pool.intern(&ty).expect("symtern interning error")
    }
    pub(crate) fn new_named(&mut self, name: String, ty: TypeSym) -> TypeSym {
        self.named.push((name, ty));
        self.intern(Type::Named(self.named.len() - 1))
    }
    pub(crate) fn get_named(&self, index: usize) -> &(String, TypeSym) {
        &self.named[index]
    }
}
impl Type {
    pub(crate) fn size(&self, type_context: &TypeContext) -> usize {
        match self {
            Type::Bit => 1,
            Type::Product(fields) => fields.iter().map(|(_, tyi)| type_context.get(*tyi).size(type_context)).sum(),
            Type::Named(named_index) => type_context.get(type_context.named[*named_index].1).size(type_context),
        }
    }

    // TODO: there is probably a better solution to this
    pub(crate) fn fmt(&self, type_context: &TypeContext) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        match self {
            Type::Bit => write!(s, "'").unwrap(),
            Type::Product(items) => {
                write!(s, "[named ").unwrap();
                if let Some(((first_name, first), more)) = items.split_first() {
                    write!(s, "{}; {}", first_name, type_context.get(*first).fmt(type_context)).unwrap();
                    for (more_name, more) in more {
                        write!(s, ", {}; {}", more_name, type_context.get(*more).fmt(type_context)).unwrap();
                    }
                }
                write!(s, "]").unwrap();
            }
            Type::Named(index) => write!(s, "{}", type_context.get_named(*index).0).unwrap(),
        };

        s
    }

    pub(crate) fn field_type(&self, type_context: &TypeContext, field: &str) -> Option<TypeSym> {
        match self {
            Type::Bit => None,
            Type::Product(fields) => fields.iter().find_map(|(field_name, field_type)| if field_name == field { Some(field_type) } else { None }).copied(),
            Type::Named(named_index) => type_context.get(type_context.get_named(*named_index).1).field_type(type_context, field), // TODO: unwrap expressions
        }
    }

    pub(crate) fn field_indexes(&self, type_context: &TypeContext, field: &str) -> Option<std::ops::Range<usize>> {
        match self {
            Type::Bit => None,
            Type::Product(fields) => {
                let mut cur_index = 0;
                for (field_name, field_type) in fields {
                    let cur_type_size = type_context.get(*field_type).size(type_context);
                    if field_name == field {
                        return Some(cur_index..cur_index + cur_type_size);
                    }
                    cur_index += cur_type_size;
                }

                None
            }
            Type::Named(named_index) => type_context.get(type_context.get_named(*named_index).1).field_indexes(type_context, field),
        }
    }
}
