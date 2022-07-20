#![feature(generic_const_exprs)]
use std::{
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

pub mod canvas;
pub mod color;
pub mod light;
pub mod matrix;
pub mod ray;
pub mod transformation;
pub mod tuple;
pub mod world;

/// A slightly more lenient epsilon.
const EPSILON: f64 = 0.00001;

macro_rules! assert_f64_eq {
    ($a:expr, $b:expr) => {
        assert!(($a - $b).abs() < 100.0 * std::f64::EPSILON);
    };

    ($a:expr, $b:expr, $eps:expr) => {
        assert!(($a - $b).abs() < $eps);
    };
}

pub(crate) use assert_f64_eq;

/// Represents a globally unique ID within the lifetime of the program.
static UID: AtomicUsize = AtomicUsize::new(0);
/// Retrieves a globally unique ID within the lifetime of the program.
pub fn get_uid() -> usize {
    UID.fetch_add(1, Ordering::SeqCst)
}

/// Given a filepath, adds shit to it such that the path does not currently exist.
pub fn pad_filepath(s: &str) -> String {
    let mut x = 1;
    let mut res = String::from(s);
    while Path::new(&res).exists() {
        res = format!("{} ({})", s, x);
        x += 1;
    }
    res
}
