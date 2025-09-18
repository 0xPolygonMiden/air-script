use crate::{TypeError, Typing};

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScalarType {
    Felt,
    Bool,
    UInt,
}

impl core::fmt::Display for ScalarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Felt => f.write_str("felt"),
            Self::Bool => f.write_str("bool"),
            Self::UInt => f.write_str("uint"),
        }
    }
}

#[macro_export]
macro_rules! sty {
    // for pattern matching
    // equivalent to a `_` in a match or let expression
    (any) => {
        _
    };
    // for pattern matching
    // equivalent to a `$name` in a match or let expression
    (any: $name:ident) => {
        $name
    };
    (_) => {
        None
    };
    (felt) => {
        Some($crate::ScalarType::Felt)
    };
    (bool) => {
        Some($crate::ScalarType::Bool)
    };
    (uint) => {
        Some($crate::ScalarType::UInt)
    };
    ($sty:ident) => {
        $sty
    };
}

/// The types of values which can be represented in an AirScript program
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    // annotation: sty
    // where sty is the scalar type
    Scalar(Option<ScalarType>),
    // annotation: `sty[len]`
    // where len is the number of elements in the vector,
    // and sty is the scalar type
    Vector(Option<ScalarType>, usize),
    // annotation: `sty[rows, cols]`
    // where rows and cols are the dimensions of the matrix,
    // and sty is the scalar type
    Matrix(Option<ScalarType>, usize, usize),
}

impl core::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scalar(None) => f.write_str("_"),
            Self::Vector(None, len) => write!(f, "_[{len}]"),
            Self::Matrix(None, rows, cols) => write!(f, "_[{rows}, {cols}]"),
            Self::Scalar(Some(sty)) => f.write_str(&sty.to_string()),
            Self::Vector(Some(sty), len) => write!(f, "{sty}[{len}]"),
            Self::Matrix(Some(sty), rows, cols) => write!(f, "{sty}[{rows}, {cols}]"),
        }
    }
}

#[macro_export]
macro_rules! ty {
    // for pattern matching
    // equivalent to a `_` in a match or let expression
    (any) => {
        _
    };
    // for pattern matching
    // equivalent to a `$name` in a match or let expression
    (any: $name:ident) => {
        $name
    };
    (?) => {
        None::<$crate::Type>
    };
    (_) => {
        Some($crate::Type::Scalar(None))
    };
    ($sty:ident) => {
        Some($crate::Type::Scalar($crate::sty!($sty)))
    };
    (_[$len:expr]) => {
        Some($crate::Type::Vector($crate::sty!(_), $len))
    };
    ($sty:ident[$len:expr]) => {
        Some($crate::Type::Vector($crate::sty!($sty), $len))
    };
    (_[$rows:expr, $cols:expr]) => {
        Some($crate::Type::Matrix($crate::sty!(_), $rows, $cols))
    };
    ($sty:ident[$rows:expr, $cols:expr]) => {
        Some($crate::Type::Matrix($crate::sty!($sty), $rows, $cols))
    };
}

pub struct Push<T>(pub Vec<T>);
impl<T> Push<T> {
    pub fn push(mut self, ty: T) -> Self {
        self.0.push(ty);
        self
    }
}

#[macro_export]
macro_rules! tys {
    ([$($args:tt)+]) => {
        tys!(RES: Push(vec![]); $($args)+).0
    };
    (RES: $res:expr; ) => {
        $res
    };
    (RES: $res:expr; ? $(, $($rest:tt)+)?) => {
        tys!(RES: $crate::Push::push($res, $crate::ty!(?)); $($($rest)+)?)
    };
    (RES: $res:expr; _$([$($spec:tt)+])? $(, $($rest:tt)+)?) => {
        tys!(RES: $crate::Push::push($res, $crate::ty!(_$([$($spec)+])?)); $($($rest)+)?)
    };
    (RES: $res:expr; $name:ident$([$($spec:tt)+])? $(, $($rest:tt)+)?) => {
        tys!(RES: $crate::Push::push($res, $crate::ty!($name$([$($spec)+])?)); $($($rest)+)?)
    };
}

