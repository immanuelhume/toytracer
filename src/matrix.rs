use crate::tuple::{Point, Tuple};
use crate::EPSILON;
use std::ops;

#[derive(Debug, Clone, Copy)]
pub struct Matrix<const M: usize, const N: usize>([[f64; N]; M]);

impl<const M: usize, const N: usize> PartialEq for Matrix<M, N> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..M {
            for j in 0..N {
                if (self.0[i][j] - other.0[i][j]).abs() > EPSILON {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug)]
pub enum Error {
    Uninvertible,
}

impl<const M: usize, const N: usize> Matrix<M, N> {
    pub fn new(xss: [[f64; N]; M]) -> Self {
        Self(xss)
    }

    fn get(&self, i: usize, j: usize) -> f64 {
        self.0[i][j]
    }

    pub fn set(&mut self, i: usize, j: usize, val: f64) {
        self.0[i][j] = val
    }

    /// Matrix multiplication.
    fn mult<const K: usize>(&self, other: &Matrix<N, K>) -> Matrix<M, K> {
        let mut xss = [[0.0; K]; M];
        for i in 0..M {
            for j in 0..K {
                let mut v = 0.0;
                for k in 0..N {
                    v += self.0[i][k] * other.0[k][j];
                }
                xss[i][j] = v;
            }
        }
        Matrix(xss)
    }
}

impl Matrix<2, 2> {
    fn det(&self) -> f64 {
        let [[a, b], [c, d]] = self.0;
        a * d - b * c
    }

    fn is_invertible(&self) -> bool {
        self.det() != 0.0
    }
}

impl Matrix<3, 3> {
    fn det(&self) -> f64 {
        // just expand along the first row
        let mut res = 0.0;
        for j in 0..3 {
            let x = self.0[0][j];
            res += x * self.cofactor(0, j);
        }
        res
    }

    fn minor(&self, i: usize, j: usize) -> f64 {
        self.submatrix(i, j).det()
    }

    fn cofactor(&self, i: usize, j: usize) -> f64 {
        let x = self.submatrix(i, j).det();
        if (i + j) % 2 == 0 {
            x
        } else {
            -x
        }
    }

    fn is_invertible(&self) -> bool {
        self.det() != 0.0
    }
}

impl Matrix<4, 4> {
    fn mult_tuple(&self, t: Tuple) -> Tuple {
        let Tuple(a, b, c, d) = t;
        let mut tmp = [0.0; 4];
        for i in 0..4 {
            let [w, x, y, z] = self.0[i];
            tmp[i] = w * a + x * b + y * c + z * d;
        }
        Tuple(tmp[0], tmp[1], tmp[2], tmp[3])
    }

    fn det(&self) -> f64 {
        // just expand along the first row
        let mut res = 0.0;
        for j in 0..4 {
            let x = self.0[0][j];
            res += x * self.cofactor(0, j);
        }
        res
    }

    fn minor(&self, i: usize, j: usize) -> f64 {
        self.submatrix(i, j).det()
    }

    fn cofactor(&self, i: usize, j: usize) -> f64 {
        let x = self.submatrix(i, j).det();
        if (i + j) % 2 == 0 {
            x
        } else {
            -x
        }
    }

    fn is_invertible(&self) -> bool {
        self.det() != 0.0
    }

    pub fn inverse(&self) -> Result<Self, Error> {
        if !self.is_invertible() {
            return Err(Error::Uninvertible);
        }
        let mut xss = [[0.0; 4]; 4];
        let det = self.det();
        for i in 0..4 {
            for j in 0..4 {
                xss[j][i] = self.cofactor(i, j) / det;
            }
        }
        Ok(Self(xss))
    }
}

impl<const K: usize> Matrix<K, K> {
    pub fn ident() -> Self {
        let mut xss = [[0.0; K]; K];
        for i in 0..K {
            for j in 0..K {
                xss[i][j] = if i == j { 1.0 } else { 0.0 };
            }
        }
        Matrix(xss)
    }

