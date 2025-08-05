mod types;

use std::fmt::Debug;

use miden_diagnostics::{SourceSpan, Span};
pub use types::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeError {
    IncompatibleScalarTypes {
        lhs: Option<ScalarType>,
        rhs: Option<ScalarType>,
        span: Option<SourceSpan>,
    },
    IncompatibleShapes {
        lhs: Option<Type>,
        rhs: Option<Type>,
        span: Option<SourceSpan>,
    },
    IncompatibleType {
        lhs: Option<Type>,
        rhs: Option<Type>,
        span: Option<SourceSpan>,
    },
    TypeAlreadySet {
        lhs: Option<Type>,
        rhs: Option<Type>,
        span: Option<SourceSpan>,
    },
    NotASubtype {
        lhs: Option<Type>,
        rhs: Option<Type>,
        span: Option<SourceSpan>,
    },
    IncompatibleBinOp {
        bin_ty: BinType,
        span: Option<SourceSpan>,
    },
}

impl core::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::IncompatibleScalarTypes { lhs, rhs, .. } => {
                write!(f, "incompatible scalar types: {} and {}", Show(*lhs), Show(*rhs))?;
                Ok(())
            },
            TypeError::IncompatibleShapes { lhs, rhs, .. } => {
                write!(f, "incompatible shapes: {} and {}", Show(*lhs), Show(*rhs))?;
                Ok(())
            },
            TypeError::IncompatibleType { lhs, rhs, .. } => {
                write!(f, "incompatible types: {} and {}", Show(*lhs), Show(*rhs))?;
                Ok(())
            },
            TypeError::TypeAlreadySet { lhs, rhs, .. } => {
                write!(f, "type already set: {} vs {}", Show(*lhs), Show(*rhs))?;
                Ok(())
            },
            TypeError::NotASubtype { lhs, rhs, .. } => {
                write!(f, "type {} is not a subtype of {}", Show(*lhs), Show(*rhs))?;
                Ok(())
            },
            TypeError::IncompatibleBinOp { bin_ty, .. } => {
                write!(f, "incompatible types for binary operation: {}", bin_ty.show_fn_ty())?;
                Ok(())
            },
        }
    }
}

pub trait Typing {
    fn kind(&self) -> Option<Kind> {
        Some(Kind::Value(self.ty()))
    }
    fn ty(&self) -> Option<Type>;
    fn shape(&self) -> Option<Type> {
        self.ty().and_then(|t| match t {
            Type::Scalar(_) => ty!(_),
            Type::Vector(_, len) => ty!(_[len]),
            Type::Matrix(_, rows, cols) => ty!(_[rows, cols]),
        })
    }
    fn scalar_ty(&self) -> Option<ScalarType> {
        self.ty().scalar_ty()
    }
    fn ty_with_shape(&self, shape: impl Typing) -> Option<Type> {
        let sty = self.scalar_ty();
        let shape = shape.shape();
        if sty.is_none() {
            return shape;
        }
        match (self.scalar_ty().unwrap(), shape) {
            (sty, None | Some(Type::Scalar(_))) => Some(Type::Scalar(Some(sty))),
            (sty, Some(Type::Vector(_, len))) => Some(Type::Vector(Some(sty), len)),
            (sty, Some(Type::Matrix(_, rows, cols))) => Some(Type::Matrix(Some(sty), rows, cols)),
        }
    }
    fn is_scalar_felt(&self) -> bool {
        matches!(self.scalar_ty(), sty!(felt))
    }
    fn is_scalar_bool(&self) -> bool {
        matches!(self.scalar_ty(), sty!(bool))
    }
    fn is_scalar_int(&self) -> bool {
        matches!(self.scalar_ty(), sty!(uint))
    }
    fn is_scalar(&self) -> bool {
        matches!(self.ty(), Some(Type::Scalar(_)))
    }
    fn is_vector(&self) -> bool {
        matches!(self.ty(), Some(Type::Vector(_, _)))
    }
    fn is_matrix(&self) -> bool {
        matches!(self.ty(), Some(Type::Matrix(_, _, _)))
    }
    /// Returns true if this type is an aggregate
    #[inline]
    fn is_aggregate(&self) -> bool {
        self.is_vector() || self.is_matrix()
    }

    /// Returns true if this type is a valid iterable in a comprehension
    #[inline]
    fn is_iterable(&self) -> bool {
        self.is_vector()
    }
    /// Returns true if the shape of `self` is a sub-shape of the shape of `other`
    /// The shape of `self` is a sub-shape of the shape of `other` if:
    /// - other is `?` (None)
    /// - both are scalars
    /// - both are vectors of the same length
    /// - both are vectors with one of the lengths being `u32::MAX`
    /// - both are matrices with the same number of rows and columns
    /// - both are matrices with one or more of the rows or columns being `u32::MAX`, the other pair
    ///   (if any) being equal
    ///
    /// self\\other || ? | _ | _[l] | _[r,c]
    /// ============||===|===|======|========
    /// ?           || y | n |   n  |   n
    /// _           || y | y |   n  |   n
    /// _[l]        || y | n |   y  |   n
    /// _[r,c]      || y | n |   n  |   y
    fn is_subshape(&self, other: &impl Typing) -> bool {
        match (self.ty(), other.ty()) {
            (_, None) => true,
            (Some(Type::Scalar(_)), Some(Type::Scalar(_))) => true,
            (Some(Type::Vector(_, len1)), Some(Type::Vector(_, len2))) => {
                len1 == len2 || len1 == u32::MAX as usize || len2 == u32::MAX as usize
            },
            (Some(Type::Matrix(_, rows1, cols1)), Some(Type::Matrix(_, rows2, cols2))) => {
                (rows1 == rows2 || rows1 == u32::MAX as usize || rows2 == u32::MAX as usize)
                    && (cols1 == cols2 || cols1 == u32::MAX as usize || cols2 == u32::MAX as usize)
            },
            _ => false,
        }
    }

