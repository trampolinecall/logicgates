use symtern::prelude::*;

use crate::utils::arena;

use super::named_type;

pub(crate) struct TypeContext<Struct>
where
    named_type::StructId: arena::IsArenaIdFor<Struct>,
{
    pool: symtern::Pool<Type>, // ideally, i would use a interner crate that doesnt use ids to access types but they dont handle cyclic references nicely

    // this stores all the named types, one for each named type definition ast
    // this needs to be an arena and not an interner because every named type definition ast makes a unique type
    // these are used through the Type::Named constructor which is compared based off of its index into this array, meaning that named types will not be equal unless they point to the same item in this array
    pub(crate) structs: arena::Arena<Struct, named_type::StructId>,
}

pub(crate) enum NeverNamedType {} // this is kind of a not ideal way of doing this but it works
impl arena::IsArenaIdFor<NeverNamedType> for named_type::StructId {}

pub(crate) type TypeSym = symtern::Sym<usize>;
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) enum Type {
    Bit,
    Product(Vec<(String, TypeSym)>),
    Struct(named_type::StructId),
}

impl<NamedType> TypeContext<NamedType>
where
    named_type::StructId: arena::IsArenaIdFor<NamedType>,
{
    pub(crate) fn new() -> Self {
        Self { pool: symtern::Pool::new(), structs: arena::Arena::new() }
    }

    pub(crate) fn get(&self, sym: TypeSym) -> &Type {
        self.pool.resolve(sym).expect("symtern resolution error")
    }

    pub(crate) fn intern(&mut self, ty: Type) -> TypeSym {
        self.pool.intern(&ty).expect("symtern interning error")
    }

    pub(crate) fn transform_structs<NewNamedType>(self, mut op: impl FnMut(&mut TypeContext<NeverNamedType>, NamedType) -> Option<NewNamedType>) -> Option<TypeContext<NewNamedType>>
    where
        named_type::StructId: arena::IsArenaIdFor<NewNamedType>,
    {
        let mut no_struct_context = TypeContext { pool: self.pool, structs: arena::Arena::new() };
        let structs = self.structs.transform(|struct_| op(&mut no_struct_context, struct_))?;
        Some(TypeContext { pool: no_struct_context.pool, structs })
    }

    /* (unused)
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
    pub(crate) fn size(&self, type_context: &TypeContext<named_type::FullyDefinedStruct>) -> usize {
        // TODO: make a pass for this so that it can be computed only once and also so that loops and checking for infinitely sized types is easier
        match self {
            Type::Bit => 1,
            Type::Product(fields) => fields.iter().map(|(_, t)| type_context.get(*t).size(type_context)).sum(),
            Type::Struct(struct_id) => (type_context.structs.get(*struct_id)).fields.iter().map(|(_, t)| type_context.get(*t).size(type_context)).sum(),
        }
    }

    // TODO: there is probably a better solution to this
    pub(crate) fn fmt(&self, type_context: &TypeContext<named_type::FullyDefinedStruct>) -> String {
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
            Type::Struct(index) => write!(s, "{}", type_context.structs.get(*index).name.1).unwrap(),
        };

        s
    }

    pub(crate) fn field_type(&self, type_context: &TypeContext<named_type::FullyDefinedStruct>, field: &str) -> Option<TypeSym> {
        match self {
            Type::Bit => None,
            Type::Product(fields) => fields.iter().find_map(|(field_name, field_type)| if field_name == field { Some(field_type) } else { None }).copied(),
            Type::Struct(struct_id) => type_context.structs.get(*struct_id).fields.iter().find_map(|((_, field_name), field_type)| if *field_name == field { Some(field_type) } else { None }).copied(),
        }
    }

    pub(crate) fn field_indexes(&self, type_context: &TypeContext<named_type::FullyDefinedStruct>, field: &str) -> Option<std::ops::Range<usize>> {
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
            Type::Struct(struct_id) => {
                let fields = &type_context.structs.get(*struct_id).fields;
                let mut cur_index = 0;
                for ((_, field_name), field_type) in fields {
                    let cur_type_size = type_context.get(*field_type).size(type_context);
                    if *field_name == field {
                        return Some(cur_index..cur_index + cur_type_size);
                    }
                    cur_index += cur_type_size;
                }

                None
            }
        }
    }
}
