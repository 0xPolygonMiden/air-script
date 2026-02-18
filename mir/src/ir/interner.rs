use std::collections::HashMap;

use miden_diagnostics::SourceSpan;

use crate::ir::{
    Accessor, Add, Exp, Link, Matrix, MirAccessType, Mul, Op, SpannedMirValue, Sub, Value, Vector,
};

#[derive(Hash, Eq, PartialEq, Clone)]
enum AccessKey {
    Default,
    Index { index: usize },
    Matrix { row: usize, col: usize },
}

impl AccessKey {
    fn from_access_type(access_type: &MirAccessType) -> Self {
        match access_type {
            MirAccessType::Default => AccessKey::Default,
            MirAccessType::Index(index) => AccessKey::Index { index: index.get_ptr() },
            MirAccessType::Matrix(row, col) => {
                AccessKey::Matrix { row: row.get_ptr(), col: col.get_ptr() }
            },
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
enum OpKey {
    Add {
        lhs: usize,
        rhs: usize,
        span: SourceSpan,
    },
    Sub {
        lhs: usize,
        rhs: usize,
        span: SourceSpan,
    },
    Mul {
        lhs: usize,
        rhs: usize,
        span: SourceSpan,
    },
    Exp {
        lhs: usize,
        rhs: usize,
        span: SourceSpan,
    },
    Vector {
        elements: Vec<usize>,
        span: SourceSpan,
    },
    Matrix {
        rows: Vec<usize>,
        span: SourceSpan,
    },
    Accessor {
        indexable: usize,
        access: AccessKey,
        offset: usize,
        span: SourceSpan,
    },
    Value(SpannedMirValue),
}

/// A small, conservative interner for MIR ops. Intended for use in hot paths.
pub struct OpInterner {
    map: HashMap<OpKey, Link<Op>>,
}

impl OpInterner {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn intern_add(&mut self, lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let key = OpKey::Add {
            lhs: lhs.get_ptr(),
            rhs: rhs.get_ptr(),
            span,
        };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Add::create(lhs, rhs, span);
        self.map.insert(key, node.clone());
        node
    }

    pub fn intern_sub(&mut self, lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let key = OpKey::Sub {
            lhs: lhs.get_ptr(),
            rhs: rhs.get_ptr(),
            span,
        };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Sub::create(lhs, rhs, span);
        self.map.insert(key, node.clone());
        node
    }

    pub fn intern_mul(&mut self, lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let key = OpKey::Mul {
            lhs: lhs.get_ptr(),
            rhs: rhs.get_ptr(),
            span,
        };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Mul::create(lhs, rhs, span);
        self.map.insert(key, node.clone());
        node
    }

    pub fn intern_exp(&mut self, lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let key = OpKey::Exp {
            lhs: lhs.get_ptr(),
            rhs: rhs.get_ptr(),
            span,
        };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Exp::create(lhs, rhs, span);
        self.map.insert(key, node.clone());
        node
    }

    pub fn intern_vector(&mut self, elements: Vec<Link<Op>>, span: SourceSpan) -> Link<Op> {
        let elem_ptrs = elements.iter().map(|elem| elem.get_ptr()).collect();
        let key = OpKey::Vector { elements: elem_ptrs, span };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Vector::create(elements, span);
        self.map.insert(key, node.clone());
        node
    }

    pub fn intern_matrix(&mut self, rows: Vec<Link<Op>>, span: SourceSpan) -> Link<Op> {
        let row_ptrs = rows.iter().map(|row| row.get_ptr()).collect();
        let key = OpKey::Matrix { rows: row_ptrs, span };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Matrix::create(rows, span);
        self.map.insert(key, node.clone());
        node
    }

    pub fn intern_accessor(
        &mut self,
        indexable: Link<Op>,
        access_type: MirAccessType,
        offset: usize,
        span: SourceSpan,
    ) -> Link<Op> {
        let access = AccessKey::from_access_type(&access_type);
        let key = OpKey::Accessor {
            indexable: indexable.get_ptr(),
            access,
            offset,
            span,
        };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Accessor::create(indexable, access_type, offset, span);
        self.map.insert(key, node.clone());
        node
    }

    pub fn intern_value(&mut self, value: SpannedMirValue) -> Link<Op> {
        let key = OpKey::Value(value.clone());
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Value::create(value);
        self.map.insert(key, node.clone());
        node
    }
}
