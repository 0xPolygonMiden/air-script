use air_types::{ScalarTypeMut, Type, TypeMut, Typing};
use miden_diagnostics::{SourceSpan, Spanned};

#[derive(Clone, PartialEq, Eq, Debug, Hash, Spanned)]
pub struct Stale {
    #[span]
    pub span: SourceSpan,
    pub ty: Option<Type>,
}

impl ScalarTypeMut for Stale {
    fn update_scalar_ty_unchecked(&mut self, new_ty: Option<air_types::ScalarType>) {
        self.ty.update_scalar_ty_unchecked(new_ty);
    }
}

impl TypeMut for Stale {
    fn update_ty_unchecked(&mut self, new_ty: Option<Type>) {
        self.ty = new_ty;
    }
}

impl Typing for Stale {
    fn ty(&self) -> Option<air_types::Type> {
        self.ty.ty()
    }
}

impl Default for Stale {
    fn default() -> Self {
        Stale { span: Default::default(), ty: None }
    }
}
