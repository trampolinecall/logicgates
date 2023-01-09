use symtern::prelude::*;

use crate::{compiler::data::nominal_type, utils::arena};

pub(crate) struct TypeContext<Struct>
where
    nominal_type::StructId: arena::IsArenaIdFor<Struct>,
{
    pool: symtern::Pool<Type>, // ideally, i would use a interner crate that doesnt use ids to access types but they dont handle cyclic references nicely

    // this stores all the nominal types, one for each nominal type definition ast
    // this needs to be an arena and not an interner because every nominal type definition ast makes a unique type
    // these are used through the Type::Nominal constructor which is compared based off of its index into this array, meaning that nominal types will not be equal unless they point to the same item in this array
    pub(crate) structs: arena::Arena<Struct, nominal_type::StructId>,
}

pub(crate) enum NeverNominalType {} // this is kind of a not ideal way of doing this but it works
impl arena::IsArenaIdFor<NeverNominalType> for nominal_type::StructId {}

pub(crate) type TypeSym = symtern::Sym<usize>;
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) enum Type {
    Bit,
    Product(Vec<(String, TypeSym)>),
    Nominal(nominal_type::StructId),
}

impl<NominalType> TypeContext<NominalType>
where
    nominal_type::StructId: arena::IsArenaIdFor<NominalType>,
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

    pub(crate) fn transform_nominals<NewNominalType>(self, mut op: impl FnMut(&mut TypeContext<NeverNominalType>, NominalType) -> Option<NewNominalType>) -> Option<TypeContext<NewNominalType>>
    where
        nominal_type::StructId: arena::IsArenaIdFor<NewNominalType>,
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
    pub(crate) fn size(&self, type_context: &TypeContext<nominal_type::FullyDefinedStruct>) -> usize {
        // TODO: make a pass for this so that it can be computed only once and also so that loops and checking for infinitely sized types is easier
        match self {
            Type::Bit => 1,
            Type::Product(fields) => fields.iter().map(|(_, t)| type_context.get(*t).size(type_context)).sum(),
            Type::Nominal(struct_id) => (type_context.structs.get(*struct_id)).fields.iter().map(|(_, t)| type_context.get(*t).size(type_context)).sum(),
        }
    }

    // TODO: there is probably a better solution to this
    pub(crate) fn fmt(&self, type_context: &TypeContext<nominal_type::FullyDefinedStruct>) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        match self {
            Type::Bit => write!(s, "bit").unwrap(),
            Type::Product(items) => {
                if items.is_empty() {
                    write!(s, "[]").unwrap();
                } else {
                    let field_numbers_ascending = items.iter().map(|(name, _)| name.parse::<usize>()).zip(0..).all(|(actual, number)| actual == Ok(number));
                    let first_ty = items[0].1;
                    let all_same = items.iter().map(|(_, ty)| ty).all(|t| *t == first_ty);

                    write!(s, "[").unwrap();
                    if all_same && field_numbers_ascending {
                        write!(s, "{}]{}", items.len(), type_context.get(first_ty).fmt(type_context)).unwrap();
                    } else if field_numbers_ascending {
                        if let Some(((_, first), more)) = items.split_first() {
                            write!(s, "{}", type_context.get(*first).fmt(type_context)).unwrap();
                            for (_, more) in more {
                                write!(s, ", {}", type_context.get(*more).fmt(type_context)).unwrap();
                            }
                        }
                        write!(s, "]").unwrap();
                    } else {
                        write!(s, "; ").unwrap();
                        if let Some(((first_name, first), more)) = items.split_first() {
                            write!(s, "{}; {}", first_name, type_context.get(*first).fmt(type_context)).unwrap();
                            for (more_name, more) in more {
                                write!(s, ", {}; {}", more_name, type_context.get(*more).fmt(type_context)).unwrap();
                            }
                        }
                        write!(s, "]").unwrap();
                    }
                }
            }
            Type::Nominal(index) => write!(s, "{}", type_context.structs.get(*index).name.1).unwrap(),
        };

        s
    }

    pub(crate) fn fields<'s: 'r, 'c: 'r, 'r>(&'s self, type_context: &'c TypeContext<nominal_type::FullyDefinedStruct>) -> Vec<(&'r str, TypeSym)> {
        // TODO: figure out a better way than to return a vec (return slice or iterator?)
        match self {
            Type::Bit => Vec::new(),
            Type::Product(fields) => fields.iter().map(|(name, ty)| (name.as_str(), *ty)).collect(),
            Type::Nominal(struct_id) => type_context.structs.get(*struct_id).fields.iter().map(|((_, name), ty)| (*name, *ty)).collect(),
        }
    }

    pub(crate) fn get_field_type(fields: &[(&str, TypeSym)], field: &str) -> Option<TypeSym> {
        fields.iter().find_map(|(field_name, field_type)| if *field_name == field { Some(field_type) } else { None }).copied()
    }
}
