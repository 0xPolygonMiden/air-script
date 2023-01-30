use super::Identifier;

// RANDOM VALUES
// ================================================================================================

/// Declaration of random values for an AIR. Random values colud be represented by a named
/// identifier which is used to identify a fixed size array of length `size` and empty `bindings`
/// vector or by named identifier which is used to identify a `bindings` [RandBinding] vector and
/// its `size` field.
///
/// # Examples
///
/// If random values are declared in form
///
/// ```airscript
/// random_values:
///     rand: [15]
/// ```
///
/// created [RandomValues] instance will look like
///
/// `RandomValues{name: "rand", size: 15, bindings: []}`
///
/// If random values declared in form
///
/// ```airscript
/// random_values:
///     rand: [a, b[12]]
/// ```
///
/// created [RandomValues] instance will look like
///
/// `RandomValues{name: "rand", size: 2, bindings: [RandBinding{name: "a", size: 1}, RandBinding{name: "b", size: 12}]}`
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RandomValues {
    name: Identifier,
    size: u64,
    bindings: Vec<RandBinding>,
}

impl RandomValues {
    pub(crate) fn new(name: Identifier, size: u64, bindings: Vec<RandBinding>) -> Self {
        Self {
            name,
            size,
            bindings,
        }
    }

    pub fn name(&self) -> &str {
        let Identifier(name) = &self.name;
        name
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn bindings(&self) -> &Vec<RandBinding> {
        &self.bindings
    }
}

/// Declaration of a random value used in [RandomValues]. It is represented by a named identifier and its size.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RandBinding {
    name: Identifier,
    size: u64,
}

impl RandBinding {
    pub(crate) fn new(name: Identifier, size: u64) -> Self {
        Self { name, size }
    }

    pub fn name(&self) -> &str {
        let Identifier(name) = &self.name;
        name
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}
