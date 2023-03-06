use std::fmt;

/// [Identifier] is used to represent variable names.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct Identifier(pub String);

impl Identifier {
    /// Returns the name of the identifier.
    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn into_name(self) -> String {
        self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