#[macro_export]
macro_rules! kinds {
    ([$($args:tt)+]) => {
        kinds!(RES: Push(vec![]); $($args)+).0
    };
    (RES: $res:expr; ) => {
        $res
    };
    (RES: $res:expr; ?) => {
        kinds!(RES: $crate::Push::push($res, Option::Some(Box::new($crate::kind!(?))));)
    };
    (RES: $res:expr; _$([$($spec:tt)+])? $(, $($rest:tt)+)?) => {
        kinds!(RES: $crate::Push::push($res, Option::Some(Box::new($crate::kind!(_$([$($spec)+])?)))); $($($rest)+)?)
    };
    (RES: $res:expr; $name:ident$([$($spec:tt)+])? $(, $($rest:tt)+)?) => {
        kinds!(RES: $crate::Push::push($res, Option::Some(Box::new($crate::kind!($name$([$($spec)+])?)))); $($($rest)+)?)
    };
}

#[macro_export]
macro_rules! tty {
    ([$($n1:ident$([$l1:expr])?),*]) => {
        Vec::<Option<$crate::Type>>::from([
            $($crate::tty!($n1$([$l1])?)),*
        ])
    };
    ($name:ident[$len:expr]) => {
        match $len {
            1 => $crate::ty!(felt),
            _ => $crate::ty!(felt[$len]),
        }
    };
    ($name:ident) => {
        $crate::ty!(felt)
    };
}

/// Represents the type signature of a function
#[derive(Hash, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FunctionType {
    /// An evaluator function, which has no results, and has
    /// a complex type signature due to the nature of trace bindings
    Evaluator(Vec<Option<Type>>),
    /// A standard function with one or more inputs, and a result
    Function(Vec<Option<Type>>, Option<Type>),
}

impl Default for FunctionType {
    fn default() -> Self {
        Self::Evaluator(vec![])
    }
}

impl FunctionType {
    pub fn params(&self) -> &[Option<Type>] {
        match self {
            Self::Evaluator(params) => params,
            Self::Function(params, _) => params,
        }
    }

    pub fn result(&self) -> Option<Type> {
        match self {
            Self::Evaluator(_) => None,
            Self::Function(_, ret) => *ret,
        }
    }

    pub fn check_args_kinds(&self, args: &[&Kind]) -> bool {
        eprintln!("Checking function type {self} against params {args:?}");
        let params = self.params();
        if params.len() != args.len() {
            return false;
        }
        for (arg_ty, param_kind) in args.iter().zip(params.iter()) {
            eprintln!("  Checking arg_ty {arg_ty:?} against param_kind {param_kind:?}");
            if !arg_ty.is_subtype(param_kind) {
                eprintln!("  Failed!: {arg_ty:?} is not a subtype of {param_kind:?}");
                return false;
            }
        }
        eprintln!("  Success!");
        true
    }
}

impl core::fmt::Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Evaluator(args) => {
                f.write_str("ev(")?;
                write!(
                    f,
                    "[{}]",
                    args.iter().map(|ty| ty.show_ty().to_string()).collect::<Vec<_>>().join(", ")
                )?;
                f.write_str(")")
            },
            Self::Function(args, ret) => {
                f.write_str("fn(")?;
                f.write_str(
                    &args.iter().map(|ty| ty.show_ty().to_string()).collect::<Vec<_>>().join(", "),
                )?;
                f.write_str(") -> ")?;
                if let Some(ret_type) = ret {
                    write!(f, "{ret_type}")
                } else {
                    f.write_str("?")
                }
            },
        }
    }
}

