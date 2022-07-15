use crate::matrix::Matrix;
use crate::tuple::{Point, Tuple, Vector};
use std::ops;

pub fn translation(x: f64, y: f64, z: f64) -> Matrix<4, 4> {
    let xss = [
        [1.0, 0.0, 0.0, x],
        [0.0, 1.0, 0.0, y],
        [0.0, 0.0, 1.0, z],
        [0.0, 0.0, 0.0, 1.0],
    ];
    Matrix::new(xss)
}

pub fn scaling(x: f64, y: f64, z: f64) -> Matrix<4, 4> {
    let xss = [
        [x, 0.0, 0.0, 0.0],
        [0.0, y, 0.0, 0.0],
        [0.0, 0.0, z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    Matrix::new(xss)
}

/// A clockwise rotation about the x axis.
pub fn rotation_x(rad: f64) -> Matrix<4, 4> {
    let xss = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, rad.cos(), -rad.sin(), 0.0],
        [0.0, rad.sin(), rad.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    Matrix::new(xss)
}

/// A clockwise rotation about the y axis.
pub fn rotation_y(rad: f64) -> Matrix<4, 4> {
    let xss = [
        [rad.cos(), 0.0, rad.sin(), 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-rad.sin(), 0.0, rad.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    Matrix::new(xss)
}

/// A clockwise rotation about the z axis.
pub fn rotation_z(rad: f64) -> Matrix<4, 4> {
    let xss = [
        [rad.cos(), -rad.sin(), 0.0, 0.0],
        [rad.sin(), rad.cos(), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    Matrix::new(xss)
}

pub fn shearing(xy: f64, xz: f64, yx: f64, yz: f64, zx: f64, zy: f64) -> Matrix<4, 4> {
    let xss = [
        [1.0, xy, xz, 0.0],
        [yx, 1.0, yz, 0.0],
        [zx, zy, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    Matrix::new(xss)
}

impl ops::Mul<Point> for Matrix<4, 4> {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        let Tuple(x, y, z, ..) = self * rhs.inner();
        Point::new(x, y, z)
    }
}

impl ops::Mul<Vector> for Matrix<4, 4> {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        let Tuple(x, y, z, ..) = self * rhs.inner();
        Vector::new(x, y, z)
    }
}

struct Transformation(Matrix<4, 4>);

impl Default for Transformation {
    fn default() -> Self {
        Self(Matrix::<4, 4>::ident())
    }
}

impl Transformation {
    pub fn identity(self) -> Self {
        self
    }

    pub fn translate(self, x: f64, y: f64, z: f64) -> Self {
        Self(translation(x, y, z) * self.0)
    }

    pub fn scale(self, x: f64, y: f64, z: f64) -> Self {
        Self(scaling(x, y, z) * self.0)
    }

    pub fn rotate_x(self, rad: f64) -> Self {
        Self(rotation_x(rad) * self.0)
    }

    pub fn rotate_y(self, rad: f64) -> Self {
        Self(rotation_y(rad) * self.0)
    }

    pub fn rotate_z(self, rad: f64) -> Self {
        Self(rotation_y(rad) * self.0)
    }

    pub fn shear(self, xy: f64, xz: f64, yx: f64, yz: f64, zx: f64, zy: f64) -> Self {
        Self(shearing(xy, xz, yx, yz, zx, zy) * self.0)
    }
}

impl Into<Matrix<4, 4>> for Transformation {
    fn into(self) -> Matrix<4, 4> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::{
        rotation_x, rotation_y, rotation_z, scaling, shearing, translation, Transformation,
    };
    use crate::matrix::Matrix;
    use crate::tuple::{Point, Vector};

    #[test]
    fn multiply_by_translation_matrix() {
        let transform = translation(5.0, -3.0, 2.0);
        let p = Point::new(-3.0, 4.0, 5.0);

        let want = Point::new(2.0, 1.0, 7.0);
        let got = transform * p;
        assert_eq!(want, got);
    }

    #[test]
    fn multiply_by_translation_matrix_inverse() {
        let transform = translation(5.0, -3.0, 2.0);
        let inv = transform.inverse().unwrap();
        let p = Point::new(-3.0, 4.0, 5.0);

        let want = Point::new(-8.0, 7.0, 3.0);
        let got = inv * p;
        assert_eq!(want, got);
    }

    #[test]
    fn translating_vector_no_change() {
        let transform = translation(5.0, -3.0, 2.0);
        let v = Vector::new(-3.0, 4.0, 5.0);

        let want = v;
        let got = transform * v;
        assert_eq!(want, got);
    }

    #[test]
    fn scaling_a_point() {
        let transform = scaling(2.0, 3.0, 4.0);
        let p = Point::new(-4.0, 6.0, 8.0);

        let want = Point::new(-8.0, 18.0, 32.0);
        let got = transform * p;
        assert_eq!(want, got);
    }

    #[test]
    fn scaling_a_vector() {
        let transform = scaling(2.0, 3.0, 4.0);
        let v = Vector::new(-4.0, 6.0, 8.0);

        let want = Vector::new(-8.0, 18.0, 32.0);
        let got = transform * v;
        assert_eq!(want, got);
    }

    #[test]
    fn multiply_by_inverse_of_scale() {
        let transform = scaling(2.0, 3.0, 4.0);
        let inv = transform.inverse().unwrap();
        let v = Vector::new(-4.0, 6.0, 8.0);

        let want = Vector::new(-2.0, 2.0, 2.0);
        let got = inv * v;
        assert_eq!(want, got);
    }

    #[test]
    fn reflection_is_scaling_by_negative() {
        let transform = scaling(-1.0, 1.0, 1.0);
        let p = Point::new(2.0, 3.0, 4.0);

        let want = Point::new(-2.0, 3.0, 4.0);
        let got = transform * p;
        assert_eq!(want, got);
    }

    #[test]
    fn rotating_about_x_axis() {
        let p = Point::new(0.0, 1.0, 0.0);
        let half_quarter = rotation_x(PI / 4.0);
        let full_quarter = rotation_x(PI / 2.0);

        let want = Point::new(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0);
        let got = half_quarter * p;
        assert_eq!(want, got);

        let want = Point::new(0.0, 0.0, 1.0);
        let got = full_quarter * p;
        assert_eq!(want, got);
    }

    #[test]
    fn inverse_rotate_about_x_axis() {
        let p = Point::new(0.0, 1.0, 0.0);
        let half_quarter = rotation_x(PI / 4.0);
        let inv = half_quarter.inverse().unwrap();

        let want = Point::new(0.0, 2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0);
        let got = inv * p;
        assert_eq!(want, got);
    }

    #[test]
    fn rotating_about_y_axis() {
        let p = Point::new(0.0, 0.0, 1.0);
        let half_quarter = rotation_y(PI / 4.0);
        let full_quarter = rotation_y(PI / 2.0);

        let want = Point::new(2.0_f64.sqrt() / 2.0, 0.0, 2.0_f64.sqrt() / 2.0);
        let got = half_quarter * p;
        assert_eq!(want, got);

        let want = Point::new(1.0, 0.0, 0.0);
        let got = full_quarter * p;
        assert_eq!(want, got);
    }

    #[test]
    fn rotating_about_z_axis() {
        let p = Point::new(0.0, 1.0, 0.0);
        let half_quarter = rotation_z(PI / 4.0);
        let full_quarter = rotation_z(PI / 2.0);

        let want = Point::new(-2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0.0);
        let got = half_quarter * p;
        assert_eq!(want, got);

        let want = Point::new(-1.0, 0.0, 0.0);
        let got = full_quarter * p;
        assert_eq!(want, got);
    }

    #[test]
    fn shearing_x_prop_y() {
        let p = Point::new(2.0, 3.0, 4.0);
        let tests = vec![
            (
                shearing(1.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                Point::new(5.0, 3.0, 4.0),
            ),
            (
                shearing(0.0, 1.0, 0.0, 0.0, 0.0, 0.0),
                Point::new(6.0, 3.0, 4.0),
            ),
            (
                shearing(0.0, 0.0, 1.0, 0.0, 0.0, 0.0),
                Point::new(2.0, 5.0, 4.0),
            ),
            (
                shearing(0.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                Point::new(2.0, 7.0, 4.0),
            ),
            (
                shearing(0.0, 0.0, 0.0, 0.0, 1.0, 0.0),
                Point::new(2.0, 3.0, 6.0),
            ),
            (
                shearing(0.0, 0.0, 0.0, 0.0, 0.0, 1.0),
                Point::new(2.0, 3.0, 7.0),
            ),
        ];

        for test in tests {
            let want = test.1;
            let got = test.0 * p;
            assert_eq!(want, got);
        }
    }

    #[test]
    fn transformations_in_sequence() {
        let p = Point::new(1.0, 0.0, 1.0);
        let a = rotation_x(PI / 2.0);
        let b = scaling(5.0, 5.0, 5.0);
        let c = translation(10.0, 5.0, 7.0);

        let p2 = a * p;
        let want = Point::new(1.0, -1.0, 0.0);
        assert_eq!(p2, want);

        let p3 = b * p2;
        let want = Point::new(5.0, -5.0, 0.0);
        assert_eq!(p3, want);

        let p4 = c * p3;
        let want = Point::new(15.0, 0.0, 7.0);
        assert_eq!(p4, want);
    }

    #[test]
    fn chained_transformations() {
        let p = Point::new(1.0, 0.0, 1.0);

        let a = rotation_x(PI / 2.0);
        let b = scaling(5.0, 5.0, 5.0);
        let c = translation(10.0, 5.0, 7.0);

        let got = (c * b * a) * p;
        let want = Point::new(15.0, 0.0, 7.0);
        assert_eq!(got, want);

        let transform: Matrix<4, 4> = Transformation::default()
            .rotate_x(PI / 2.0)
            .scale(5.0, 5.0, 5.0)
            .translate(10.0, 5.0, 7.0)
            .into();
        let got = transform * p;
        assert_eq!(got, want);
    }
}