    fn transpose(&self) -> Self {
        let mut xss = self.0.clone();
        for i in 0..K {
            for j in i..K {
                (xss[j][i], xss[i][j]) = (xss[i][j], xss[j][i]);
            }
        }
        Matrix(xss)
    }

    fn submatrix(&self, i: usize, j: usize) -> Matrix<{ K - 1 }, { K - 1 }>
    where
        [(); K - 1]:,
    {
        let mut xss = [[0.0; K - 1]; K - 1];
        let (mut a, mut b) = (0, 0);
        for y in 0..K {
            if y == i {
                continue;
            }
            for x in 0..K {
                if x == j {
                    continue;
                }
                xss[b][a] = self.0[y][x];
                a += 1;
            }
            a = 0;
            b += 1;
        }
        Matrix::new(xss)
    }
}

impl<const M: usize, const N: usize, const K: usize> ops::Mul<Matrix<N, K>> for Matrix<M, N> {
    type Output = Matrix<M, K>;

    fn mul(self, rhs: Matrix<N, K>) -> Self::Output {
        self.mult(&rhs)
    }
}

impl ops::Mul<Tuple> for Matrix<4, 4> {
    type Output = Tuple;

    fn mul(self, rhs: Tuple) -> Self::Output {
        self.mult_tuple(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix;
    use crate::assert_f64_eq;
    use crate::tuple::Tuple;

    #[test]
    fn can_create() {
        let xss = [
            [1.0, 2.0, 3.0, 4.0],
            [5.5, 6.5, 7.5, 8.5],
            [9.0, 10.0, 11.0, 12.0],
            [13.5, 14.5, 15.5, 16.5],
        ];
        let m = Matrix::new(xss);
        assert_eq!(m.get(0, 0), 1.0);
        assert_eq!(m.get(0, 3), 4.0);
        assert_eq!(m.get(1, 0), 5.5);
        assert_eq!(m.get(1, 2), 7.5);
        assert_eq!(m.get(2, 2), 11.0);
        assert_eq!(m.get(3, 0), 13.5);
        assert_eq!(m.get(3, 2), 15.5);

        let xss = [[-3.0, 5.0], [1.0, -2.0]];
        let m = Matrix::new(xss);
        assert_eq!(m.get(0, 0), -3.0);
        assert_eq!(m.get(0, 1), 5.0);
        assert_eq!(m.get(1, 0), 1.0);
        assert_eq!(m.get(1, 1), -2.0);
    }

    #[test]
    fn check_equality() {
        let xss = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ];
        let yss = [
            [0.0, 2.0, 3.0, 4.0],
            [0.0, 6.0, 7.0, 8.0],
            [0.0, 8.0, 7.0, 6.0],
            [0.0, 4.0, 3.0, 2.0],
        ];

        let a = Matrix::new(xss.clone());
        let b = Matrix::new(xss);
        assert_eq!(a, b);

        let b = Matrix::new(yss);
        assert_ne!(a, b);
    }

    #[test]
    fn multiplication() {
        let xss = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ];
        let yss = [
            [-2.0, 1.0, 2.0, 3.0],
            [3.0, 2.0, 1.0, -1.0],
            [4.0, 3.0, 6.0, 5.0],
            [1.0, 2.0, 7.0, 8.0],
        ];
        let a = Matrix::new(xss);
        let b = Matrix::new(yss);

        let zss = [
            [20.0, 22.0, 50.0, 48.0],
            [44.0, 54.0, 114.0, 108.0],
            [40.0, 58.0, 110.0, 102.0],
            [16.0, 26.0, 46.0, 42.0],
        ];

        let want = Matrix::new(zss);
        assert_eq!(a * b, want);
    }

