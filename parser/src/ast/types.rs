use super::*;
pub use typing::*;
pub use typing::{bty, fty, kind, sty, tty, ty, tys};

impl Access for Type {
    type Accessed = Self;
    /// Return a new [Type] representing the type of the value produced by the given [AccessType]
    fn access(&self, access_type: AccessType) -> Result<Self::Accessed, InvalidAccessError> {
        match *self {
            ty if access_type == AccessType::Default => Ok(ty),
            Self::Scalar(_) => Err(InvalidAccessError::IndexIntoScalar),
            Self::Vector(sty, len) => match access_type {
                AccessType::Slice(range) => {
                    let slice_range = range.to_slice_range();
                    if slice_range.end > len {
                        Err(InvalidAccessError::IndexOutOfBounds)
                    } else {
                        Ok(Self::Vector(sty, slice_range.len()))
                    }
                },
                AccessType::Index(idx) if idx >= len => Err(InvalidAccessError::IndexOutOfBounds),
                AccessType::Index(_) => Ok(Self::Scalar(sty)),
                AccessType::Matrix(..) => Err(InvalidAccessError::IndexIntoScalar),
                _ => unreachable!(),
            },
            Self::Matrix(sty, rows, cols) => match access_type {
                AccessType::Slice(range) => {
                    let slice_range = range.to_slice_range();
                    if slice_range.end > rows {
                        Err(InvalidAccessError::IndexOutOfBounds)
                    } else {
                        Ok(Self::Matrix(sty, slice_range.len(), cols))
                    }
                },
                AccessType::Index(idx) if idx >= rows => Err(InvalidAccessError::IndexOutOfBounds),
                AccessType::Index(_) => Ok(Self::Vector(sty, cols)),
                AccessType::Matrix(row, col) if row >= rows || col >= cols => {
                    Err(InvalidAccessError::IndexOutOfBounds)
                },
                AccessType::Matrix(..) => Ok(Self::Scalar(sty)),
                _ => unreachable!(),
            },
        }
    }
}
