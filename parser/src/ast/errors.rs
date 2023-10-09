use miden_diagnostics::{Diagnostic, Label, SourceSpan, ToDiagnostic};

/// Represents an invalid expression for use in an `Expr` context
#[derive(Debug, thiserror::Error)]
pub enum InvalidExprError {
    #[error("this value is too large for an exponent")]
    InvalidExponent(SourceSpan),
    #[error("expected exponent to be a constant")]
    NonConstantExponent(SourceSpan),
    #[error("accessing column boundaries is not allowed here")]
    BoundedSymbolAccess(SourceSpan),
    #[error("expected scalar expression")]
    InvalidScalarExpr(SourceSpan),
}
impl Eq for InvalidExprError {}
impl PartialEq for InvalidExprError {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}
impl ToDiagnostic for InvalidExprError {
    fn to_diagnostic(self) -> Diagnostic {
        let message = format!("{}", &self);
        match self {
            Self::InvalidExponent(span) => Diagnostic::error()
                .with_message("invalid expression")
                .with_labels(vec![
                    Label::primary(span.source_id(), span).with_message(message)
                ]),
            Self::NonConstantExponent(span) => Diagnostic::error()
                .with_message("invalid expression")
                .with_labels(vec![
                    Label::primary(span.source_id(), span).with_message(message)
                ])
                .with_notes(vec![
                    "Only constant powers are supported with the exponentiation operator currently"
                        .to_string(),
                ]),
            Self::BoundedSymbolAccess(span) => Diagnostic::error()
                .with_message("invalid expression")
                .with_labels(vec![
                    Label::primary(span.source_id(), span).with_message(message)
                ]),
            Self::InvalidScalarExpr(span) => Diagnostic::error()
                .with_message("invalid expression")
                .with_labels(vec![
                    Label::primary(span.source_id(), span).with_message(message)
                ]),
        }
    }
}

/// Represents an invalid type for use in a `BindingType` context
#[derive(Debug, thiserror::Error)]
pub enum InvalidTypeError {
    #[error("expected iterable to be a vector")]
    NonVectorIterable(SourceSpan),
}
impl Eq for InvalidTypeError {}
impl PartialEq for InvalidTypeError {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}
impl ToDiagnostic for InvalidTypeError {
    fn to_diagnostic(self) -> Diagnostic {
        let message = format!("{}", &self);
        match self {
            Self::NonVectorIterable(span) => Diagnostic::error()
                .with_message("invalid type")
                .with_labels(vec![
                    Label::primary(span.source_id(), span).with_message(message)
                ])
                .with_notes(vec!["Only vectors can be used as iterables".to_string()]),
        }
    }
}
