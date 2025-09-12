use air_types::{ScalarTypeMut, Type, TypeMut, Typing};
use miden_diagnostics::{SourceSpan, Spanned};

#[derive(Clone, PartialEq, Eq, Debug, Hash, Spanned)]
pub struct None {
    #[span]
    pub span: SourceSpan,
    pub ty: Option<Type>,
}

impl ScalarTypeMut for None {
    fn scalar_ty_mut(&mut self) -> &mut Option<air_types::ScalarType> {
        self.ty.scalar_ty_mut()
    }
}

impl TypeMut for None {
    fn ty_mut(&mut self) -> &mut Option<air_types::Type> {
        self.ty.ty_mut()
    }
}

impl Typing for None {
    fn ty(&self) -> Option<air_types::Type> {
        self.ty.ty()
    }
}

impl Default for None {
    fn default() -> Self {
        None { span: Default::default(), ty: None }
    }
}
