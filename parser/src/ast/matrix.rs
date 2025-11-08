//! Strided dense matrix representation for constant matrices.
//!
//! This module provides a memory-efficient representation of matrices using a single
//! flat vector with strided access patterns, replacing the nested `Vec<Vec<u64>>` structure.

use std::ops::Index;

use ref_cast::RefCast;

/// A strided dense matrix that stores elements in a single flat vector.
///
/// This provides more efficient memory usage and better cache locality compared to
/// `Vec<Vec<u64>>`, while maintaining the same double-indexing interface.
///
/// The matrix is stored in row-major order: `storage[row * cols + col]`
///
/// # Example
/// ```ignore
/// let matrix = Matrix::new(vec![
///     vec![1, 2, 3],
///     vec![4, 5, 6],
/// ]).unwrap();
/// assert_eq!(matrix[0][0], 1);
/// assert_eq!(matrix[0][1], 2);
/// assert_eq!(matrix[1][0], 4);
/// assert_eq!(matrix[1][2], 6);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Matrix {
    /// The number of rows in the matrix
    rows: usize,
    /// The number of columns in the matrix
    cols: usize,
    /// The flat storage containing all matrix elements in row-major order
    storage: Vec<u64>,
}

impl Matrix {
    /// Creates a new matrix from nested vectors.
    ///
    /// # Validation
    /// - All rows must have the same length
    /// - Matrix must have at least one row and one column
    ///
    /// # Errors
    /// Returns `None` if the validation fails.
    pub fn new(data: Vec<Vec<u64>>) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        let rows = data.len();
        let cols = data.first()?.len();

        // Validate that all rows have the same length
        for row in &data {
            if row.len() != cols {
                return None;
            }
        }

        // Flatten the data into a single vector
        let storage: Vec<u64> = data.into_iter().flatten().collect();

        Some(Self { rows, cols, storage })
    }

    /// Creates a new matrix from flat storage. Validates that the storage size matches rows * cols.
    pub fn from_flat(rows: usize, cols: usize, storage: Vec<u64>) -> Option<Self> {
        if rows == 0 || cols == 0 || storage.len() != rows * cols {
            return None;
        }

        Some(Self { rows, cols, storage })
    }

    /// Returns the number of rows in the matrix
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Returns the number of columns in the matrix
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Returns the dimensions of the matrix as (rows, cols)
    pub fn dimensions(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// Returns the total number of elements in the matrix
    pub fn len(&self) -> usize {
        self.rows * self.cols
    }

    /// Returns true if the matrix has no elements
    pub fn is_empty(&self) -> bool {
        self.rows == 0 || self.cols == 0
    }

    /// Returns the underlying flat storage slice
    pub fn as_slice(&self) -> &[u64] {
        &self.storage
    }

    /// Returns a specific element by row and column
    ///
    /// # Panics
    /// Panics if row >= rows or col >= cols
    pub fn get(&self, row: usize, col: usize) -> u64 {
        assert!(row < self.rows, "row index out of bounds");
        assert!(col < self.cols, "column index out of bounds");
        self.storage[row * self.cols + col]
    }

    /// Extracts a submatrix consisting of the specified row range.
    ///
    /// # Panics
    /// Panics if the range is out of bounds
    pub fn slice_rows(&self, range: std::ops::Range<usize>) -> Self {
        assert!(range.start <= range.end, "Invalid range");
        assert!(range.end <= self.rows, "Range end out of bounds");

        let new_rows = range.end - range.start;
        let mut new_storage = Vec::with_capacity(new_rows * self.cols);

        for row in range {
            let start = row * self.cols;
            let end = start + self.cols;
            new_storage.extend_from_slice(&self.storage[start..end]);
        }

        Self {
            rows: new_rows,
            cols: self.cols,
            storage: new_storage,
        }
    }

    /// Consumes the matrix and returns the flat storage
    pub fn into_vec(self) -> Vec<u64> {
        self.storage
    }
}

/// A view into a single row of a matrix, allowing double-indexing.
///
/// This type uses ref-cast to provide efficient zero-cost conversion from
/// a slice of the matrix storage to a row view.
#[derive(RefCast)]
#[repr(transparent)]
pub struct MatrixRow([u64]);

impl MatrixRow {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_vec(&self) -> Vec<u64> {
        self.0.to_vec()
    }
}

impl Index<usize> for MatrixRow {
    type Output = u64;

    fn index(&self, col: usize) -> &Self::Output {
        &self.0[col]
    }
}

impl std::ops::Deref for MatrixRow {
    type Target = [u64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Allows indexing into a matrix by row, returning a view that can be indexed by column.
///
/// # Example
/// ```ignore
/// let matrix = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
/// let row = &matrix[0];  // Returns &MatrixRow
/// let elem = row[1];     // Returns &u64
/// ```
impl Index<usize> for Matrix {
    type Output = MatrixRow;

    fn index(&self, row: usize) -> &Self::Output {
        assert!(row < self.rows, "row index out of bounds");
        let start = row * self.cols;
        let end = start + self.cols;
        MatrixRow::ref_cast(&self.storage[start..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_creation() {
        let data = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let matrix = Matrix::new(data.clone()).unwrap();

        assert_eq!(matrix.rows(), 2);
        assert_eq!(matrix.cols(), 3);
        assert_eq!(matrix.len(), 6);
        assert_eq!(matrix.dimensions(), (2, 3));
    }

    #[test]
    fn test_matrix_indexing() {
        let data = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let matrix = Matrix::new(data).unwrap();

        assert_eq!(matrix[0][0], 1);
        assert_eq!(matrix[0][1], 2);
        assert_eq!(matrix[0][2], 3);
        assert_eq!(matrix[1][0], 4);
        assert_eq!(matrix[1][1], 5);
        assert_eq!(matrix[1][2], 6);
    }

    #[test]
    fn test_matrix_get() {
        let data = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let matrix = Matrix::new(data).unwrap();

        assert_eq!(matrix.get(0, 0), 1);
        assert_eq!(matrix.get(0, 2), 3);
        assert_eq!(matrix.get(1, 0), 4);
        assert_eq!(matrix.get(1, 2), 6);
    }

    #[test]
    fn test_flat_storage() {
        let data = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let matrix = Matrix::new(data.clone()).unwrap();

        assert_eq!(matrix.as_slice(), &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_validation_different_row_lengths() {
        let data = vec![vec![1, 2], vec![3, 4, 5]]; // Different lengths
        assert!(Matrix::new(data).is_none());
    }

    #[test]
    fn test_validation_empty_matrix() {
        let data = vec![];
        assert!(Matrix::new(data).is_none());
    }

    #[test]
    fn test_from_flat() {
        let storage = vec![1, 2, 3, 4, 5, 6];
        let matrix = Matrix::from_flat(2, 3, storage).unwrap();

        assert_eq!(matrix.rows(), 2);
        assert_eq!(matrix.cols(), 3);
        assert_eq!(matrix[0][0], 1);
        assert_eq!(matrix[1][2], 6);
    }
}
