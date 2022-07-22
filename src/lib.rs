#![feature(generic_const_exprs)]
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod camera;
pub mod canvas;
pub mod color;
pub mod light;
pub mod matrix;
pub mod ray;
pub mod transformation;
pub mod tuple;
pub mod world;

/// A much more lenient epsilon for convenience.
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

pub fn file_exists(filename: &str) -> bool {
    Path::new(filename).exists()
}

/// Given a filepath, adds shit to it such that the path does not currently exist.
pub fn pad_filepath<T>(s: &str, exists_fn: T) -> String
where
    T: Fn(&str) -> bool,
{
    let path = Path::new(s);
    let mut par = match path.parent() {
        None => "".to_string(),
        Some(p) => p.to_str().unwrap().to_string(),
    };
    if par.len() > 0 {
        par.push('/');
    }
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let ext = path.extension().unwrap().to_str().unwrap();

    let mut x = 1;
    let mut res = String::from(s);
    while exists_fn(&res) {
        res = format!("{}{} ({}).{}", par, stem, x, ext);
        x += 1;
    }
    res
}

#[test]
fn test_pad_filepath() {
    let exists_fn = |filename: &str| match filename {
        "foo.ppm" | "foo (1).ppm" | "foo (2).ppm" => true,
        _ => false,
    };

    let got = pad_filepath("foo.ppm", exists_fn);
    let want = "foo (3).ppm";
    assert_eq!(got, want);

    let exists_fn = |filename: &str| match filename {
        "foo/bar.ppm" | "foo/bar (1).ppm" => true,
        _ => false,
    };

    let got = pad_filepath("foo/bar.ppm", exists_fn);
    let want = "foo/bar (2).ppm";
    assert_eq!(got, want);
}