    /// Returns true if the shape of `self` is compatible with the shape of `other`
    /// The shapes are compatible if:
    /// - either is `?` (None)
    /// - both are scalars
    /// - both are vectors of the same length
    /// - both are vectors with one of the lengths being `u32::MAX`
    /// - both are matrices with the same number of rows and columns
    /// - both are matrices with one or more of the rows or columns being `u32::MAX`, the other pair
    ///   (if any) being equal
    ///
    /// self\\other || ? | _ | _[l] | _[r,c]
    /// ============||===|===|======|========
    /// ?           || y | y |  y   |   y
    /// _           || y | y |  n   |   n
    /// _[l]        || y | n |  y   |   n
    /// _[r,c]      || y | n |  n   |   y
    ///
    /// This is a more relaxed version of [Typing::is_subshape],
    /// allowing for bi-directional compatibility checks. The only
    /// difference is that it allows for `self` to be `?` (None).
    fn is_shape_compatible(&self, other: &impl Typing) -> bool {
        self.ty().is_none() || self.is_subshape(other)
    }
    /// Returns true if `self` is a subtype of `other`
    /// Notation:
    /// _ : ScalaType::Scalar(None)
    ///   Unknown scalar type
    /// felt: ScalarType::Felt
    ///   Felt type
    /// bool: ScalarType::Bool
    ///   Boolean type
    /// uint: ScalarType::UInt
    ///   Integer type
    ///
    /// Subtyping rules:
    /// - _ > felt > bool
    /// - _ > felt > uint
    ///
    /// Which means:
    /// - all scalar types are subtypes of `_`
    /// - `bool` is a subtype of `felt`: a `bool` is a `felt with a `is_bool` property
    /// - `uint` is a subtype of `felt`: a `uint` is a `felt` with the `constant` property
    ///
    /// self\\other || _ | felt | bool | uint |
    /// ============||===|======|======|======|
    /// _           || y |   n  |    n |    n |
    /// felt        || y |   y  |    n |    n |
    /// bool        || y |   y  |    y |    n |
    /// uint        || y |   y  |    n |    y |
    fn is_scalar_subtype(&self, other: &impl Typing) -> bool {
        !matches!(
            (self.scalar_ty(), other.scalar_ty()),
            (sty!(_), sty!(felt) | sty!(bool) | sty!(uint))
                | (sty!(felt), sty!(bool) | sty!(uint))
                | (sty!(bool), sty!(uint))
                | (sty!(uint), sty!(bool))
        )
    }
    /// Returns true if `self` is a subtype of `other`
    /// Notation:
    /// ?: None
    ///   Unknown type
    /// _: Type::Scalar(None)
    ///   Unknown scalar type
    /// felt: Type::Scalar(Some(ScalarType::Felt))
    ///   Felt type
    /// bool: Type::Scalar(Some(ScalarType::Bool))
    ///   Boolean type
    /// uint: Type::Scalar(Some(ScalarType::UInt))
    ///   Integer type
    /// sty[len]: Type::Vector(Some(sty), len)
    ///   Vector of length `len` with scalar type `sty`
    /// sty[rows, cols]: Type::Matrix(Some(sty), rows, cols)
    ///   Matrix with `rows` and `cols` with scalar type `sty`
    ///
    /// Subtyping rules:
    /// ? > _       > felt       > bool
    /// ? > _       > felt       > uint
    /// ? > _[l]    > felt[l]    > bool[l]
    /// ? > _[l]    > felt[l]    > uint[l]
    /// ? > _[r, c] > felt[r, c] > bool[r, c]
    /// ? > _[r, c] > felt[r, c] > uint[r, c]
    /// Assuming the shape of `self` is a sub-shape of the shape of `other`,
    /// this function checks if `self` is a subtype of `other`,
    /// with the added case of `?`, which all types are subtypes of.
    /// See [Typing::is_scalar_subtype] for a more detailed explanation
    /// of the subtyping rules of scalar types.
    ///
    /// self\\other || ? | _ | felt | bool | uint |
    /// ============||===|===|======|======|======|
    /// ?           || y | n |   n  |    n |    n |
    /// _           || y |[y |   n  |    n |    n]|
    /// felt        || y |[y |   y  |    n |    n]|
    /// bool        || y |[y |   y  |    y |    n]|
    /// uint        || y |[y |   y  |    n |    y]|
    ///
    /// = self.is_scalar_subtype(other) | other == ?
    /// [...] Denotes the result of the [Typing::is_scalar_subtype] method.
    fn is_subtype(&self, other: &impl Typing) -> bool {
        self.is_subshape(other) && self.is_scalar_subtype(other)
    }
    fn show_kind(&self) -> Show<Option<Kind>> {
        Show(self.kind())
    }
    fn show_fn_ty(&self) -> Show<Option<FunctionType>> {
        match self.kind() {
            Some(Kind::Callable(fn_ty)) => Show(Some(fn_ty)),
            _ => Show(None),
        }
    }
    fn show_ty(&self) -> Show<Option<Type>> {
        Show(self.ty())
    }
    fn show_scalar_ty(&self) -> Show<Option<ScalarType>> {
        Show(self.scalar_ty())
    }
    /// Returns the type of the current object, if it is known or can be inferred.
    /// If the type is not known, it returns `None`.
    /// If the type can be inferred, it returns the inferred type.
    /// If the type cannot be inferred, it returns an appropriate error.
    fn infer_ty(&self) -> Result<Option<Type>, TypeError> {
        Ok(self.ty())
    }
    fn lowest_common_supertype(&self, other: &impl Typing) -> Option<Type> {
        match (self.ty(), other.ty()) {
            (ty!(?), _) | (_, ty!(?)) => ty!(?),
            (ty!(_), Some(Type::Scalar(_))) | (Some(Type::Scalar(_)), ty!(_)) => ty!(_),
            (Some(Type::Vector(sty!(_), llen)), Some(Type::Vector(_, rlen)))
            | (Some(Type::Vector(_, llen)), Some(Type::Vector(sty!(_), rlen))) => {
                ty!(_[llen.max(rlen)])
            },
            (Some(Type::Matrix(sty!(_), lrows, lcols)), Some(Type::Matrix(_, rrows, rcols)))
            | (Some(Type::Matrix(_, lrows, lcols)), Some(Type::Matrix(sty!(_), rrows, rcols))) => {
                ty!(_[lrows.max(rrows), lcols.max(rcols)])
            },
            (lhs, rhs) if lhs.is_subtype(&rhs) => rhs,
            (lhs, rhs) if rhs.is_subtype(&lhs) => lhs,
            (ty!(uint), ty!(bool)) | (ty!(bool), ty!(uint)) => ty!(felt),
            (Some(Type::Vector(sty!(uint), llen)), Some(Type::Vector(sty!(bool), rlen)))
            | (Some(Type::Vector(sty!(bool), llen)), Some(Type::Vector(sty!(uint), rlen))) => {
                ty!(felt[core::cmp::max(llen, rlen)])
            },
            (
                Some(Type::Matrix(sty!(uint), lrows, lcols)),
                Some(Type::Matrix(sty!(bool), rrows, rcols)),
            )
            | (
                Some(Type::Matrix(sty!(bool), lrows, lcols)),
                Some(Type::Matrix(sty!(uint), rrows, rcols)),
            ) => {
                ty!(felt[core::cmp::max(lrows, rrows), core::cmp::max(lcols, rcols)])
            },
            _ => None,
        }
    }
}

