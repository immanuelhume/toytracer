// Draws the 12 points in a clock.

use std::env;
use std::fs::write;
use toytracer::canvas::Canvas;
use toytracer::color::Color;
use toytracer::pad_filepath;
use toytracer::transformation::{rotation_x, translation};
use toytracer::tuple::Point;

const N: usize = 256;
const L: f64 = N as f64 * 3_f64 / 8_f64;

fn main() {
    let filepath = env::args().nth(1).unwrap_or("./tmp/clock.ppm".to_string());
    let filepath = pad_filepath(&filepath);

    println!("writing output to {}", filepath);

    // Create an N by N canvas.
    let mut canvas = Canvas::new(N, N);
    let mut p = Point::new(0.0, L, 0.0);
    let to_center = translation(0.0, (N / 2) as f64, (N / 2) as f64);
    let tick = rotation_x(std::f64::consts::PI / 6.0);
    for _ in 0..12 {
        let dot = to_center * p;
        canvas.draw(
            dot.y() as usize,
            dot.z() as usize,
            Color::new(1.0, 1.0, 1.0),
        );
        p = tick * p;
    }

    write(filepath, canvas.to_ppm().as_bytes()).unwrap();
}
