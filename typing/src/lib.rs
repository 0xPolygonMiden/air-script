mod types;

use std::fmt::Debug;

use miden_diagnostics::Span;
pub use types::*;

pub enum TypeError {
    IncompatibleScalarTypes {
        lhs: Option<ScalarType>,
        rhs: Option<ScalarType>,
    },
    IncompatibleShapes {
        lhs: Option<Type>,
        rhs: Option<Type>,
    },
    IncompatibleType {
        lhs: Option<Type>,
        rhs: Option<Type>,
    },
    TypeAlreadySet {
        lhs: Option<Type>,
        rhs: Option<Type>,
    },
    NotASubtype {
        lhs: Option<Type>,
        rhs: Option<Type>,
    },
    IncompatibleBinOp {
        bin_ty: BinType,
    },
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
        matches!(self.scalar_ty(), sty!(int))
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
    /// The shapes are compatible if:
    /// - self is `?` (None)
    /// - both are scalars
    /// - both are vectors of the same length
    /// - both are vectors with one of the lengths being `u32::MAX`
    /// - both are matrices with the same number of rows and columns
    /// - both are matrices with one or more of the rows or columns being `u32::MAX`, the other pair
    ///   (if any) being equal
    ///
    /// self\\other || _[r,c] | _[l] | _ | ?
    /// ============||========|======|===|==
    /// _[r,c]      ||   y    |  n   | n | n
    /// _[l]        ||   n    |  y   | n | n
    /// _           ||   n    |  n   | y | n
    /// ?           ||   y    |  y   | y | y
    fn is_subshape(&self, other: &impl Typing) -> bool {
        match (self.ty(), other.ty()) {
            (None, _) => true,
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
    /// self\\other || _[r,c] | _[l] | _ | ?
    /// ============||========|======|===|==
    /// _[r,c]      ||   y    |  n   | n | y
    /// _[l]        ||   n    |  y   | n | y
    /// _           ||   n    |  n   | y | y
    /// ?           ||   y    |  y   | y | y
    ///
    /// This is a more relaxed version of [Typing::is_subshape],
    /// allowing for bi-directional compatibility checks. The only
    /// difference is that it allows for `other` to be `?` (None).
    fn is_shape_compatible(&self, other: &impl Typing) -> bool {
        other.ty().is_none() || self.is_subshape(other)
    }
    /// Returns true if `self` is a subtype of `other`
    /// Notation:
    /// _ : ScalaType::Scalar(None)
    ///   Unknown scalar type
    /// felt: ScalarType::Felt
    ///   Felt type
    /// bool: ScalarType::Bool
    ///   Boolean type
    /// int: ScalarType::Int
    ///   Integer type
    ///
    /// Subtyping rules:
    /// - felt > bool > _
    /// - felt > int > _
    ///
    /// Which means:
    /// - `_` is a subtype of all scalar types
    /// - `bool` is a subtype of `felt`: a `bool` is a `felt with a `is_bool` property
    /// - `int` is a subtype of `felt`: a `int` is a `felt` with the `constant` property
    ///
    /// self\\other || felt | bool | int | _ |
    /// ============||======|======|=====|===|
    /// felt        ||   y  |    n |   n | n |
    /// bool        ||   y  |    y |   n | n |
    /// int         ||   y  |    n |   y | n |
    /// _           ||   y  |    y |   y | y |
    fn is_scalar_subtype(&self, other: &impl Typing) -> bool {
        !matches!(
            (self.scalar_ty(), other.scalar_ty()),
            (sty!(felt), sty!(bool) | sty!(int) | sty!(_))
                | (sty!(bool), sty!(int) | sty!(_))
                | (sty!(int), sty!(bool) | sty!(_))
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
    /// int: Type::Scalar(Some(ScalarType::Int))
    ///   Integer type
    /// sty[len]: Type::Vector(Some(sty), len)
    ///   Vector of length `len` with scalar type `sty`
    /// sty[rows, cols]: Type::Matrix(Some(sty), rows, cols)
    ///   Matrix with `rows` and `cols` with scalar type `sty`
    ///
    /// Subtyping rules:
    /// ? > _       > felt       > bool
    ///         ... > felt       > int
    /// ? > _[l]    > felt[l]    > bool[l]
    ///         ... > felt[l]    > int[l]
    /// ? > _[r, c] > felt[r, c] > bool[r, c]
    ///         ... > felt[r, c] > int[r, c]
    /// Assuming shapes are compatible, this function checks if the scalar types,
    /// with the added case of `?`, which all types are subtypes of.
    /// See [Typing::is_scalar_subtype] for a more detailed explanation
    /// of the subtyping rules of scalar types.
    ///
    /// self\\other || felt | bool | int | _ | ? |
    /// ============||======|======|=====|===|===|
    /// felt        ||[  y  |    n |   n | n]| n |
    /// bool        ||[  y  |    y |   n | n]| n |
    /// int         ||[  y  |    n |   y | n]| n |
    /// _           ||[  y  |    y |   y | y]| n |
    /// ?           ||   y  |    y |   y | y | y |
    ///
    /// = self.is_scalar_subtype(other) | self == ?
    /// [...] Denotes the result of the [Typing::is_scalar_subtype] method.
    fn is_subtype(&self, other: &impl Typing) -> bool {
        self.is_subshape(other) && self.is_scalar_subtype(other)
    }
    fn show_kind(&self) -> ShowOption<Kind> {
        ShowOption(self.kind())
    }
    fn show_fn_ty(&self) -> ShowOption<FunctionType> {
        match self.kind() {
            Some(Kind::Callable(fn_ty)) => ShowOption(Some(fn_ty)),
            _ => ShowOption(None),
        }
    }
    fn show_ty(&self) -> ShowOption<Type> {
        ShowOption(self.ty())
    }
    fn show_scalar_ty(&self) -> ShowOption<ScalarType> {
        ShowOption(self.scalar_ty())
    }
    /// Returns the type of the current object, if it is known or can be inferred.
    /// If the type is not known, it returns `None`.
    /// If the type can be inferred, it returns the inferred type.
    /// If the type cannot be inferred, it returns an appropriate error.
    fn infer_ty(&self) -> Result<Option<Type>, TypeError> {
        Ok(self.ty())
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
            return Err(TypeError::IncompatibleScalarTypes { lhs: ty, rhs: new_ty });
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
            return Err(TypeError::NotASubtype { lhs: ty, rhs: new_ty });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShowOption<T>(Option<T>);

impl core::fmt::Display for ShowOption<Kind> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            None => f.write_str("!"),
            Some(kind) => write!(f, "{kind}"),
        }
    }
}

impl core::fmt::Display for ShowOption<FunctionType> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            None => f.write_str("?"),
            Some(fn_ty) => write!(f, "{fn_ty}"),
        }
    }
}