pub trait ScalarTypeMut: Typing {
    fn scalar_ty_mut(&mut self) -> &mut Option<ScalarType>;
    fn update_scalar_ty(&mut self, new_ty: Option<ScalarType>) -> Result<(), TypeError> {
        let ty = self.scalar_ty();
        if ty.is_none() {
            // WARN: This should only be true before type inference
            // Any None type should raise a diagnostic after type inference
            *self.scalar_ty_mut() = new_ty;
        } else if ty.is_scalar_subtype(&new_ty) {
            // Allow widening of types
            *self.scalar_ty_mut() = new_ty;
        } else {
            return Err(TypeError::IncompatibleScalarTypes { lhs: ty, rhs: new_ty, span: None });
        }
        Ok(())
    }
}

pub trait TypeMut: Typing + ScalarTypeMut {
    fn ty_mut(&mut self) -> &mut Option<Type>;
    fn update_ty(&mut self, new_ty: Option<Type>) -> Result<(), TypeError> {
        let ty = self.ty();
        if ty.is_none() {
            // WARN: This should only be true before type inference
            // Any None type should raise a diagnostic after type inference
            *self.ty_mut() = new_ty;
        } else if ty.is_subtype(&new_ty) {
            // Allow widening of types
            *self.ty_mut() = new_ty;
        } else {
            return Err(TypeError::NotASubtype { lhs: ty, rhs: new_ty, span: None });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Show<T>(T);

impl core::fmt::Display for Show<Option<Kind>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            None => f.write_str("!"),
            Some(kind) => write!(f, "{kind}"),
        }
    }
}

impl core::fmt::Display for Show<Option<FunctionType>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            None => f.write_str("?"),
            Some(fn_ty) => write!(f, "{fn_ty}"),
        }
    }
}

impl core::fmt::Display for Show<Option<Type>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            None => f.write_str("?"),
            Some(ty) => write!(f, "{ty}"),
        }
    }
}

impl core::fmt::Display for Show<Option<ScalarType>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            None => f.write_str("_"),
            Some(sty) => write!(f, "{sty}"),
        }
    }
}

