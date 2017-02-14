use matrix::{Matrix, BaseMatrix};
use error::{Error, ErrorKind};
use matrix::decomposition::Decomposition;
use matrix::forward_substitution;
use vector::Vector;
use utils::dot;

use std::any::Any;

use libnum::{Zero, Float};

/// TODO
#[derive(Clone, Debug)]
pub struct Cholesky<T> {
    l: Matrix<T>
}

impl<T> Cholesky<T> where T: 'static + Float {
    /// TODO
    pub fn decompose(matrix: Matrix<T>) -> Result<Self, Error> {
        assert!(matrix.rows() == matrix.cols(),
            "Matrix must be square for Cholesky decomposition.");
        let n = matrix.rows();

        // The implementation here is based on the
        // "Gaxpy-Rich Cholesky Factorization"
        // from Chapter 4.2.5 in
        // Matrix Computations, 4th Edition,
        // (Golub and Van Loan).

        // We consume the matrix we're given, and overwrite its
        // lower diagonal part with the L factor. However,
        // we ignore the strictly upper triangular part of the matrix,
        // because this saves us a few operations.
        // When the decomposition is unpacked, we will completely zero
        // the upper triangular part.
        let mut a = matrix;

        // Resolve each submatrix (j .. n, j .. n)
        for j in 0 .. n {
            if j > 0 {
                // This is essentially a GAXPY operation y = y - Bx
                // where B is the [j .. n, 0 .. j] submatrix of A,
                // x is the [ j, 0 .. j ] submatrix of A,
                // and y is the [ j .. n, j ] submatrix of A
                for k in j .. n {
                    let kj_dot = {
                        let j_row = a.row(j).raw_slice();
                        let k_row = a.row(k).raw_slice();
                        dot(&k_row[0 .. j], &j_row[0 .. j])
                    };
                    a[[k, j]] = a[[k, j]] - kj_dot;
                }
            }

            let diagonal = a[[j, j]];
            if diagonal.abs() < T::epsilon() {
                return Err(Error::new(ErrorKind::DecompFailure,
                    "Matrix is singular to working precision."));
            } else if diagonal < T::zero() {
                return Err(Error::new(ErrorKind::DecompFailure,
                    "Diagonal entries of matrix are not all positive."));
            }

            let divisor = diagonal.sqrt();
            for k in j .. n {
                a[[k, j]] = a[[k, j]] / divisor;
            }
        }

        Ok(Cholesky {
            l: a
        })
    }

    /// TODO
    pub fn det(&self) -> T {
        let l_det = self.l.diag()
                          .cloned()
                          .fold(T::one(), |a, b| a * b);
        l_det * l_det
    }

    /// TODO
    pub fn solve(&self, b: Vector<T>) -> Vector<T> {
        assert!(self.l.rows() == b.size(),
            "RHS vector and coefficient matrix must be
             dimensionally compatible.");
        // Solve Ly = b
        let y = forward_substitution(&self.l, b)
                    .expect("Internal error: L should be invertible.");
        // Solve L^T x = y
        transpose_back_substitution(&self.l, y)
            .expect("Internal error: L^T should be invertible.")
    }
}

impl<T: Zero> Decomposition for Cholesky<T> {
    type Factors = Matrix<T>;

    fn unpack(self) -> Matrix<T> {
        use internal_utils::nullify_upper_triangular_part;
        let mut l = self.l;
        nullify_upper_triangular_part(&mut l);
        l
    }
}