#[macro_export]
macro_rules! fty {
    (ev ([])) => {
        $crate::FunctionType::Evaluator(vec![])
    };
    (ev ([$($tty:tt)+])) => {
        $crate::FunctionType::Evaluator($crate::tty!([$($tty)+]))
    };
    (fn ($($arg:tt)*) -> $($ret:tt)+) => {
        $crate::FunctionType::Function(tys!([$($arg)*]), $crate::ty!($($ret)+))
    };
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinType {
    Eq(Option<Type>, Option<Type>, Option<Type>),
    Add(Option<Type>, Option<Type>, Option<Type>),
    Sub(Option<Type>, Option<Type>, Option<Type>),
    Mul(Option<Type>, Option<Type>, Option<Type>),
    Exp(Option<Type>, Option<Type>, Option<Type>),
}

impl Default for BinType {
    fn default() -> Self {
        Self::Eq(None, None, None)
    }
}

impl BinType {
    pub fn lhs(&self) -> Option<Type> {
        match self {
            Self::Eq(lhs, ..)
            | Self::Add(lhs, ..)
            | Self::Sub(lhs, ..)
            | Self::Mul(lhs, ..)
            | Self::Exp(lhs, ..) => *lhs,
        }
    }

    pub fn lhs_mut(&mut self) -> &mut Option<Type> {
        match self {
            Self::Eq(lhs, ..)
            | Self::Add(lhs, ..)
            | Self::Sub(lhs, ..)
            | Self::Mul(lhs, ..)
            | Self::Exp(lhs, ..) => lhs,
        }
    }

    pub fn rhs(&self) -> Option<Type> {
        match self {
            Self::Eq(_, rhs, _)
            | Self::Add(_, rhs, _)
            | Self::Sub(_, rhs, _)
            | Self::Mul(_, rhs, _)
            | Self::Exp(_, rhs, _) => *rhs,
        }
    }

    pub fn rhs_mut(&mut self) -> &mut Option<Type> {
        match self {
            Self::Eq(_, rhs, _)
            | Self::Add(_, rhs, _)
            | Self::Sub(_, rhs, _)
            | Self::Mul(_, rhs, _)
            | Self::Exp(_, rhs, _) => rhs,
        }
    }

    pub fn result(&self) -> Option<Type> {
        match self {
            Self::Eq(_, _, ret)
            | Self::Add(_, _, ret)
            | Self::Sub(_, _, ret)
            | Self::Mul(_, _, ret)
            | Self::Exp(_, _, ret) => *ret,
        }
    }

    pub fn result_mut(&mut self) -> &mut Option<Type> {
        match self {
            Self::Eq(_, _, ret)
            | Self::Add(_, _, ret)
            | Self::Sub(_, _, ret)
            | Self::Mul(_, _, ret)
            | Self::Exp(_, _, ret) => ret,
        }
    }

    pub fn as_fn(&self) -> FunctionType {
        match self {
            Self::Eq(lhs, rhs, ret)
            | Self::Add(lhs, rhs, ret)
            | Self::Sub(lhs, rhs, ret)
            | Self::Mul(lhs, rhs, ret)
            | Self::Exp(lhs, rhs, ret) => FunctionType::Function(vec![*lhs, *rhs], *ret),
        }
    }
    /// Returns a new [BinType] with all types casted to their [Type::Scalar] equivalent:
    /// - `?`               -> `_`
    /// - `sty`             -> `sty`
    /// - `sty[len]`        -> `sty`
    /// - `sty[rows, cols]` -> `sty`
    ///
    /// This corresponds to the shape `_`.
    pub fn without_shape(&self) -> Self {
        match self {
            Self::Eq(lhs, rhs, ret) => Self::Eq(
                (*lhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*rhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*ret).and_then(|ty| ty.ty_with_shape(ty!(_))),
            ),
            Self::Add(lhs, rhs, ret) => Self::Add(
                (*lhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*rhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*ret).and_then(|ty| ty.ty_with_shape(ty!(_))),
            ),
            Self::Sub(lhs, rhs, ret) => Self::Sub(
                (*lhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*rhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*ret).and_then(|ty| ty.ty_with_shape(ty!(_))),
            ),
            Self::Mul(lhs, rhs, ret) => Self::Mul(
                (*lhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*rhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*ret).and_then(|ty| ty.ty_with_shape(ty!(_))),
            ),
            Self::Exp(lhs, rhs, ret) => Self::Exp(
                (*lhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*rhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*ret).and_then(|ty| ty.ty_with_shape(ty!(_))),
            ),
        }
    }
}

impl core::fmt::Display for BinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq(lhs, rhs, None) => write!(f, "{} = {}", lhs.show_ty(), rhs.show_ty()),
            Self::Add(lhs, rhs, None) => write!(f, "{} + {}", lhs.show_ty(), rhs.show_ty()),
            Self::Sub(lhs, rhs, None) => write!(f, "{} - {}", lhs.show_ty(), rhs.show_ty()),
            Self::Mul(lhs, rhs, None) => write!(f, "{} * {}", lhs.show_ty(), rhs.show_ty()),
            Self::Exp(lhs, rhs, None) => write!(f, "{} ^ {}", lhs.show_ty(), rhs.show_ty()),
            Self::Eq(lhs, rhs, ret) => {
                write!(f, "{} = {} -> {}", lhs.show_ty(), rhs.show_ty(), ret.show_ty())
            },
            Self::Add(lhs, rhs, ret) => {
                write!(f, "{} + {} -> {}", lhs.show_ty(), rhs.show_ty(), ret.show_ty())
            },
            Self::Sub(lhs, rhs, ret) => {
                write!(f, "{} - {} -> {}", lhs.show_ty(), rhs.show_ty(), ret.show_ty())
            },
            Self::Mul(lhs, rhs, ret) => {
                write!(f, "{} * {} -> {}", lhs.show_ty(), rhs.show_ty(), ret.show_ty())
            },
            Self::Exp(lhs, rhs, ret) => {
                write!(f, "{} ^ {} -> {}", lhs.show_ty(), rhs.show_ty(), ret.show_ty())
            },
        }
    }
}