impl<T: Typing> core::fmt::Display for Show<Vec<T>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[{}]",
            self.0.iter().map(|t| t.show_ty().to_string()).collect::<Vec<_>>().join(", ")
        )
    }
}

impl<T: Typing> Typing for Show<T> {
    fn kind(&self) -> Option<Kind> {
        self.0.kind()
    }
    fn ty(&self) -> Option<Type> {
        self.0.ty()
    }
    fn scalar_ty(&self) -> Option<ScalarType> {
        self.0.scalar_ty()
    }
    fn show_kind(&self) -> Show<Option<Kind>> {
        self.0.show_kind()
    }
    fn show_fn_ty(&self) -> Show<Option<FunctionType>> {
        self.0.show_fn_ty()
    }
    fn show_ty(&self) -> Show<Option<Type>> {
        self.0.show_ty()
    }
    fn show_scalar_ty(&self) -> Show<Option<ScalarType>> {
        self.0.show_scalar_ty()
    }
}

impl Typing for ScalarType {
    fn ty(&self) -> Option<Type> {
        Some(Type::Scalar(Some(*self)))
    }
    fn scalar_ty(&self) -> Option<ScalarType> {
        Some(*self)
    }
}

impl Typing for Type {
    fn ty(&self) -> Option<Type> {
        Some(*self)
    }
    fn scalar_ty(&self) -> Option<ScalarType> {
        match self {
            Type::Scalar(st) => *st,
            Type::Vector(st, _) => *st,
            Type::Matrix(st, ..) => *st,
        }
    }
}

impl ScalarTypeMut for Type {
    fn scalar_ty_mut(&mut self) -> &mut Option<ScalarType> {
        match self {
            Type::Scalar(st) => st,
            Type::Vector(st, _) => st,
            Type::Matrix(st, ..) => st,
        }
    }
}

impl Typing for FunctionType {
    fn kind(&self) -> Option<Kind> {
        Some(Kind::Callable(self.clone()))
    }
    fn ty(&self) -> Option<Type> {
        panic!("FunctionType does not have a concrete type")
    }
    fn scalar_ty(&self) -> Option<ScalarType> {
        panic!("FunctionType does not have a concrete scalar type")
    }
}

impl ScalarTypeMut for BinType {
    fn scalar_ty_mut(&mut self) -> &mut Option<ScalarType> {
        self.result_mut().scalar_ty_mut()
    }
}

impl TypeMut for BinType {
    fn ty_mut(&mut self) -> &mut Option<Type> {
        self.result_mut()
    }
}

impl Typing for BinType {
    fn ty(&self) -> Option<Type> {
        self.infer_ty().ok()?
    }
    fn infer_ty(&self) -> Result<Option<Type>, TypeError> {
        match self {
            BinType::Eq(.., Some(ret))
            | BinType::Add(.., Some(ret))
            | BinType::Sub(.., Some(ret))
            | BinType::Mul(.., Some(ret))
            | BinType::Exp(.., Some(ret)) => Ok(Some(*ret)),
            BinType::Eq(.., None) => self.infer_bin_ty_eq(),
            BinType::Add(.., None) => self.infer_bin_ty_add(),
            BinType::Sub(.., None) => self.infer_bin_ty_sub(),
            BinType::Mul(.., None) => self.infer_bin_ty_mul(),
            BinType::Exp(.., None) => self.infer_bin_ty_exp(),
        }
    }
}

impl ScalarTypeMut for Kind {
    fn scalar_ty_mut(&mut self) -> &mut Option<ScalarType> {
        match self {
            Kind::Value(ty) => ty.scalar_ty_mut(),
            Kind::Aggregate(_) => panic!("Cannot mutate scalar type of an aggregate kind"),
            Kind::Callable(_) => panic!("Cannot mutate scalar type of a callable kind"),
        }
    }
}

impl TypeMut for Kind {
    fn ty_mut(&mut self) -> &mut Option<Type> {
        match self {
            Kind::Value(ty) => ty,
            Kind::Aggregate(_) => panic!("Cannot mutate type of an aggregate kind"),
            Kind::Callable(_) => panic!("Cannot mutate type of a callable kind"),
        }
    }
}

impl Typing for Kind {
    fn kind(&self) -> Option<Kind> {
        Some(self.clone())
    }
    fn ty(&self) -> Option<Type> {
        match self {
            Kind::Value(ty) => *ty,
            Kind::Aggregate(a) => {
                let mut inner_ty = a.first().and_then(|t| t.ty());
                for item in a.iter().skip(1) {
                    let item_ty = item.ty();
                    inner_ty = item_ty.lowest_common_supertype(&inner_ty);
                }
                match inner_ty {
                    None => None,
                    Some(Type::Scalar(st)) => ty!(st[a.len()]),
                    Some(Type::Vector(st, cols)) => ty!(st[a.len(), cols]),
                    Some(Type::Matrix(..)) => {
                        // An aggregate of matrices is not supported
                        None
                    },
                }
            },
            Kind::Callable(_) => None,
        }
    }
}

impl ScalarTypeMut for Option<ScalarType> {
    fn scalar_ty_mut(&mut self) -> &mut Option<ScalarType> {
        self
    }
}

