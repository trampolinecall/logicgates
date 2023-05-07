use crate::compiler::data::{ir::GateIdx, nominal_type, ty};

#[derive(Clone, Debug)]
pub(crate) enum Bundle {
    CurCircuitInput(ty::TypeSym),
    CurCircuitOutput(ty::TypeSym),
    GateOutput(ty::TypeSym, GateIdx),
    GateInput(ty::TypeSym, GateIdx),
    Get(Box<Bundle>, String),
    Product(Vec<(String, Bundle)>),
}

impl Bundle {
    pub(crate) fn type_(&self, type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>) -> ty::TypeSym {
        match self {
            Bundle::CurCircuitInput(ty) | Bundle::GateOutput(ty, _) | Bundle::CurCircuitOutput(ty) | Bundle::GateInput(ty, _) => *ty,
            Bundle::Get(bundle, field) => {
                let bundle_type = bundle.type_(type_context);
                ty::Type::get_field_type(&type_context.get(bundle_type).fields(type_context), field).unwrap()
            }
            Bundle::Product(tys) => {
                let ty = ty::Type::Product(tys.iter().map(|(name, subbundle)| (name.to_string(), subbundle.type_(type_context))).collect());
                type_context.intern(ty)
            }
        }
    }
}
