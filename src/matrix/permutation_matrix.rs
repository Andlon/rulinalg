use matrix::BaseMatrixMut;
use std::ops::Mul;
// use std::any::Any;
use std;

use utils::Permutation;

/// TODO
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PermutationMatrix<T> {
    // An N x N permutation matrix P can be seen as a map
    // { 1, ..., N } -> { 1, ..., N }
    // Hence, we merely store N indices, such that
    // perm[[i]] = j
    // means that index i is mapped to index j
    perm: Permutation,

    // Currently, we need to let PermutationMatrix be generic over T,
    // because BaseMatrixMut is.
    marker: std::marker::PhantomData<T>
}

impl<T> PermutationMatrix<T> {
    /// The identity permutation.
    pub fn identity(n: usize) -> Self {
        PermutationMatrix {
            perm: Permutation::identity(n),
            marker: std::marker::PhantomData
        }
    }

    /// Swaps indices i and j
    pub fn swap(&mut self, i: usize, j: usize) {
        self.perm.swap(i, j);
    }

    /// The inverse of the permutation matrix.
    pub fn inverse(&self) -> PermutationMatrix<T> {
        PermutationMatrix {
            perm: self.perm.inverse(),
            marker: std::marker::PhantomData
        }
    }

    /// The dimensions of the permutation matrix.
    ///
    /// A permutation matrix is a square matrix, so `dim()` is equal
    /// to both the number of rows, as well as the number of columns.
    pub fn dim(&self) -> usize {
        self.perm.cardinality()
    }

    /// The permutation matrix in an equivalent full matrix representation.
    pub fn as_matrix(&self) -> Matrix<T> {
        unimplemented!();
    }
}

impl<T> From<Permutation> for PermutationMatrix<T> {
    fn from(perm: Permutation) -> Self {
        PermutationMatrix {
            perm: perm,
            marker: std::marker::PhantomData
        }
    }
}
