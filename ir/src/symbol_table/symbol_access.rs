enum AccessType {
    Default,
    Vector(idx),
    Matrix(row_idx, col_idx),
}

struct SymbolAccess {
    symbol: Symbol,
    access_type: AccessType,
}

impl SymbolAccess {
    pub fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    pub fn access_type(&self) -> &AccessType {
        &self.access_type
    }
}