impl ScalarTypeMut for Option<Type> {
    fn scalar_ty_mut(&mut self) -> &mut Option<ScalarType> {
        match self {
            Some(Type::Scalar(st)) => st,
            Some(Type::Vector(st, _)) => st,
            Some(Type::Matrix(st, ..)) => st,
            None => panic!("Cannot mutate scalar type of None"),
        }
    }
}

impl TypeMut for Option<Type> {
    fn ty_mut(&mut self) -> &mut Option<Type> {
        self
    }
}

impl<T: Typing> Typing for Option<T> {
    fn kind(&self) -> Option<Kind> {
        self.as_ref().and_then(|t| t.kind())
    }
    fn ty(&self) -> Option<Type> {
        self.as_ref().and_then(|t| t.ty())
    }
    fn scalar_ty(&self) -> Option<ScalarType> {
        self.as_ref().and_then(|t| t.scalar_ty())
    }
}

impl<T: Typing> Typing for Box<T> {
    fn kind(&self) -> Option<Kind> {
        T::kind(self)
    }
    fn ty(&self) -> Option<Type> {
        T::ty(self)
    }
}

impl<T: Typing> Typing for Span<T> {
    fn kind(&self) -> Option<Kind> {
        self.item.kind()
    }
    fn ty(&self) -> Option<Type> {
        self.item.ty()
    }
}

impl<T: Typing> Typing for Vec<T> {
    fn kind(&self) -> Option<Kind> {
        let agg = self.iter().map(|t| t.kind().map(Box::new)).collect();
        Some(Kind::Aggregate(agg))
    }
    fn ty(&self) -> Option<Type> {
        self.kind().ty()
    }
}