impl<T> Matrix<T>
    where T: Any + Float
{
    /// Cholesky decomposition
    ///
    /// Returns the cholesky decomposition of a positive definite matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::Matrix;
    ///
    /// let m = matrix![1.0, 0.5, 0.5;
    ///                 0.5, 1.0, 0.5;
    ///                 0.5, 0.5, 1.0];
    ///
    /// let l = m.cholesky();
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrix is not square.
    ///
    /// # Failures
    ///
    /// - Matrix is not positive definite.
    pub fn cholesky(&self) -> Result<Matrix<T>, Error> {
        assert!(self.rows == self.cols,
                "Matrix must be square for Cholesky decomposition.");

        let mut new_data = Vec::<T>::with_capacity(self.rows() * self.cols());

        for i in 0..self.rows() {

            for j in 0..self.cols() {

                if j > i {
                    new_data.push(T::zero());
                    continue;
                }

                let mut sum = T::zero();
                for k in 0..j {
                    sum = sum + (new_data[i * self.cols() + k] * new_data[j * self.cols() + k]);
                }

                if j == i {
                    new_data.push((self[[i, i]] - sum).sqrt());
                } else {
                    let p = (self[[i, j]] - sum) / new_data[j * self.cols + j];

                    if !p.is_finite() {
                        return Err(Error::new(ErrorKind::DecompFailure,
                                              "Matrix is not positive definite."));
                    } else {

                    }
                    new_data.push(p);
                }
            }
        }

        Ok(Matrix {
            rows: self.rows(),
            cols: self.cols(),
            data: new_data,
        })
    }
}

/// Solves the square system L^T x = b,
/// where L is lower triangular
fn transpose_back_substitution<T>(l: &Matrix<T>, b: Vector<T>)
    -> Result<Vector<T>, Error> where T: Float {
    assert!(l.rows() == l.cols(), "Matrix L must be square.");
    assert!(l.rows() == b.size(), "L and b must be dimensionally compatible.");
    let n = l.rows();
    let mut x = b;

    // TODO: Make this implementation more cache efficient
    // At the moment it is a simple naive (very cache inefficient)
    // implementation for the sake of correctness.
    for i in (0 .. n).rev() {
        let mut inner_product = T::zero();
        for j in (i + 1) .. n {
            inner_product = inner_product + l[[j, i]] * x[j];
        }

        let diagonal = l[[i, i]];
        if diagonal.abs() < T::epsilon() {
            return Err(Error::new(ErrorKind::DivByZero,
                "Matrix L is singular to working precision."));
        }

        x[i] = (x[i] - inner_product) / diagonal;
    }

    Ok(x)
}

#[cfg(test)]
mod tests {
    use matrix::Matrix;
    use matrix::decomposition::Decomposition;
    use vector::Vector;

    use super::Cholesky;
    use super::transpose_back_substitution;

    use libnum::Float;
    use quickcheck::TestResult;

    #[test]
    #[should_panic]
    fn test_non_square_cholesky() {
        let a = Matrix::<f64>::ones(2, 3);

        let _ = a.cholesky();
    }

    #[test]
    fn cholesky_unpack_empty() {
        let x: Matrix<f64> = matrix![];
        let l = Cholesky::decompose(x.clone())
                            .unwrap()
                            .unpack();
        assert_matrix_eq!(l, x);
    }

    #[test]
    fn cholesky_unpack_1x1() {
        let x = matrix![ 4.0 ];
        let expected = matrix![ 2.0 ];
        let l = Cholesky::decompose(x)
                            .unwrap()
                            .unpack();
        assert_matrix_eq!(l, expected, comp = float);
    }

    #[test]
    fn cholesky_unpack_2x2() {
        {
            let x = matrix![ 9.0, -6.0;
                            -6.0, 20.0];
            let expected = matrix![ 3.0, 0.0;
                                   -2.0, 4.0];

            let l = Cholesky::decompose(x)
                        .unwrap()
                        .unpack();
            assert_matrix_eq!(l, expected, comp = float);
        }
    }

    #[test]
    fn cholesky_singular_fails() {
        {
            let x = matrix![0.0];
            assert!(Cholesky::decompose(x).is_err());
        }

        {
            let x = matrix![0.0, 0.0;
                            0.0, 1.0];
            assert!(Cholesky::decompose(x).is_err());
        }

        {
            let x = matrix![1.0, 0.0;
                            0.0, 0.0];
            assert!(Cholesky::decompose(x).is_err());
        }

        {
            let x = matrix![1.0,   3.0,   5.0;
                            3.0,   9.0,  15.0;
                            5.0,  15.0,  65.0];
            assert!(Cholesky::decompose(x).is_err());
        }
    }

