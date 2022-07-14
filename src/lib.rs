#![feature(generic_const_exprs)]

pub mod canvas;
pub mod color;
pub mod matrix;
pub mod transformation;
pub mod tuple;

/// A slightly more lenient epsilon.
const EPSILON: f64 = 100.0 * std::f64::EPSILON;

macro_rules! assert_f64_eq {
    ($a:expr, $b:expr) => {
        assert!(($a - $b).abs() < 100.0 * std::f64::EPSILON);
    };

    ($a:expr, $b:expr, $eps:expr) => {
        assert!(($a - $b).abs() < $eps);
    };
}

pub(crate) use assert_f64_eq;
