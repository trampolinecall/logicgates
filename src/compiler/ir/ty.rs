use symtern::prelude::*;

use super::named_type;
use crate::compiler::arena;

pub(crate) struct TypeContext<NamedType>
where
    named_type::NamedTypeId: arena::IsArenaIdFor<NamedType>,
{
    pool: symtern::Pool<Type>, // ideally, i would use a interner crate that doesnt use ids to access types but they dont handle cyclic references nicely
    pub(crate) named: arena::Arena<NamedType, named_type::NamedTypeId>,
}

pub(crate) enum NeverNamedType {} // this is kind of a not ideal way of doing this but it works
impl arena::IsArenaIdFor<NeverNamedType> for named_type::NamedTypeId {}

pub(crate) type TypeSym = symtern::Sym<usize>;
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) enum Type {
    Bit,
    Product(Vec<(String, TypeSym)>),
    Named(named_type::NamedTypeId),
}

impl<NamedType> TypeContext<NamedType>
where
    named_type::NamedTypeId: arena::IsArenaIdFor<NamedType>,
{
    pub(crate) fn new() -> Self {
        Self { pool: symtern::Pool::new(), named: arena::Arena::new() }
    }

    pub(crate) fn get(&self, sym: TypeSym) -> &Type {
        self.pool.resolve(sym).expect("symtern resolution error")
    }

    pub(crate) fn intern(&mut self, ty: Type) -> TypeSym {
        self.pool.intern(&ty).expect("symtern interning error")
    }

    pub(crate) fn transform_named<NewNamedType>(self, mut op: impl FnMut(&mut TypeContext<NeverNamedType>, NamedType) -> Option<NewNamedType>) -> Option<TypeContext<NewNamedType>>
    where
        named_type::NamedTypeId: arena::IsArenaIdFor<NewNamedType>,
    {
        let mut no_named_context = TypeContext { pool: self.pool, named: arena::Arena::new() };
        let named = self.named.transform(|named| op(&mut no_named_context, named))?;
        Some(TypeContext { pool: no_named_context.pool, named })
    }

    /*
    pub(crate) fn transform_named_infallible<NewNamedType>(self, mut op: impl FnMut(&TypeContext<NeverNamedType>, NamedType) -> NewNamedType) -> TypeContext<NewNamedType>
    where
        named_type::NamedTypeId: arena::IsArenaIdFor<NewNamedType>,
    {
        let mut no_named_context = TypeContext { pool: self.pool, named: arena::Arena::new() };
        let named = self.named.transform_infallible(|named| op(&mut no_named_context, named));
        TypeContext { pool: no_named_context.pool, named }
    }
    */
}
impl Type {
    pub(crate) fn size(&self, type_context: &TypeContext<named_type::FullyDefinedNamedType>) -> usize {
        // TODO: make a pass for this so that it can be computed only once and also so that loops and checking for infinitely sized types is easier
        match self {
            Type::Bit => 1,
            Type::Product(fields) => fields.iter().map(|(_, tyi)| type_context.get(*tyi).size(type_context)).sum(),
            Type::Named(named_index) => type_context.get(type_context.named.get(*named_index).1).size(type_context),
        }
    }

    // TODO: there is probably a better solution to this
    pub(crate) fn fmt(&self, type_context: &TypeContext<named_type::FullyDefinedNamedType>) -> String {
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
            Type::Named(index) => write!(s, "{}", type_context.named.get(*index).0).unwrap(),
        };

        s
    }

    pub(crate) fn field_type(&self, type_context: &TypeContext<named_type::FullyDefinedNamedType>, field: &str) -> Option<TypeSym> {
        match self {
            Type::Bit => None,
            Type::Product(fields) => fields.iter().find_map(|(field_name, field_type)| if field_name == field { Some(field_type) } else { None }).copied(),
            Type::Named(named_index) => type_context.get(type_context.named.get(*named_index).1).field_type(type_context, field), // TODO: unwrap expressions
        }
    }

    pub(crate) fn field_indexes(&self, type_context: &TypeContext<named_type::FullyDefinedNamedType>, field: &str) -> Option<std::ops::Range<usize>> {
        // TODO: move this to converting circuit2
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
            Type::Named(named_index) => type_context.get(type_context.named.get(*named_index).1).field_indexes(type_context, field),
        }
    }
}