    #[test]
    fn cholesky_det_empty() {
        let x: Matrix<f64> = matrix![];
        let cholesky = Cholesky::decompose(x).unwrap();
        assert_eq!(cholesky.det(), 1.0);
    }

    #[test]
    fn cholesky_det() {
        {
            let x = matrix![1.0];
            let cholesky = Cholesky::decompose(x).unwrap();
            let diff = cholesky.det() - 1.0;
            assert!(diff.abs() < 1e-14);
        }

        {
            let x = matrix![1.0,   3.0,   5.0;
                            3.0,  18.0,  33.0;
                            5.0,  33.0,  65.0];
            let cholesky = Cholesky::decompose(x).unwrap();
            let diff = cholesky.det() - 36.0;
            assert!(diff.abs() < 1e-14);
        }
    }

    #[test]
    fn cholesky_solve_examples() {
        {
            // TODO: Enable this test
            // It is currently commented out because
            // backward/forward substitution don't handle
            // empty matrices. See PR #152 for more details.

            // let a: Matrix<f64> = matrix![];
            // let b: Vector<f64> = vector![];
            // let expected: Vector<f64> = vector![];
            // let cholesky = Cholesky::decompose(a).unwrap();
            // let x = cholesky.solve(b);
            // assert_eq!(x, expected);
        }

        {
            let a = matrix![ 1.0 ];
            let b = vector![ 4.0 ];
            let expected = vector![ 4.0 ];
            let cholesky = Cholesky::decompose(a).unwrap();
            let x = cholesky.solve(b);
            assert_vector_eq!(x, expected, comp = float);
        }

        {
            let a = matrix![ 4.0,  6.0;
                             6.0, 25.0];
            let b = vector![ 2.0,  4.0];
            let expected = vector![ 0.40625,  0.0625 ];
            let cholesky = Cholesky::decompose(a).unwrap();
            let x = cholesky.solve(b);
            assert_vector_eq!(x, expected, comp = float);
        }
    }

    quickcheck! {
        fn property_cholesky_of_identity_is_identity(n: usize) -> TestResult {
            if n > 30 {
                return TestResult::discard();
            }

            let x = Matrix::<f64>::identity(n);
            let l = Cholesky::decompose(x.clone()).map(|c| c.unpack());
            match l {
                Ok(l) => {
                    assert_matrix_eq!(l, x, comp = float);
                    TestResult::passed()
                },
                _ => TestResult::failed()
            }
        }
    }

    #[test]
    fn transpose_back_substitution_examples() {
        {
            let l: Matrix<f64> = matrix![];
            let b: Vector<f64> = vector![];
            let expected: Vector<f64> = vector![];
            let x = transpose_back_substitution(&l, b).unwrap();
            assert_vector_eq!(x, expected);
        }

        {
            let l = matrix![2.0];
            let b = vector![2.0];
            let expected = vector![1.0];
            let x = transpose_back_substitution(&l, b).unwrap();
            assert_vector_eq!(x, expected, comp = float);
        }

        {
            let l = matrix![2.0, 0.0;
                            3.0, 4.0];
            let b = vector![2.0, 1.0];
            let expected = vector![0.625, 0.25 ];
            let x = transpose_back_substitution(&l, b).unwrap();
            assert_vector_eq!(x, expected, comp = float);
        }

        {
            let l = matrix![ 2.0,  0.0,  0.0;
                             5.0, -1.0,  0.0;
                            -2.0,  0.0,  1.0];
            let b = vector![-1.0, 2.0, 3.0];
            let expected = vector![ 7.5, -2.0, 3.0 ];
            let x = transpose_back_substitution(&l, b).unwrap();
            assert_vector_eq!(x, expected, comp = float);
        }
    }
}