    #[test]
    fn multiplication_with_tuple() {
        let xss = [
            [1.0, 2.0, 3.0, 4.0],
            [2.0, 4.0, 4.0, 2.0],
            [8.0, 6.0, 4.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let a = Matrix::new(xss);
        let b = Tuple(1.0, 2.0, 3.0, 1.0);
        assert_eq!(a * b, Tuple(18.0, 24.0, 33.0, 1.0));
    }

    #[test]
    fn multiplication_with_identity() {
        let xss = [
            [0.0, 1.0, 2.0, 4.0],
            [1.0, 2.0, 4.0, 8.0],
            [2.0, 4.0, 8.0, 16.0],
            [4.0, 8.0, 16.0, 32.0],
        ];
        let a = Matrix::new(xss);
        assert_eq!(a.clone() * Matrix::<4, 4>::ident(), a);
    }

    #[test]
    fn transpose() {
        let xss = [
            [0.0, 9.0, 3.0, 0.0],
            [9.0, 8.0, 0.0, 8.0],
            [1.0, 8.0, 5.0, 3.0],
            [0.0, 0.0, 5.0, 8.0],
        ];
        let yss = [
            [0.0, 9.0, 1.0, 0.0],
            [9.0, 8.0, 8.0, 0.0],
            [3.0, 0.0, 5.0, 5.0],
            [0.0, 8.0, 3.0, 8.0],
        ];
        let a = Matrix::new(xss);
        let want = Matrix::new(yss);
        assert_eq!(a.transpose(), want);
    }

    #[test]
    fn transpose_of_ident() {
        let a = Matrix::<4, 4>::ident();
        assert_eq!(a.transpose(), a);
    }

    #[test]
    fn determinant_2x2() {
        let xss = [[1.0, 5.0], [-3.0, 2.0]];
        let a = Matrix::new(xss);
        assert_eq!(a.det(), 17.0);
    }

    #[test]
    fn get_submatrix() {
        // 3x3
        let xss = [[1.0, 5.0, 0.0], [-3.0, 2.0, 7.0], [0.0, 6.0, -3.0]];
        let yss = [[-3.0, 2.0], [0.0, 6.0]];
        let a = Matrix::new(xss);
        let (got, want) = (a.submatrix(0, 2), Matrix::new(yss));
        assert_eq!(got, want);

        // 4x4
        let xss = [
            [-6.0, 1.0, 1.0, 6.0],
            [-8.0, 5.0, 8.0, 6.0],
            [-1.0, 0.0, 8.0, 2.0],
            [-7.0, 1.0, -1.0, 1.0],
        ];
        let yss = [[-6.0, 1.0, 6.0], [-8.0, 8.0, 6.0], [-7.0, -1.0, 1.0]];
        let a = Matrix::new(xss);
        let (got, want) = (a.submatrix(2, 1), Matrix::new(yss));
        assert_eq!(got, want);
    }

    #[test]
    fn get_minor() {
        let xss = [[3.0, 5.0, 0.0], [2.0, -1.0, -7.0], [6.0, -1.0, 5.0]];
        let a = Matrix::new(xss);
        let b = a.submatrix(1, 0);
        assert_eq!(b.det(), 25.0);
        assert_eq!(a.minor(1, 0), 25.0);
    }

    #[test]
    fn get_cofactor() {
        let xss = [[3.0, 5.0, 0.0], [2.0, -1.0, -7.0], [6.0, -1.0, 5.0]];
        let a = Matrix::new(xss);

        assert_eq!(a.minor(0, 0), -12.0);
        assert_eq!(a.cofactor(0, 0), -12.0);
        assert_eq!(a.minor(1, 0), 25.0);
        assert_eq!(a.cofactor(1, 0), -25.0);
    }

    #[test]
    fn determinant_3x3() {
        let xss = [[1.0, 2.0, 6.0], [-5.0, 8.0, -4.0], [2.0, 6.0, 4.0]];
        let a = Matrix::new(xss);
        assert_eq!(a.cofactor(0, 0), 56.0);
        assert_eq!(a.cofactor(0, 1), 12.0);
        assert_eq!(a.cofactor(0, 2), -46.0);
        assert_eq!(a.det(), -196.0);
    }

    #[test]
    fn determinant_4x4() {
        let xss = [
            [-2.0, -8.0, 3.0, 5.0],
            [-3.0, 1.0, 7.0, 3.0],
            [1.0, 2.0, -9.0, 6.0],
            [-6.0, 7.0, 7.0, -9.0],
        ];
        let a = Matrix::new(xss);
        assert_eq!(a.cofactor(0, 0), 690.0);
        assert_eq!(a.cofactor(0, 1), 447.0);
        assert_eq!(a.cofactor(0, 2), 210.0);
        assert_eq!(a.cofactor(0, 3), 51.0);
        assert_eq!(a.det(), -4071.0);
    }

    #[test]
    fn invertibility() {
        let xss = [
            [6.0, 4.0, 4.0, 4.0],
            [5.0, 5.0, 7.0, 6.0],
            [4.0, -9.0, 3.0, -7.0],
            [9.0, 1.0, 7.0, -6.0],
        ];
        let a = Matrix::new(xss);
        assert_eq!(a.det(), -2120.0);
        assert!(a.is_invertible());

        let xss = [
            [-4.0, 2.0, -2.0, -3.0],
            [9.0, 6.0, 2.0, 6.0],
            [0.0, -5.0, 1.0, -5.0],
            [0.0, 0.0, 0.0, 0.0],
        ];
        let a = Matrix::new(xss);
        assert_eq!(a.det(), 0.0);
        assert!(!a.is_invertible());
    }

    #[test]
    fn inverse_of_4x4() {
        let tests = vec![
            (
                [
                    [-5.0, 2.0, 6.0, -8.0],
                    [1.0, -5.0, 1.0, 8.0],
                    [7.0, 7.0, -6.0, -7.0],
                    [1.0, -3.0, 7.0, 4.0],
                ],
                [
                    [0.21805, 0.45113, 0.24060, -0.04511],
                    [-0.80827, -1.45677, -0.44361, 0.52068],
                    [-0.07895, -0.22368, -0.05263, 0.19737],
                    [-0.52256, -0.81391, -0.30075, 0.30639],
                ],
            ),
            (
                [
                    [8.0, -5.0, 9.0, 2.0],
                    [7.0, 5.0, 6.0, 1.0],
                    [-6.0, 0.0, 9.0, 6.0],
                    [-3.0, 0.0, -9.0, -4.0],
                ],
                [
                    [-0.15385, -0.15385, -0.28205, -0.53846],
                    [-0.07692, 0.12308, 0.02564, 0.03077],
                    [0.35897, 0.35897, 0.43590, 0.92308],
                    [-0.69231, -0.69231, -0.76923, -1.92308],
                ],
            ),
            (
                [
                    [9.0, 3.0, 0.0, 9.0],
                    [-5.0, -2.0, -6.0, -3.0],
                    [-4.0, 9.0, 6.0, 4.0],
                    [-7.0, 6.0, 6.0, 2.0],
                ],
                [
                    [-0.04074, -0.07778, 0.14444, -0.22222],
                    [-0.07778, 0.03333, 0.36667, -0.33333],
                    [-0.02901, -0.14630, -0.10926, 0.12963],
                    [0.17778, 0.06667, -0.26667, 0.33333],
                ],
            ),
        ];

        for test in tests {
            let a = Matrix::new(test.0);
            assert!(a.is_invertible());
            let b = a.inverse().unwrap();
            let want = Matrix::new(test.1);
            for i in 0..4 {
                for j in 0..4 {
                    assert_f64_eq!(b.get(i, j), want.get(i, j), 0.00001);
                }
            }
        }
    }

    #[test]
    fn multiply_product_by_inverse() {
        let xss = [
            [3.0, -9.0, 7.0, 3.0],
            [3.0, -8.0, 2.0, -9.0],
            [-4.0, 4.0, 4.0, 1.0],
            [-6.0, 5.0, -1.0, 1.0],
        ];
        let yss = [
            [8.0, 2.0, 2.0, 2.0],
            [3.0, -1.0, 7.0, 0.0],
            [7.0, 0.0, 5.0, 4.0],
            [6.0, 2.0, 0.0, 5.0],
        ];
        let a = Matrix::new(xss);
        let b = Matrix::new(yss);
        let c = a * b;
        assert_eq!(c * b.inverse().unwrap(), a);
    }
}
