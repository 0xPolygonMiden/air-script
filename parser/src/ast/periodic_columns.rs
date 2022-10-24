use super::Identifier;

// PERIODIC COLUMNS
// ================================================================================================

/// TODO
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
