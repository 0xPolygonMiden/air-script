use super::Identifier;

// RANDOM VALUES
// ================================================================================================

/// Declaration of random values for an AIR. Random values could be represented by a named
/// identifier `name` which is used to identify a fixed size array of length `size` and an empty
/// `bindings` vector or by a named identifier `name` which is used to identify a `bindings`
/// [RandBinding] vector and its `size` field.
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
/// `RandomValues { size: 15, bindings: [ RandBinding { name: "$rand", size: 15 } ] }`
///
/// If random values are declared in form
///
/// ```airscript
/// random_values:
///     rand: [a, b[12]]
/// ```
///
/// created [RandomValues] instance will look like
///
/// `RandomValues { size: 13, bindings: [ RandBinding { name: "$rand", size: 13 },
///                                       RandBinding { name: "a", size: 1 },
///                                       RandBinding { name: "b", size: 12 } ] }`
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RandomValues {
    num_values: u64,
    bindings: Vec<RandBinding>,
}

impl RandomValues {
    pub(crate) fn new(num_values: u64, bindings: Vec<RandBinding>) -> Self {
        Self {
            num_values,
            bindings,
        }
    }

    pub fn num_values(&self) -> u64 {
        self.num_values
    }

    pub fn bindings(&self) -> &Vec<RandBinding> {
        &self.bindings
    }

    pub fn into_parts(self) -> (u64, Vec<RandBinding>) {
        (self.num_values, self.bindings)
    }
}

/// Declaration of a random value binding used in [RandomValues]. It is represented by a named
/// identifier and its size.
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

    pub fn into_parts(self) -> (String, u64) {
        (self.name.into_name(), self.size)
    }
}
