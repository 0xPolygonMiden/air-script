use super::Identifier;

// PERIODIC COLUMNS
// ================================================================================================

/// Declaration of a periodic column for an AIR. Periodic columns are columns with repeating cycles
/// of values. The values declared for the periodic column should be the cycle of values that will
/// be repeated. The length of the values vector is expected to be a power of 2 with a minimum
/// length of 2, but this is not enforced here.
#[derive(Debug, Eq, PartialEq)]
pub struct PeriodicColumn {
    name: Identifier,
    values: Vec<u64>,
}

impl PeriodicColumn {
    pub(crate) fn new(name: Identifier, values: Vec<u64>) -> Self {
        Self { name, values }
    }

    pub fn name(&self) -> &str {
        let Identifier(name) = &self.name;
        name
    }

    pub fn period(&self) -> usize {
        self.values.len()
    }

    pub fn values(&self) -> &[u64] {
        &self.values
    }
}