#[macro_export]
macro_rules! assert_subtype {
    ($a:expr; !$b:expr) => {
        eprintln!("assert_subtype!({}; !{})", stringify!($a), stringify!($b));
        let res = !$crate::Typing::is_subtype(&$a, &$b);
        assert!(
            res,
            "{}: !{}\nError: {} is a subtype of {}",
            $crate::Typing::show_ty(&$a),
            $crate::Typing::show_ty(&$b),
            $crate::Typing::show_ty(&$a),
            $crate::Typing::show_ty(&$b),
        );
    };
    ($a:expr; $b:expr) => {
        eprintln!("assert_subtype!({}; {})", stringify!($a), stringify!($b));
        let res = $crate::Typing::is_subtype(&$a, &$b);
        assert!(
            res,
            "{}: {}\nError: {} is a not subtype of {}",
            $crate::Typing::show_ty(&$a),
            $crate::Typing::show_ty(&$b),
            $crate::Typing::show_ty(&$a),
            $crate::Typing::show_ty(&$b),
        );
    };
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{sty, ty};

    #[test]
    fn test_typing() {
        assert_eq!(ty!(?).ty(), None);
        assert_eq!(ty!(?).scalar_ty(), sty!(_));
        assert_eq!(ty!(_).ty(), Some(Type::Scalar(sty!(_))));
        assert_eq!(ty!(_).scalar_ty(), sty!(_));
        assert_eq!(ty!(felt).ty(), Some(Type::Scalar(sty!(felt))));
        assert_eq!(ty!(felt).scalar_ty(), sty!(felt));
        assert_eq!(ty!(bool).ty(), Some(Type::Scalar(sty!(bool))));
        assert_eq!(ty!(bool).scalar_ty(), sty!(bool));
        assert_eq!(ty!(uint).ty(), Some(Type::Scalar(sty!(uint))));
        assert_eq!(ty!(uint).scalar_ty(), sty!(uint));
        assert_eq!(ty!(_[5]).ty(), Some(Type::Vector(sty!(_), 5)));
        assert_eq!(ty!(_[5]).scalar_ty(), sty!(_));
        assert_eq!(ty!(felt[5]).ty(), Some(Type::Vector(sty!(felt), 5)));
        assert_eq!(ty!(felt[5]).scalar_ty(), sty!(felt));
        assert_eq!(ty!(bool[5]).ty(), Some(Type::Vector(sty!(bool), 5)));
        assert_eq!(ty!(bool[5]).scalar_ty(), sty!(bool));
        assert_eq!(ty!(uint[5]).ty(), Some(Type::Vector(sty!(uint), 5)));
        assert_eq!(ty!(uint[5]).scalar_ty(), sty!(uint));
        assert_eq!(ty!(_[3, 4]).ty(), Some(Type::Matrix(sty!(_), 3, 4)));
        assert_eq!(ty!(_[3, 4]).scalar_ty(), sty!(_));
        assert_eq!(ty!(felt[3, 4]).ty(), Some(Type::Matrix(sty!(felt), 3, 4)));
        assert_eq!(ty!(felt[3, 4]).scalar_ty(), sty!(felt));
        assert_eq!(ty!(bool[3, 4]).ty(), Some(Type::Matrix(sty!(bool), 3, 4)));
        assert_eq!(ty!(bool[3, 4]).scalar_ty(), sty!(bool));
        assert_eq!(ty!(uint[3, 4]).ty(), Some(Type::Matrix(sty!(uint), 3, 4)));
        assert_eq!(ty!(uint[3, 4]).scalar_ty(), sty!(uint));
    }

    #[test]
    fn test_typing_subtype() {
        assert_subtype!(ty!(?); ty!(?));
        assert_subtype!(ty!(?); !ty!(_));
        assert_subtype!(ty!(?); !ty!(felt));
        assert_subtype!(ty!(?); !ty!(bool));
        assert_subtype!(ty!(?); !ty!(uint));
        assert_subtype!(ty!(?); !ty!(_[5]));
        assert_subtype!(ty!(?); !ty!(felt[5]));
        assert_subtype!(ty!(?); !ty!(bool[5]));
        assert_subtype!(ty!(?); !ty!(uint[5]));
        assert_subtype!(ty!(?); !ty!(_[3, 4]));
        assert_subtype!(ty!(?); !ty!(felt[3, 4]));
        assert_subtype!(ty!(?); !ty!(bool[3, 4]));
        assert_subtype!(ty!(?); !ty!(uint[3, 4]));

        assert_subtype!(ty!(_); ty!(?));
        assert_subtype!(ty!(_); ty!(_));
        assert_subtype!(ty!(_); !ty!(felt));
        assert_subtype!(ty!(_); !ty!(bool));
        assert_subtype!(ty!(_); !ty!(uint));
        assert_subtype!(ty!(_); !ty!(_[5]));
        assert_subtype!(ty!(_); !ty!(felt[5]));
        assert_subtype!(ty!(_); !ty!(bool[5]));
        assert_subtype!(ty!(_); !ty!(uint[5]));
        assert_subtype!(ty!(_); !ty!(_[3, 4]));
        assert_subtype!(ty!(_); !ty!(felt[3, 4]));
        assert_subtype!(ty!(_); !ty!(bool[3, 4]));
        assert_subtype!(ty!(_); !ty!(uint[3, 4]));

        assert_subtype!(ty!(felt); ty!(?));
        assert_subtype!(ty!(felt); ty!(_));
        assert_subtype!(ty!(felt); ty!(felt));
        assert_subtype!(ty!(felt); !ty!(bool));
        assert_subtype!(ty!(felt); !ty!(uint));
        assert_subtype!(ty!(felt); !ty!(_[5]));
        assert_subtype!(ty!(felt); !ty!(felt[5]));
        assert_subtype!(ty!(felt); !ty!(bool[5]));
        assert_subtype!(ty!(felt); !ty!(uint[5]));
        assert_subtype!(ty!(felt); !ty!(_[3, 4]));
        assert_subtype!(ty!(felt); !ty!(felt[3, 4]));
        assert_subtype!(ty!(felt); !ty!(bool[3, 4]));
        assert_subtype!(ty!(felt); !ty!(uint[3, 4]));

        assert_subtype!(ty!(bool); ty!(?));
        assert_subtype!(ty!(bool); ty!(_));
        assert_subtype!(ty!(bool); ty!(felt));
        assert_subtype!(ty!(bool); ty!(bool));
        assert_subtype!(ty!(bool); !ty!(uint));
        assert_subtype!(ty!(bool); !ty!(_[5]));
        assert_subtype!(ty!(bool); !ty!(felt[5]));
        assert_subtype!(ty!(bool); !ty!(bool[5]));
        assert_subtype!(ty!(bool); !ty!(uint[5]));
        assert_subtype!(ty!(bool); !ty!(_[3, 4]));
        assert_subtype!(ty!(bool); !ty!(felt[3, 4]));
        assert_subtype!(ty!(bool); !ty!(bool[3, 4]));
        assert_subtype!(ty!(bool); !ty!(uint[3, 4]));

        assert_subtype!(ty!(uint); ty!(?));
        assert_subtype!(ty!(uint); ty!(_));
        assert_subtype!(ty!(uint); ty!(felt));
        assert_subtype!(ty!(uint); !ty!(bool));
        assert_subtype!(ty!(uint); ty!(uint));
        assert_subtype!(ty!(uint); !ty!(_[5]));
        assert_subtype!(ty!(uint); !ty!(felt[5]));
        assert_subtype!(ty!(uint); !ty!(bool[5]));
        assert_subtype!(ty!(uint); !ty!(uint[5]));
        assert_subtype!(ty!(uint); !ty!(_[3, 4]));
        assert_subtype!(ty!(uint); !ty!(felt[3, 4]));
        assert_subtype!(ty!(uint); !ty!(bool[3, 4]));
        assert_subtype!(ty!(uint); !ty!(uint[3, 4]));

        assert_subtype!(ty!(_[5]); ty!(?));
        assert_subtype!(ty!(_[5]); !ty!(_));
        assert_subtype!(ty!(_[5]); !ty!(felt));
        assert_subtype!(ty!(_[5]); !ty!(bool));
        assert_subtype!(ty!(_[5]); !ty!(uint));
        assert_subtype!(ty!(_[5]); ty!(_[5]));
        assert_subtype!(ty!(_[5]); !ty!(felt[5]));
        assert_subtype!(ty!(_[5]); !ty!(bool[5]));
        assert_subtype!(ty!(_[5]); !ty!(uint[5]));
        assert_subtype!(ty!(_[5]); !ty!(_[3, 4]));
        assert_subtype!(ty!(_[5]); !ty!(felt[3, 4]));
        assert_subtype!(ty!(_[5]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(_[5]); !ty!(uint[3, 4]));

        assert_subtype!(ty!(felt[5]); ty!(?));
        assert_subtype!(ty!(felt[5]); !ty!(_));
        assert_subtype!(ty!(felt[5]); !ty!(felt));
        assert_subtype!(ty!(felt[5]); !ty!(bool));
        assert_subtype!(ty!(felt[5]); !ty!(uint));
        assert_subtype!(ty!(felt[5]); ty!(_[5]));
        assert_subtype!(ty!(felt[5]); ty!(felt[5]));
        assert_subtype!(ty!(felt[5]); !ty!(bool[5]));
        assert_subtype!(ty!(felt[5]); !ty!(uint[5]));
        assert_subtype!(ty!(felt[5]); !ty!(_[3, 4]));
        assert_subtype!(ty!(felt[5]); !ty!(felt[3, 4]));
        assert_subtype!(ty!(felt[5]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(felt[5]); !ty!(uint[3, 4]));

        assert_subtype!(ty!(bool[5]); ty!(?));
        assert_subtype!(ty!(bool[5]); !ty!(_));
        assert_subtype!(ty!(bool[5]); !ty!(felt));
        assert_subtype!(ty!(bool[5]); !ty!(bool));
        assert_subtype!(ty!(bool[5]); !ty!(uint));
        assert_subtype!(ty!(bool[5]); ty!(_[5]));
        assert_subtype!(ty!(bool[5]); ty!(felt[5]));
        assert_subtype!(ty!(bool[5]); ty!(bool[5]));
        assert_subtype!(ty!(bool[5]); !ty!(uint[5]));
        assert_subtype!(ty!(bool[5]); !ty!(_[3, 4]));
        assert_subtype!(ty!(bool[5]); !ty!(felt[3, 4]));
        assert_subtype!(ty!(bool[5]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(bool[5]); !ty!(uint[3, 4]));

        assert_subtype!(ty!(uint[5]); ty!(?));
        assert_subtype!(ty!(uint[5]); !ty!(_));
        assert_subtype!(ty!(uint[5]); !ty!(felt));
        assert_subtype!(ty!(uint[5]); !ty!(bool));
        assert_subtype!(ty!(uint[5]); !ty!(uint));
        assert_subtype!(ty!(uint[5]); ty!(_[5]));
        assert_subtype!(ty!(uint[5]); ty!(felt[5]));
        assert_subtype!(ty!(uint[5]); !ty!(bool[5]));
        assert_subtype!(ty!(uint[5]); ty!(uint[5]));
        assert_subtype!(ty!(uint[5]); !ty!(_[3, 4]));
        assert_subtype!(ty!(uint[5]); !ty!(felt[3, 4]));
        assert_subtype!(ty!(uint[5]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(uint[5]); !ty!(uint[3, 4]));

        assert_subtype!(ty!(_[3, 4]); ty!(?));
        assert_subtype!(ty!(_[3, 4]); !ty!(_));
        assert_subtype!(ty!(_[3, 4]); !ty!(felt));
        assert_subtype!(ty!(_[3, 4]); !ty!(bool));
        assert_subtype!(ty!(_[3, 4]); !ty!(uint));
        assert_subtype!(ty!(_[3, 4]); !ty!(_[5]));
        assert_subtype!(ty!(_[3, 4]); !ty!(felt[5]));
        assert_subtype!(ty!(_[3, 4]); !ty!(bool[5]));
        assert_subtype!(ty!(_[3, 4]); !ty!(uint[5]));
        assert_subtype!(ty!(_[3, 4]); ty!(_[3, 4]));
        assert_subtype!(ty!(_[3, 4]); !ty!(felt[3, 4]));
        assert_subtype!(ty!(_[3, 4]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(_[3, 4]); !ty!(uint[3, 4]));

        assert_subtype!(ty!(felt[3, 4]); ty!(?));
        assert_subtype!(ty!(felt[3, 4]); !ty!(_));
        assert_subtype!(ty!(felt[3, 4]); !ty!(felt));
        assert_subtype!(ty!(felt[3, 4]); !ty!(bool));
        assert_subtype!(ty!(felt[3, 4]); !ty!(uint));
        assert_subtype!(ty!(felt[3, 4]); !ty!(_[5]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(felt[5]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(bool[5]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(uint[5]));
        assert_subtype!(ty!(felt[3, 4]); ty!(_[3, 4]));
        assert_subtype!(ty!(felt[3, 4]); ty!(felt[3, 4]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(uint[3, 4]));

        assert_subtype!(ty!(bool[3, 4]); ty!(?));
        assert_subtype!(ty!(bool[3, 4]); !ty!(_));
        assert_subtype!(ty!(bool[3, 4]); !ty!(felt));
        assert_subtype!(ty!(bool[3, 4]); !ty!(bool));
        assert_subtype!(ty!(bool[3, 4]); !ty!(uint));
        assert_subtype!(ty!(bool[3, 4]); !ty!(_[5]));
        assert_subtype!(ty!(bool[3, 4]); !ty!(felt[5]));
        assert_subtype!(ty!(bool[3, 4]); !ty!(bool[5]));
        assert_subtype!(ty!(bool[3, 4]); !ty!(uint[5]));
        assert_subtype!(ty!(bool[3, 4]); ty!(_[3, 4]));
        assert_subtype!(ty!(bool[3, 4]); ty!(felt[3, 4]));
        assert_subtype!(ty!(bool[3, 4]); ty!(bool[3, 4]));
        assert_subtype!(ty!(bool[3, 4]); !ty!(uint[3, 4]));

        assert_subtype!(ty!(uint[3, 4]); ty!(?));
        assert_subtype!(ty!(uint[3, 4]); !ty!(_));
        assert_subtype!(ty!(uint[3, 4]); !ty!(felt));
        assert_subtype!(ty!(uint[3, 4]); !ty!(bool));
        assert_subtype!(ty!(uint[3, 4]); !ty!(uint));
        assert_subtype!(ty!(uint[3, 4]); !ty!(_[5]));
        assert_subtype!(ty!(uint[3, 4]); !ty!(felt[5]));
        assert_subtype!(ty!(uint[3, 4]); !ty!(bool[5]));
        assert_subtype!(ty!(uint[3, 4]); !ty!(uint[5]));
        assert_subtype!(ty!(uint[3, 4]); ty!(_[3, 4]));
        assert_subtype!(ty!(uint[3, 4]); ty!(felt[3, 4]));
        assert_subtype!(ty!(uint[3, 4]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(uint[3, 4]); ty!(uint[3, 4]));
    }

    macro_rules! assert_ty_eq {
        ($a:expr, $b:expr) => {{
            eprintln!("{}: {} == {}", $a.show_kind(), $a.show_ty(), $b.show_ty());
            assert_eq!(
                $a.ty(),
                $b,
                "Expected {} to be equal to {}, but it was not",
                $a.ty().show_ty(),
                $b.show_ty(),
            );
        }};
    }
    #[track_caller]
    fn assert_tys_eq_with_rev(a: Vec<impl Typing + Clone>, b: Option<Type>) {
        assert_ty_eq!(a, b);
        assert_ty_eq!(a.iter().rev().cloned().collect::<Vec<_>>(), b);
    }
    #[test]
    fn test_vec_typing() {
        assert_ty_eq!(vec![ty!(felt), ty!(felt), ty!(felt)], ty!(felt[3]));
        assert_tys_eq_with_rev(tys!([uint, felt]), ty!(felt[2]));
        assert_tys_eq_with_rev(tys!([bool, uint]), ty!(felt[2]));
        assert_tys_eq_with_rev(tys!([_, uint]), ty!(_[2]));
        assert_tys_eq_with_rev(tys!([?, uint]), ty!(?));
        assert_tys_eq_with_rev(tys!([felt[5], felt[5]]), ty!(felt[2, 5]));
        assert_tys_eq_with_rev(tys!([uint[5], felt[5]]), ty!(felt[2, 5]));
        assert_tys_eq_with_rev(tys!([bool[5], uint[5]]), ty!(felt[2, 5]));
        assert_tys_eq_with_rev(tys!([_[5], uint[5]]), ty!(_[2, 5]));
        assert_tys_eq_with_rev(tys!([bool[3], uint[8]]), ty!(felt[2, 8]));
        assert_tys_eq_with_rev(tys!([_[3], uint[8]]), ty!(_[2, 8]));
        assert_tys_eq_with_rev(tys!([?, uint[5]]), ty!(?));
        assert_tys_eq_with_rev(tys!([uint[5], felt]), ty!(?));
        assert_tys_eq_with_rev(tys!([bool[5, 2], felt]), ty!(?));
        assert_tys_eq_with_rev(tys!([uint[5], felt]), ty!(?));
        assert_tys_eq_with_rev(tys!([bool[5, 2], felt]), ty!(?));
        assert_tys_eq_with_rev(tys!([uint[5, 2], _]), ty!(?));
        assert_tys_eq_with_rev(tys!([uint[5, 2]]), ty!(?));
        assert_tys_eq_with_rev(tys!([uint, felt]), ty!(felt[2]));
        assert_tys_eq_with_rev(tys!([bool, uint]), ty!(felt[2]));
        assert_tys_eq_with_rev(tys!([_, uint]), ty!(_[2]));
        assert_tys_eq_with_rev(tys!([?, uint]), ty!(?));

        assert_tys_eq_with_rev(
            vec![tys!([uint, felt]), tys!([uint, felt]), tys!([uint, felt])],
            ty!(felt[3, 2]),
        );
        assert_tys_eq_with_rev(
            vec![tys!([bool, uint]), tys!([bool, uint]), tys!([bool, uint])],
            ty!(felt[3, 2]),
        );
        assert_tys_eq_with_rev(
            vec![tys!([_, uint]), tys!([_, uint]), tys!([_, uint])],
            ty!(_[3, 2]),
        );
        assert_tys_eq_with_rev(vec![tys!([?, uint]), tys!([?, uint]), tys!([?, uint])], ty!(?));
        assert_tys_eq_with_rev(tys!([felt[5], uint[5], bool[5]]), ty!(felt[3, 5]));
    }
}
