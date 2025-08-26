use std::fmt;

use air_types::*;

use crate::ast::{
    Access, AccessType, BusType, FunctionType, InvalidAccessError, TraceBinding, TraceSegment, Type,
};

/// This type provides type and contextual information about a binding,
/// i.e. not only does it tell us the type of a binding, but what type
/// of value was bound. This is used during analysis to check whether a
/// particular access is valid for the context it is in, as well as to
/// propagate type information while retaining information about where
/// the type was derived from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingType {
    /// A local variable whose value is not an alias of a global/module declaration
    Local(Type),
    /// A local variable that aliases a global/module declaration
    Alias(Box<BindingType>),
    /// A direct reference to a constant declaration
    Constant(Type),
    /// A type associated with a function signature
    ///
    /// The result type is None if the function is an evaluator
    Function(FunctionType),
    Evaluator(Vec<TraceSegment>),
    /// A binding to a bus definition
    Bus(BusType),
    /// A function parameter corresponding to trace columns
    TraceParam(TraceBinding),
    /// A direct reference to one or more contiguous trace columns
    TraceColumn(TraceBinding),
    /// A potentially non-contiguous set of trace columns
    Vector(Vec<BindingType>),
    /// A direct reference to a public input
    PublicInput(Type),
    /// A direct reference to a periodic column
    PeriodicColumn(usize),
}

impl Typing for BindingType {
    fn kind(&self) -> Option<Kind> {
        match self {
            Self::Alias(aliased) => aliased.kind(),
            Self::Local(ty) => ty.kind(),
            Self::Constant(ty) => ty.kind(),
            Self::Function(func) => func.kind(),
            Self::Evaluator(ev) => {
                Some(Kind::Callable(FunctionType::Evaluator(ev.iter().map(|tb| tb.ty()).collect())))
            },
            Self::Bus(_) => self.ty().kind(),
            Self::TraceColumn(tb) | Self::TraceParam(tb) => tb.kind(),
            Self::Vector(elems) => elems.kind(),
            Self::PublicInput(ty) => ty.kind(),
            // NOTE: this may need to be felt?
            Self::PeriodicColumn(_) => Some(kind!(bool)),
        }
    }
    /// Get the value type of this binding, if applicable
    fn ty(&self) -> Option<Type> {
        match self {
            Self::TraceColumn(tb) | Self::TraceParam(tb) => tb.ty(),
            Self::Vector(elems) => elems.ty(),
            Self::Alias(aliased) => aliased.ty(),
            Self::Local(ty) | Self::Constant(ty) | Self::PublicInput(ty) => Some(*ty),
            Self::PeriodicColumn(_) => ty!(felt),
            Self::Function(_) => None,
            Self::Evaluator(_) => None,
            Self::Bus(_) => ty!(felt),
        }
    }
}

impl Access for BindingType {
    type Accessed = Self;
    /// Produce a new [BindingType] which represents accessing the current binding via `access_type`
    fn access(&self, access_type: AccessType) -> Result<Self::Accessed, InvalidAccessError> {
        match self {
            Self::Alias(aliased) => aliased.access(access_type),
            Self::Local(ty) => ty.access(access_type).map(Self::Local),
            Self::Constant(ty) => {
                ty.access(access_type).map(|t| Self::Alias(Box::new(Self::Constant(t))))
            },
            Self::TraceColumn(tb) => tb.access(access_type).map(Self::TraceColumn),
            Self::TraceParam(tb) => tb.access(access_type).map(Self::TraceParam),
            Self::Vector(elems) => match access_type {
                AccessType::Default => Ok(Self::Vector(elems.clone())),
                AccessType::Index(idx) if idx >= elems.len() => {
                    Err(InvalidAccessError::IndexOutOfBounds)
                },
                AccessType::Index(idx) => Ok(elems[idx].clone()),
                AccessType::Slice(range) => {
                    let slice_range = range.to_slice_range();
                    if slice_range.end > elems.len() {
                        Err(InvalidAccessError::IndexOutOfBounds)
                    } else {
                        Ok(Self::Vector(elems[slice_range].to_vec()))
                    }
                },
                AccessType::Matrix(row, _) if row >= elems.len() => {
                    Err(InvalidAccessError::IndexOutOfBounds)
                },
                AccessType::Matrix(row, col) => elems[row].access(AccessType::Index(col)),
            },
            Self::PublicInput(ty) => ty.access(access_type).map(Self::PublicInput),
            Self::PeriodicColumn(period) => match access_type {
                AccessType::Default => Ok(Self::PeriodicColumn(*period)),
                _ => Err(InvalidAccessError::IndexIntoScalar),
            },
            Self::Function(_) | Self::Evaluator(_) => Err(InvalidAccessError::InvalidBinding),
            Self::Bus(bus) => Ok(Self::Bus(*bus)),
        }
    }
}

impl fmt::Display for BindingType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Update to reflect the type signature
        match self {
            Self::Alias(aliased) => write!(f, "{aliased}"),
            Self::Local(_) => f.write_str("local"),
            Self::Constant(_) => f.write_str("constant"),
            Self::Vector(_) => f.write_str("vector"),
            Self::Function(_) => f.write_str("function"),
            Self::Evaluator(_) => f.write_str("evaluator"),
            Self::TraceColumn(_) | Self::TraceParam(_) => f.write_str("trace column(s)"),
            Self::PublicInput(_) => f.write_str("public input(s)"),
            Self::PeriodicColumn(_) => f.write_str("periodic column(s)"),
            Self::Bus(_) => f.write_str("bus"),
        }
    }
}
