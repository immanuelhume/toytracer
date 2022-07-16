use std::{env, fs::write, path};
use toytracer::{
    canvas::Canvas, color::Color, pad_filepath, ray::Ray, ray::Sphere,
    transformation::Transformation, tuple::Point,
};

/// Number of pixels along one side of the square canvas.
const CANVAS_PIXELS: usize = 100;
/// The z-coordinate of the wall plane.
const WALL_Z: f64 = 10.0;
/// Virtual size of one side of the wall.
const WALL_SIZE: f64 = 7.0;
/// Half the wall size, which represents the maximum x and y values for any point on the wall.
const HALF_WALL: f64 = WALL_SIZE / 2.0;
/// The size of each "virtual pixel" relative to the actual pixels on the canvas.
const PIXEL_SIZE: f64 = WALL_SIZE / CANVAS_PIXELS as f64;

fn main() {
    let filepath = env::args().nth(1).unwrap();
    let filepath = pad_filepath(&filepath);

    println!("will be writing file to {}", filepath);

    let mut canvas = Canvas::new(CANVAS_PIXELS, CANVAS_PIXELS);
    let ray_origin = Point::new(0.0, 0.0, -5.0);
    let red = Color::new(1.0, 0.0, 0.0);
    let mut s = Sphere::default();
    s.set_transform(
        Transformation::default()
            .scale(1.0, 0.5, 1.0)
            .shear(1.0, 0.0, 0.5, 0.0, 0.0, 0.0)
            .into(),
    );

    for i in 0..CANVAS_PIXELS {
        for j in 0..CANVAS_PIXELS {
            let ray_end = Point::new(
                j as f64 * PIXEL_SIZE - HALF_WALL,
                i as f64 * PIXEL_SIZE - HALF_WALL,
                WALL_Z,
            );
            let ray = Ray::new(ray_origin, ray_end - ray_origin);
            match ray.when_intersect_sphere(&s) {
                Some(_) => canvas.draw(i, j, red),
                _ => (),
            }
        }
    }

    write(filepath, canvas.to_ppm().as_bytes()).unwrap();
}