#[macro_export]
macro_rules! bty {
    ($($bty:tt)+ -> $($ret:tt)+) => {{
        let b = $crate::bty!($($bty)+);
        b.result_mut().replace($crate::ty!($($ret)+));
        b
    }};
    // for pattern matching
    // equivalent to a `$name` in a match or let expression
    (any:$name:ident = $($rhs:tt)+) => {
        $crate::BinType::Eq($crate::ty!(any:$name), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    (? = $($rhs:tt)+) => {
        $crate::BinType::Eq($crate::ty!(?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    (_$([$($spec:tt)+])? = $($rhs:tt)+) => {
        $crate::BinType::Eq($crate::ty!(_$([$($spec)+])?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    ($sty:ident$([$($spec:tt)+])? = $($rhs:tt)+) => {
        $crate::BinType::Eq($crate::ty!($sty$([$($spec)+])?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    // for pattern matching
    // equivalent to a `$name` in a match or let expression
    (any:$name:ident + $($rhs:tt)+) => {
        $crate::BinType::Add($crate::ty!(any:$name), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    (? + $($rhs:tt)+) => {
        $crate::BinType::Add($crate::ty!(?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    (_$([$($spec:tt)+])? + $($rhs:tt)+) => {
        $crate::BinType::Add($crate::ty!(_$([$($spec)+])?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    ($sty:ident$([$($spec:tt)+])? + $($rhs:tt)+) => {
        $crate::BinType::Add($crate::ty!($sty$([$($spec)+])?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    // for pattern matching
    // equivalent to a `$name` in a match or let expression
    (any:$name:ident - $($rhs:tt)+) => {
        $crate::BinType::Sub($crate::ty!(any:$name), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    (? - $($rhs:tt)+) => {
        $crate::BinType::Sub($crate::ty!(?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    (_$([$($spec:tt)+])? - $($rhs:tt)+) => {
        $crate::BinType::Sub($crate::ty!(_$([$($spec)+])?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    ($sty:ident$([$($spec:tt)+])? - $($rhs:tt)+) => {
        $crate::BinType::Sub($crate::ty!($sty$([$($spec)+])?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    // for pattern matching
    // equivalent to a `$name` in a match or let expression
    (any:$name:ident * $($rhs:tt)+) => {
        $crate::BinType::Mul($crate::ty!(any:$name), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    (? * $($rhs:tt)+) => {
        $crate::BinType::Mul($crate::ty!(?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    (_$([$($spec:tt)+])? * $($rhs:tt)+) => {
        $crate::BinType::Mul($crate::ty!(_$([$($spec)+])?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    ($sty:ident$([$($spec:tt)+])? * $($rhs:tt)+) => {
        $crate::BinType::Mul($crate::ty!($sty$([$($spec)+])?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    // for pattern matching
    // equivalent to a `$name` in a match or let expression
    (any:$name:ident ^ $($rhs:tt)+) => {
        $crate::BinType::Exp($crate::ty!(any:$name), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    (? ^ $($rhs:tt)+) => {
        $crate::BinType::Exp($crate::ty!(?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    (_$([$($spec:tt)+])? ^ $($rhs:tt)+) => {
        $crate::BinType::Exp($crate::ty!(_$([$($spec)+])?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
    ($sty:ident$([$($spec:tt)+])? ^ $($rhs:tt)+) => {
        $crate::BinType::Exp($crate::ty!($sty$([$($spec)+])?), $crate::ty!($($rhs)+), $crate::ty!(?))
    };
}

impl BinType {
    /// Returns the type of the result of an equality based on the types
    /// of the left-hand side and right-hand side operands.
    /// If the types are not compatible, it returns a [TypeError::IncompatibleBinOp].
    ///
    /// Assuming shapes are compatible, the following table shows the result type
    /// based on the scalar types of the operands:
    /// ? == ?   || felt | bool | uint | _    | ?
    /// =========||======|======|======|======|=====
    /// felt     || bool | bool | bool | bool |    ?
    /// bool     || bool | bool | bool | bool |    ?
    /// uint     || bool | bool | bool | bool |    ?
    /// _        || bool | bool | bool | bool |    ?
    /// ?        ||    ? |    ? |    ? |    ? |    ?
    ///
    /// So, the result type of an equality is:
    /// - an error if lhs or rhs don't have a compatible shape,
    /// - symmetric over the operands,
    /// - any == ? -> ?,
    /// - always `bool` otherwise
    pub fn infer_bin_ty_eq(&self) -> Result<Option<Type>, TypeError> {
        if let Some(ret) = self.result() {
            return Ok(Some(ret));
        }
        let lhs = self.lhs();
        let rhs = self.rhs();
        if lhs.is_none() || rhs.is_none() {
            return Ok(ty!(?));
        }
        if self.lhs().is_shape_compatible(&self.rhs()) {
            Ok(ty!(bool))
        } else {
            Err(TypeError::IncompatibleBinOp { bin_ty: *self, span: None })
        }
    }

    /// Returns the type of the result of an addition based on the types
    /// of the left-hand side and right-hand side operands.
    /// If lhs or rhs is not a scalar type or `?`, it returns a [TypeError::IncompatibleShapes].
    ///
    /// based on the scalar types of the operands:
    /// ? + ?    || felt | bool | uint |    _ |    ?
    /// =========||======|======|======|======|=====
    /// felt     || felt | felt | felt | felt | felt
    /// bool     || felt | felt | felt | felt | felt
    /// uint     || felt | felt | uint |    _ |    ?
    /// _        || felt | felt |    _ |    _ |    ?
    /// ?        || felt | felt |    ? |    ? |    ?
    ///
    /// So, the result type of an addition is:
    /// - an error if lhs or rhs is not a scalar type or `?`,
    /// - symmetric over the operands,
    /// - felt + any  -> felt
    /// - bool + any  -> felt
    /// - ?    + any  -> ?
    /// - uint + uint -> uint
    /// - everything else is an unknown scalar type `_`
    pub fn infer_bin_ty_add(&self) -> Result<Option<Type>, TypeError> {
        if let Some(ret) = self.result() {
            return Ok(Some(ret));
        }
        let lhs = self.lhs();
        let rhs = self.rhs();
        if !((lhs.is_scalar() | lhs.is_none()) && (rhs.is_scalar() | rhs.is_none())) {
            return Err(TypeError::IncompatibleShapes { lhs, rhs, span: None });
        }
        match self {
            bty!(felt + any) | bty!(any + felt) => Ok(ty!(felt)),
            bty!(bool + any) | bty!(any + bool) => Ok(ty!(felt)),
            bty!(? + any) | bty!(any + ?) => Ok(ty!(?)),
            bty!(uint + uint) => Ok(ty!(uint)),
            _ => Ok(ty!(_)),
        }
    }

    /// Returns the type of the result of a substraction based on the types
    /// of the left-hand side and right-hand side operands.
    /// If lhs or rhs is not a scalar type or `?`, it returns a [TypeError::IncompatibleShapes].
    ///
    /// based on the scalar types of the operands:
    /// ? - ?    || felt | bool | uint |    _ |    ?
    /// =========||======|======|======|======|=====
    /// felt     || felt | felt | felt | felt | felt
    /// bool     || felt | felt | felt | felt | felt
    /// uint     || felt | felt | uint |    _ |    ?
    /// _        || felt | felt |    _ |    _ |    ?
    /// ?        || felt | felt |    ? |    ? |    ?
    ///
    /// So, the result type of a substraction is:
    /// - an error if either lhs or rhs is not a scalar type or `?`,
    /// - symmetric over the operands,
    /// - felt - any  -> felt
    /// - bool - any  -> felt
    /// - uint - uint -> uint
    /// - ?    - any  -> ?
    /// - everything else is an unknown scalar type `_`
    ///
    /// This is the same as [BinType::infer_bin_ty_add], so it reuses that method.
    ///
    /// NOTE: if we refine the types as described in #432, this method will need to be
    /// updated to handle the substraction of `bool` and `uint` types correctly.
    /// This will no longer be symmetric over the operands!
    /// Because:
    /// - 0    - bool = - bool -> felt
    /// - bool -    0          -> bool
    /// - 0    - uint = - uint -> uint (or error depending on the design)
    /// - uint -    0          -> uint
    /// - 1    - bool          -> bool
    /// - bool -    1          -> felt
    pub fn infer_bin_ty_sub(&self) -> Result<Option<Type>, TypeError> {
        self.infer_bin_ty_add()
    }

    /// Returns the type of the result of a multiplication based on the types
    /// of the left-hand side and right-hand side operands.
    /// If lhs or rhs is not a scalar type or `?`, it returns a [TypeError::IncompatibleShapes].
    ///
    /// based on the scalar types of the operands:
    /// ? * ?    || felt | bool | uint |    _ |    ?
    /// =========||======|======|======|======|=====
    /// felt     || felt | felt | felt | felt | felt
    /// bool     || felt | bool | uint |    _ |    ?
    /// uint     || felt | uint | uint |    _ |    ?
    /// _        || felt |    _ |    _ |    _ |    ?
    /// ?        || felt |    ? |    ? |    ? |    ?
    ///
    /// So, the result type of a multiplication is:
    /// - an error if either lhs or rhs is not a scalar type or `?`,
    /// - symmetric over the operands,
    /// - felt * any  -> felt
    /// - ?    * any  -> ?
    /// - _    * any  -> _
    /// - uint * uint -> uint
    /// - bool * x    -> x
    /// - everything else is an unknown scalar type `_`
    pub fn infer_bin_ty_mul(&self) -> Result<Option<Type>, TypeError> {
        if let Some(ret) = self.result() {
            return Ok(Some(ret));
        }
        let lhs = self.lhs();
        let rhs = self.rhs();
        if !((lhs.is_scalar() | lhs.is_none()) && (rhs.is_scalar() | rhs.is_none())) {
            return Err(TypeError::IncompatibleShapes { lhs, rhs, span: None });
        }
        match self {
            bty!(felt * any) | bty!(any * felt) => Ok(ty!(felt)),
            bty!(? * any) | bty!(any * ?) => Ok(ty!(?)),
            bty!(_ * any) | bty!(any * _) => Ok(ty!(_)),
            bty!(uint * uint) => Ok(ty!(uint)),
            bty!(bool * any:x) | bty!(any:x * bool) => Ok(*x),
            _ => Ok(ty!(_)),
        }
    }

    /// Returns the type of the result of an exponentiation based on the types
    /// of the left-hand side and right-hand side operands.
    /// If lhs is not a scalar type, or rhs is not `uint`,
    /// it returns a [TypeError::IncompatibleBinOp].
    ///
    /// based on the scalar types of the operands:
    /// ? ^ ?    || felt | bool | uint |    _ |    ?
    /// =========||======|======|======|======|=====
    /// felt     ||  err |  err | felt |  err |  err
    /// bool     ||  err |  err | bool |  err |  err
    /// uint     ||  err |  err | uint |  err |  err
    /// _        ||  err |  err |    _ |  err |  err
    /// ?        ||  err |  err |    ? |  err |  err
    ///
    /// So, the result type of an exponentiation is:
    /// - an error if either lhs or rhs isn't scalar types,
    /// - an error if the rhs is not an uint
    /// - the lhs type otherwise
    ///
    /// Because:
    /// - it is an error if rhs is not an uint
    /// - a bool to any power is still a bool:
    ///   - 0^n = 0
    ///   - 1^n = 1
    /// - a felt to any power is still a felt
    /// - an uint to any power is still an uint
    /// - a _ to any power is still a _
    /// - a ? to any power is still a ?
    pub fn infer_bin_ty_exp(&self) -> Result<Option<Type>, TypeError> {
        if let Some(ret) = self.result() {
            eprintln!("infer_bin_ty_exp: returning cached result {ret:?}");
            return Ok(Some(ret));
        }
        let lhs = self.lhs();
        let rhs = self.rhs();
        eprintln!("infer_bin_ty_exp: lhs = {lhs:?}, rhs = {rhs:?}");
        if !((lhs.is_scalar() | lhs.is_none()) && (rhs.is_scalar() | rhs.is_none())) {
            return Err(TypeError::IncompatibleBinOp { bin_ty: *self, span: None });
        }
        eprintln!("  MADE IT PAST THE SHAPE CHECK");
        match self {
            bty!(any ^ uint) => Ok(lhs),
            bty!(any ^ felt) | bty!(any ^ bool) | bty!(any ^ _) | bty!(any ^ ?) => {
                eprintln!("  ERROR: any ^ !uint");
                Err(TypeError::NonConstantExponent { bin_ty: *self, span: None })
            },
            _ => unreachable!("Undefined case for infer_bin_ty_exp: {self}"),
        }
    }
}

#[derive(Hash, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Value(Option<Type>),
    Aggregate(Vec<Option<Box<Kind>>>),
    Callable(FunctionType),
}

impl Default for Kind {
    fn default() -> Self {
        Self::Value(None)
    }
}

impl core::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(ty) => write!(f, "{}", ty.show_ty()),
            Self::Aggregate(tys) => {
                write!(
                    f,
                    "[{}]",
                    tys.iter()
                        .map(|ty| ty
                            .as_ref()
                            .map_or("?".to_string(), |k| k.show_kind().to_string()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            Self::Callable(fty) => write!(f, "{}", fty.show_fn_ty()),
        }
    }
}

#[macro_export]
macro_rules! kind {
    (ev $($spec:tt)+) => {
        $crate::Kind::Callable($crate::fty!(ev $($spec)+))
    };
    (fn ($($args:tt)*) -> $($ret:tt)+) => {
        $crate::Kind::Callable($crate::fty!(fn ($($args)*) -> $($ret)+))
    };
    ([$($spec:tt)+]) => {
        $crate::Kind::Aggregate(kinds!([$($spec)+]))
    };
    ($($spec:tt)+) => {
        $crate::Kind::Value($crate::ty!($($spec)+))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_scalar_type() {
        assert_eq!(sty!(_), None::<ScalarType>);
        assert_eq!(sty!(felt), Some(ScalarType::Felt));
        assert_eq!(sty!(bool), Some(ScalarType::Bool));
        assert_eq!(sty!(uint), Some(ScalarType::UInt));
    }

    #[test]
    fn test_macro_type() {
        assert_eq!(ty!(?), None::<Type>);
        assert_eq!(ty!(_), Some(Type::Scalar(None)));
        assert_eq!(ty!(felt), Some(Type::Scalar(Some(ScalarType::Felt))));
        assert_eq!(ty!(bool), Some(Type::Scalar(Some(ScalarType::Bool))));
        assert_eq!(ty!(uint), Some(Type::Scalar(Some(ScalarType::UInt))));
        assert_eq!(ty!(_[5]), Some(Type::Vector(None, 5)));
        assert_eq!(ty!(uint[5]), Some(Type::Vector(Some(ScalarType::UInt), 5)));
        assert_eq!(ty!(_[3, 4]), Some(Type::Matrix(None, 3, 4)));
        assert_eq!(ty!(felt[3, 4]), Some(Type::Matrix(Some(ScalarType::Felt), 3, 4)));
    }

    #[test]
    fn test_macro_trace_segment_type() {
        assert_eq!(tty!(a), ty!(felt));
        assert_eq!(tty!(a[5]), ty!(felt[5]));
        assert_eq!(tty!([]), Vec::<Option<Type>>::new());
        assert_eq!(tty!([a]), vec![ty!(felt)]);
        assert_eq!(tty!([a[5]]), vec![ty!(felt[5])]);
        assert_eq!(tty!([a[1], b[3]]), vec![ty!(felt), ty!(felt[3])]);
    }

    #[test]
    fn test_macro_function_type() {
        assert_eq!(fty!(ev([])), FunctionType::Evaluator(vec![]));
        assert_eq!(fty!(ev([a])), FunctionType::Evaluator(vec![ty!(felt)]));
        assert_eq!(fty!(ev([a[5]])), FunctionType::Evaluator(vec![ty!(felt[5])]));
        assert_eq!(fty!(ev([a, b[3]])), FunctionType::Evaluator(vec![ty!(felt), ty!(felt[3])]));
        assert_eq!(fty!(ev([a[1], b[3]])), FunctionType::Evaluator(vec![ty!(felt), ty!(felt[3])]));
        assert_eq!(fty!(ev([a[1], b[3]])), FunctionType::Evaluator(vec![ty!(felt), ty!(felt[3])]));

        assert_eq!(fty!(fn(uint) -> felt), FunctionType::Function(vec![ty!(uint)], ty!(felt)));
        assert_eq!(
            fty!(fn(uint[5]) -> felt[3, 4]),
            FunctionType::Function(vec![ty!(uint[5])], ty!(felt[3, 4]),)
        );
        assert_eq!(
            fty!(fn(uint[5], felt) -> felt[3, 4]),
            FunctionType::Function(vec![ty!(uint[5]), ty!(felt)], ty!(felt[3, 4]),)
        );
        assert_eq!(
            fty!(fn(uint[5], felt, bool[3, 4]) -> felt[3, 4]),
            FunctionType::Function(
                vec![ty!(uint[5]), ty!(felt), ty!(bool[3, 4]),],
                ty!(felt[3, 4]),
            )
        );
    }

    #[test]
    fn test_macro_bin_type() {
        assert_eq!(bty!(uint + felt), BinType::Add(ty!(uint), ty!(felt), ty!(?)));
        assert_eq!(bty!(_ - felt), BinType::Sub(ty!(_), ty!(felt), ty!(?)));
        assert_eq!(bty!(? = felt), BinType::Eq(ty!(?), ty!(felt), ty!(?)));
        assert_eq!(bty!(uint + ?), BinType::Add(ty!(uint), ty!(?), ty!(?)));
        assert_eq!(bty!(uint - felt), BinType::Sub(ty!(uint), ty!(felt), ty!(?)));
        assert_eq!(bty!(uint[2] * felt[2]), BinType::Mul(ty!(uint[2]), ty!(felt[2]), ty!(?)));
        assert_eq!(bty!(uint[2, 3] ^ _), BinType::Exp(ty!(uint[2, 3]), ty!(_), ty!(?)));
        assert_eq!(bty!(bool[5] = _[5]), BinType::Eq(ty!(bool[5]), ty!(_[5]), ty!(?)));
    }

    #[test]
    fn test_macro_kind() {
        assert_eq!(kind!(ev([])), Kind::Callable(fty!(ev([]))));
        assert_eq!(kind!(ev([a])), Kind::Callable(fty!(ev([a]))));
        assert_eq!(kind!(fn(uint) -> felt), Kind::Callable(fty!(fn(uint) -> felt)));
        assert_eq!(kind!(uint), Kind::Value(ty!(uint)));
        assert_eq!(kind!(_), Kind::Value(ty!(_)));
        assert_eq!(kind!(bool[3, 4]), Kind::Value(ty!(bool[3, 4])));
    }

    #[test]
    fn test_fn_ty_check_param_kinds() {
        // Scalar types
        assert!(fty!(fn(uint, felt) -> felt).check_args_kinds(&[&kind!(uint), &kind!(felt)]),);
        assert!(fty!(fn(felt, felt) -> felt).check_args_kinds(&[&kind!(felt), &kind!(bool)]),);
        // Vector types
        assert!(fty!(fn(_[3], felt[2]) -> felt).check_args_kinds(&[&kind!(_[3]), &kind!(felt[2])]));
        assert!(
            fty!(fn(_[3], felt[2]) -> felt).check_args_kinds(&[&kind!(bool[3]), &kind!(uint[2])])
        );
        // Aggregate types
        assert!(fty!(fn(felt[2], bool[3], uint[2]) -> felt).check_args_kinds(&[
            &kind!([bool, uint]),
            &kind!([bool, bool, bool]),
            &kind!([uint, uint]),
        ]));

        // Negative cases

        // Scalar types
        assert!(!fty!(fn(uint, bool) -> felt).check_args_kinds(&[&kind!(bool), &kind!(felt)]),);
        assert!(!fty!(fn(felt, bool) -> felt).check_args_kinds(&[&kind!(felt), &kind!(uint[2])]),);
        // Vector types
        assert!(!fty!(fn(_[3], felt[2]) -> felt).check_args_kinds(&[&kind!(_), &kind!(felt[2])]));
        assert!(
            !fty!(fn(_[3], felt[2]) -> felt)
                .check_args_kinds(&[&kind!(bool[3, 5]), &kind!(uint[2])])
        );
        // Aggregate types
        assert!(!fty!(fn(felt[2], bool[3], uint[2]) -> felt).check_args_kinds(&[
            &kind!([bool, uint]),
            &kind!([bool, felt, uint]),
            &kind!([uint, uint]),
        ]));
        assert!(!fty!(fn(felt[2], bool[3], uint[2]) -> felt).check_args_kinds(&[
            &kind!([bool, uint]),
            &kind!([bool, bool]),
            &kind!([uint, uint]),
        ]));
    }
}
