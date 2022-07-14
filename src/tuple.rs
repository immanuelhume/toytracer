use std::ops;

#[derive(Debug, Clone, Copy, Default)]
pub struct Tuple(pub f64, pub f64, pub f64, pub f64);

impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < crate::EPSILON
            && (self.1 - other.1).abs() < crate::EPSILON
            && (self.2 - other.2).abs() < crate::EPSILON
            && (self.3 - other.3).abs() < crate::EPSILON
    }
}

impl Tuple {
    pub fn is_point(&self) -> bool {
        self.3 == 1.0
    }

    pub fn is_vector(&self) -> bool {
        self.3 == 0.0
    }
}

impl ops::Add<Tuple> for Tuple {
    type Output = Tuple;

    fn add(self, other: Tuple) -> Self::Output {
        Tuple(
            self.0 + other.0,
            self.1 + other.1,
            self.2 + other.2,
            self.3 + other.3,
        )
    }
}

impl ops::Sub<Tuple> for Tuple {
    type Output = Tuple;

    fn sub(self, other: Tuple) -> Self::Output {
        Tuple(
            self.0 - other.0,
            self.1 - other.1,
            self.2 - other.2,
            self.3 - other.3,
        )
    }
}

impl ops::Neg for Tuple {
    type Output = Tuple;

    fn neg(self) -> Self::Output {
        Tuple(0.0 - self.0, 0.0 - self.1, 0.0 - self.2, 0.0 - self.3)
    }
}

impl ops::Mul<f64> for Tuple {
    type Output = Tuple;

    fn mul(self, rhs: f64) -> Self::Output {
        Tuple(self.0 * rhs, self.1 * rhs, self.2 * rhs, self.3 * rhs)
    }
}

impl ops::Mul<Tuple> for f64 {
    type Output = Tuple;

    fn mul(self, rhs: Tuple) -> Self::Output {
        Tuple(self * rhs.0, self * rhs.1, self * rhs.2, self * rhs.3)
    }
}

impl ops::Div<f64> for Tuple {
    type Output = Tuple;

    fn div(self, rhs: f64) -> Self::Output {
        Tuple(self.0 / rhs, self.1 / rhs, self.2 / rhs, self.3 / rhs)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Point(Tuple);

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(Tuple(x, y, z, 1.0))
    }

    pub fn x(&self) -> f64 {
        self.0 .0
    }

    pub fn y(&self) -> f64 {
        self.0 .1
    }

    pub fn z(&self) -> f64 {
        self.0 .2
    }

    pub fn inner(&self) -> Tuple {
        self.0
    }
}

impl ops::Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Self::Output {
        Vector(self.0 - rhs.0)
    }
}

impl ops::Add<Vector> for Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Self::Output {
        Point(self.0 + rhs.0)
    }
}

impl ops::Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, rhs: Vector) -> Self::Output {
        Point(self.0 - rhs.0)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vector(Tuple);

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(Tuple(x, y, z, 0.0))
    }

    pub fn magnitude(&self) -> f64 {
        let Tuple(x, y, z, ..) = self.0;
        x.hypot(y).hypot(z)
    }

    pub fn normalize(&self) -> Vector {
        let mag = self.magnitude();
        Vector(self.0 / mag)
    }

    pub fn dot(&self, v: Vector) -> f64 {
        let Tuple(x, y, z, ..) = self.0;
        let Tuple(a, b, c, ..) = v.0;
        x * a + y * b + z * c
    }

    pub fn cross(&self, v: Vector) -> Vector {
        let Tuple(x, y, z, ..) = self.0;
        let Tuple(a, b, c, ..) = v.0;
        Vector::new(y * c - z * b, z * a - x * c, x * b - y * a)
    }

    pub fn inner(&self) -> Tuple {
        self.0
    }
}

impl ops::Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        Vector(self.0 - rhs.0)
    }
}

impl ops::Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector(self.0 + rhs.0)
    }
}

impl ops::Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        let Tuple(x, y, z, ..) = self.0;
        Vector::new(x * rhs, y * rhs, z * rhs)
    }
}