impl core::fmt::Display for ShowOption<Type> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            None => f.write_str("?"),
            Some(ty) => write!(f, "{ty}"),
        }
    }
}

impl core::fmt::Display for ShowOption<ScalarType> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            None => f.write_str("_"),
            Some(sty) => write!(f, "{sty}"),
        }
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
            Kind::Callable(_) => panic!("Cannot mutate scalar type of a callable kind"),
        }
    }
}

impl TypeMut for Kind {
    fn ty_mut(&mut self) -> &mut Option<Type> {
        match self {
            Kind::Value(ty) => ty,
            Kind::Callable(_) => panic!("Cannot mutate type of a callable kind"),
        }
    }
}

impl Typing for Kind {
    fn kind(&self) -> Option<Kind> {
        Some(self.clone())
    }
    fn ty(&self) -> Option<Type> {
        let Kind::Value(ty) = self else {
            return None;
        };
        *ty
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

impl<T: Typing> Typing for Span<T> {
    fn kind(&self) -> Option<Kind> {
        self.item.kind()
    }
    fn ty(&self) -> Option<Type> {
        self.item.ty()
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
        assert_eq!(ty!(int).ty(), Some(Type::Scalar(sty!(int))));
        assert_eq!(ty!(int).scalar_ty(), sty!(int));
        assert_eq!(ty!(_[5]).ty(), Some(Type::Vector(sty!(_), 5)));
        assert_eq!(ty!(_[5]).scalar_ty(), sty!(_));
        assert_eq!(ty!(felt[5]).ty(), Some(Type::Vector(sty!(felt), 5)));
        assert_eq!(ty!(felt[5]).scalar_ty(), sty!(felt));
        assert_eq!(ty!(bool[5]).ty(), Some(Type::Vector(sty!(bool), 5)));
        assert_eq!(ty!(bool[5]).scalar_ty(), sty!(bool));
        assert_eq!(ty!(int[5]).ty(), Some(Type::Vector(sty!(int), 5)));
        assert_eq!(ty!(int[5]).scalar_ty(), sty!(int));
        assert_eq!(ty!(_[3, 4]).ty(), Some(Type::Matrix(sty!(_), 3, 4)));
        assert_eq!(ty!(_[3, 4]).scalar_ty(), sty!(_));
        assert_eq!(ty!(felt[3, 4]).ty(), Some(Type::Matrix(sty!(felt), 3, 4)));
        assert_eq!(ty!(felt[3, 4]).scalar_ty(), sty!(felt));
        assert_eq!(ty!(bool[3, 4]).ty(), Some(Type::Matrix(sty!(bool), 3, 4)));
        assert_eq!(ty!(bool[3, 4]).scalar_ty(), sty!(bool));
        assert_eq!(ty!(int[3, 4]).ty(), Some(Type::Matrix(sty!(int), 3, 4)));
        assert_eq!(ty!(int[3, 4]).scalar_ty(), sty!(int));
    }

    #[test]
    fn test_typing_subtype() {
        assert_subtype!(ty!(?); ty!(?));
        assert_subtype!(ty!(?); ty!(_));
        assert_subtype!(ty!(?); ty!(felt));
        assert_subtype!(ty!(?); ty!(bool));
        assert_subtype!(ty!(?); ty!(int));
        assert_subtype!(ty!(?); ty!(_[5]));
        assert_subtype!(ty!(?); ty!(felt[5]));
        assert_subtype!(ty!(?); ty!(bool[5]));
        assert_subtype!(ty!(?); ty!(int[5]));
        assert_subtype!(ty!(?); ty!(_[3, 4]));
        assert_subtype!(ty!(?); ty!(felt[3, 4]));
        assert_subtype!(ty!(?); ty!(bool[3, 4]));
        assert_subtype!(ty!(?); ty!(int[3, 4]));

        assert_subtype!(ty!(_); !ty!(?));
        assert_subtype!(ty!(_); ty!(_));
        assert_subtype!(ty!(_); ty!(felt));
        assert_subtype!(ty!(_); ty!(bool));
        assert_subtype!(ty!(_); ty!(int));
        assert_subtype!(ty!(_); !ty!(_[5]));
        assert_subtype!(ty!(_); !ty!(felt[5]));
        assert_subtype!(ty!(_); !ty!(bool[5]));
        assert_subtype!(ty!(_); !ty!(int[5]));
        assert_subtype!(ty!(_); !ty!(_[3, 4]));
        assert_subtype!(ty!(_); !ty!(felt[3, 4]));
        assert_subtype!(ty!(_); !ty!(bool[3, 4]));
        assert_subtype!(ty!(_); !ty!(int[3, 4]));

        assert_subtype!(ty!(felt); !ty!(?));
        assert_subtype!(ty!(felt); !ty!(_));
        assert_subtype!(ty!(felt); ty!(felt));
        assert_subtype!(ty!(felt); !ty!(bool));
        assert_subtype!(ty!(felt); !ty!(int));
        assert_subtype!(ty!(felt); !ty!(_[5]));
        assert_subtype!(ty!(felt); !ty!(felt[5]));
        assert_subtype!(ty!(felt); !ty!(bool[5]));
        assert_subtype!(ty!(felt); !ty!(int[5]));
        assert_subtype!(ty!(felt); !ty!(_[3, 4]));
        assert_subtype!(ty!(felt); !ty!(felt[3, 4]));
        assert_subtype!(ty!(felt); !ty!(bool[3, 4]));
        assert_subtype!(ty!(felt); !ty!(int[3, 4]));

        assert_subtype!(ty!(bool); !ty!(?));
        assert_subtype!(ty!(bool); !ty!(_));
        assert_subtype!(ty!(bool); ty!(felt));
        assert_subtype!(ty!(bool); ty!(bool));
        assert_subtype!(ty!(bool); !ty!(int));
        assert_subtype!(ty!(bool); !ty!(_[5]));
        assert_subtype!(ty!(bool); !ty!(felt[5]));
        assert_subtype!(ty!(bool); !ty!(bool[5]));
        assert_subtype!(ty!(bool); !ty!(int[5]));
        assert_subtype!(ty!(bool); !ty!(_[3, 4]));
        assert_subtype!(ty!(bool); !ty!(felt[3, 4]));
        assert_subtype!(ty!(bool); !ty!(bool[3, 4]));
        assert_subtype!(ty!(bool); !ty!(int[3, 4]));

        assert_subtype!(ty!(int); !ty!(?));
        assert_subtype!(ty!(int); !ty!(_));
        assert_subtype!(ty!(int); ty!(felt));
        assert_subtype!(ty!(int); !ty!(bool));
        assert_subtype!(ty!(int); ty!(int));
        assert_subtype!(ty!(int); !ty!(_[5]));
        assert_subtype!(ty!(int); !ty!(felt[5]));
        assert_subtype!(ty!(int); !ty!(bool[5]));
        assert_subtype!(ty!(int); !ty!(int[5]));
        assert_subtype!(ty!(int); !ty!(_[3, 4]));
        assert_subtype!(ty!(int); !ty!(felt[3, 4]));
        assert_subtype!(ty!(int); !ty!(bool[3, 4]));
        assert_subtype!(ty!(int); !ty!(int[3, 4]));

        assert_subtype!(ty!(_[5]); !ty!(?));
        assert_subtype!(ty!(_[5]); !ty!(_));
        assert_subtype!(ty!(_[5]); !ty!(felt));
        assert_subtype!(ty!(_[5]); !ty!(bool));
        assert_subtype!(ty!(_[5]); !ty!(int));
        assert_subtype!(ty!(_[5]); ty!(_[5]));
        assert_subtype!(ty!(_[5]); ty!(felt[5]));
        assert_subtype!(ty!(_[5]); ty!(bool[5]));
        assert_subtype!(ty!(_[5]); ty!(int[5]));
        assert_subtype!(ty!(_[5]); !ty!(_[3, 4]));
        assert_subtype!(ty!(_[5]); !ty!(felt[3, 4]));
        assert_subtype!(ty!(_[5]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(_[5]); !ty!(int[3, 4]));

        assert_subtype!(ty!(felt[5]); !ty!(?));
        assert_subtype!(ty!(felt[5]); !ty!(_));
        assert_subtype!(ty!(felt[5]); !ty!(felt));
        assert_subtype!(ty!(felt[5]); !ty!(bool));
        assert_subtype!(ty!(felt[5]); !ty!(int));
        assert_subtype!(ty!(felt[5]); !ty!(_[5]));
        assert_subtype!(ty!(felt[5]); ty!(felt[5]));
        assert_subtype!(ty!(felt[5]); !ty!(bool[5]));
        assert_subtype!(ty!(felt[5]); !ty!(int[5]));
        assert_subtype!(ty!(felt[5]); !ty!(_[3, 4]));
        assert_subtype!(ty!(felt[5]); !ty!(felt[3, 4]));
        assert_subtype!(ty!(felt[5]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(felt[5]); !ty!(int[3, 4]));

        assert_subtype!(ty!(bool[5]); !ty!(?));
        assert_subtype!(ty!(bool[5]); !ty!(_));
        assert_subtype!(ty!(bool[5]); !ty!(felt));
        assert_subtype!(ty!(bool[5]); !ty!(bool));
        assert_subtype!(ty!(bool[5]); !ty!(int));
        assert_subtype!(ty!(bool[5]); !ty!(_[5]));
        assert_subtype!(ty!(bool[5]); ty!(felt[5]));
        assert_subtype!(ty!(bool[5]); ty!(bool[5]));
        assert_subtype!(ty!(bool[5]); !ty!(int[5]));
        assert_subtype!(ty!(bool[5]); !ty!(_[3, 4]));
        assert_subtype!(ty!(bool[5]); !ty!(felt[3, 4]));
        assert_subtype!(ty!(bool[5]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(bool[5]); !ty!(int[3, 4]));

        assert_subtype!(ty!(int[5]); !ty!(?));
        assert_subtype!(ty!(int[5]); !ty!(_));
        assert_subtype!(ty!(int[5]); !ty!(felt));
        assert_subtype!(ty!(int[5]); !ty!(bool));
        assert_subtype!(ty!(int[5]); !ty!(int));
        assert_subtype!(ty!(int[5]); !ty!(_[5]));
        assert_subtype!(ty!(int[5]); ty!(felt[5]));
        assert_subtype!(ty!(int[5]); !ty!(bool[5]));
        assert_subtype!(ty!(int[5]); ty!(int[5]));
        assert_subtype!(ty!(int[5]); !ty!(_[3, 4]));
        assert_subtype!(ty!(int[5]); !ty!(felt[3, 4]));
        assert_subtype!(ty!(int[5]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(int[5]); !ty!(int[3, 4]));

        assert_subtype!(ty!(_[3, 4]); !ty!(?));
        assert_subtype!(ty!(_[3, 4]); !ty!(_));
        assert_subtype!(ty!(_[3, 4]); !ty!(felt));
        assert_subtype!(ty!(_[3, 4]); !ty!(bool));
        assert_subtype!(ty!(_[3, 4]); !ty!(int));
        assert_subtype!(ty!(_[3, 4]); !ty!(_[5]));
        assert_subtype!(ty!(_[3, 4]); !ty!(felt[5]));
        assert_subtype!(ty!(_[3, 4]); !ty!(bool[5]));
        assert_subtype!(ty!(_[3, 4]); !ty!(int[5]));
        assert_subtype!(ty!(_[3, 4]); ty!(_[3, 4]));
        assert_subtype!(ty!(_[3, 4]); ty!(felt[3, 4]));
        assert_subtype!(ty!(_[3, 4]); ty!(bool[3, 4]));
        assert_subtype!(ty!(_[3, 4]); ty!(int[3, 4]));

        assert_subtype!(ty!(felt[3, 4]); !ty!(?));
        assert_subtype!(ty!(felt[3, 4]); !ty!(_));
        assert_subtype!(ty!(felt[3, 4]); !ty!(felt));
        assert_subtype!(ty!(felt[3, 4]); !ty!(bool));
        assert_subtype!(ty!(felt[3, 4]); !ty!(int));
        assert_subtype!(ty!(felt[3, 4]); !ty!(_[5]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(felt[5]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(bool[5]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(int[5]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(_[3, 4]));
        assert_subtype!(ty!(felt[3, 4]); ty!(felt[3, 4]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(felt[3, 4]); !ty!(int[3, 4]));

        assert_subtype!(ty!(bool[3, 4]); !ty!(?));
        assert_subtype!(ty!(bool[3, 4]); !ty!(_));
        assert_subtype!(ty!(bool[3, 4]); !ty!(felt));
        assert_subtype!(ty!(bool[3, 4]); !ty!(bool));
        assert_subtype!(ty!(bool[3, 4]); !ty!(int));
        assert_subtype!(ty!(bool[3, 4]); !ty!(_[5]));
        assert_subtype!(ty!(bool[3, 4]); !ty!(felt[5]));
        assert_subtype!(ty!(bool[3, 4]); !ty!(bool[5]));
        assert_subtype!(ty!(bool[3, 4]); !ty!(int[5]));
        assert_subtype!(ty!(bool[3, 4]); !ty!(_[3, 4]));
        assert_subtype!(ty!(bool[3, 4]); ty!(felt[3, 4]));
        assert_subtype!(ty!(bool[3, 4]); ty!(bool[3, 4]));
        assert_subtype!(ty!(bool[3, 4]); !ty!(int[3, 4]));

        assert_subtype!(ty!(int[3, 4]); !ty!(?));
        assert_subtype!(ty!(int[3, 4]); !ty!(_));
        assert_subtype!(ty!(int[3, 4]); !ty!(felt));
        assert_subtype!(ty!(int[3, 4]); !ty!(bool));
        assert_subtype!(ty!(int[3, 4]); !ty!(int));
        assert_subtype!(ty!(int[3, 4]); !ty!(_[5]));
        assert_subtype!(ty!(int[3, 4]); !ty!(felt[5]));
        assert_subtype!(ty!(int[3, 4]); !ty!(bool[5]));
        assert_subtype!(ty!(int[3, 4]); !ty!(int[5]));
        assert_subtype!(ty!(int[3, 4]); !ty!(_[3, 4]));
        assert_subtype!(ty!(int[3, 4]); ty!(felt[3, 4]));
        assert_subtype!(ty!(int[3, 4]); !ty!(bool[3, 4]));
        assert_subtype!(ty!(int[3, 4]); ty!(int[3, 4]));
    }
}