impl ops::Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        let Tuple(x, y, z, ..) = rhs.0;
        Vector::new(x * self, y * self, z * self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Point, Tuple, Vector};
    use crate::assert_f64_eq;

    #[test]
    fn is_a_point() {
        let a = Tuple(4.3, -4.2, 3.1, 1.0);
        assert!(a.is_point());
        assert!(!a.is_vector());
    }

    #[test]
    fn is_a_vector() {
        let a = Tuple(4.3, -4.2, 3.1, 0.0);
        assert!(a.is_vector());
        assert!(!a.is_point());
    }

    #[test]
    fn create_point() {
        let p = Point::new(4.0, -4.0, 3.0);
        assert_eq!(p.0, Tuple(4.0, -4.0, 3.0, 1.0))
    }

    #[test]
    fn create_vector() {
        let p = Vector::new(4.0, -4.0, 3.0);
        assert_eq!(p.0, Tuple(4.0, -4.0, 3.0, 0.0))
    }

    #[test]
    fn adding_tuples() {
        let a1 = Tuple(3.0, -2.0, 5.0, 1.0);
        let a2 = Tuple(-2.0, 3.0, 1.0, 0.0);
        assert_eq!(a1 + a2, Tuple(1.0, 1.0, 6.0, 1.0));
    }

    #[test]
    fn subtract_two_points() {
        let p1 = Point::new(3.0, 2.0, 1.0);
        let p2 = Point::new(5.0, 6.0, 7.0);
        assert_eq!(p1 - p2, Vector::new(-2.0, -4.0, -6.0))
    }

    #[test]
    fn subtract_vector_from_point() {
        let p = Point::new(3.0, 2.0, 1.0);
        let v = Vector::new(5.0, 6.0, 7.0);
        assert_eq!(p - v, Point::new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn subtract_two_vectors() {
        let v1 = Vector::new(3.0, 2.0, 1.0);
        let v2 = Vector::new(5.0, 6.0, 7.0);
        assert_eq!(v1 - v2, Vector::new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn subtract_zero_vector_from_vector() {
        let zero = Vector::new(0.0, 0.0, 0.0);
        let v = Vector::new(-1.0, 2.0, -3.0);
        assert_eq!(zero - v, Vector::new(1.0, -2.0, 3.0));
    }

    #[test]
    fn negate_tuple() {
        let a = Tuple(1.0, -2.0, 3.0, -4.0);
        assert_eq!(-a, Tuple(-1.0, 2.0, -3.0, 4.0));
    }

    #[test]
    fn mul_tuple_by_scalar() {
        let a = Tuple(1.0, -2.0, 3.0, -4.0);
        assert_eq!(a * 3.5, Tuple(3.5, -7.0, 10.5, -14.0));
    }

    #[test]
    fn mul_tuple_by_fraction() {
        let a = Tuple(1.0, -2.0, 3.0, -4.0);
        assert_eq!(a * 0.5, Tuple(0.5, -1.0, 1.5, -2.0));
    }

    #[test]
    fn div_tuple_by_scalar() {
        let a = Tuple(1.0, -2.0, 3.0, -4.0);
        assert_eq!(a / 2.0, Tuple(0.5, -1.0, 1.5, -2.0));
    }

    #[test]
    fn magnitude() {
        let tests = [
            (Vector::new(1.0, 0.0, 0.0), 1.0),
            (Vector::new(0.0, 1.0, 0.0), 1.0),
            (Vector::new(0.0, 0.0, 1.0), 1.0),
            (Vector::new(1.0, 2.0, 3.0), 14.0_f64.sqrt()),
            (Vector::new(-1.0, -2.0, -3.0), 14.0_f64.sqrt()),
        ];
        for (v, want) in tests {
            assert_f64_eq!(v.magnitude(), want);
        }
    }

    #[test]
    fn normalization() {
        let tests = [
            (Vector::new(4.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0)),
            (
                Vector::new(1.0, 2.0, 3.0),
                Vector::new(
                    1.0 / 14.0_f64.sqrt(),
                    2.0 / 14.0_f64.sqrt(),
                    3.0 / 14.0_f64.sqrt(),
                ),
            ),
        ];
        for (v1, v2) in tests {
            assert_eq!(v1.normalize(), v2);
            assert_eq!(v1.normalize().magnitude(), 1.0);
        }
    }

    #[test]
    fn dot_product() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(2.0, 3.0, 4.0);
        assert_eq!(a.dot(b), 20.0);
        assert_eq!(b.dot(a), 20.0);
    }

    #[test]
    fn cross_product() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(2.0, 3.0, 4.0);
        assert_eq!(a.cross(b), Vector::new(-1.0, 2.0, -1.0));
        assert_eq!(b.cross(a), Vector::new(1.0, -2.0, 1.0));
    }
}
