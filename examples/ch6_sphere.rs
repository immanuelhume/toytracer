// Draws a sphere.

use std::env;
use std::fs::write;
use toytracer::canvas::Canvas;
use toytracer::color::Color;
use toytracer::light::{lighting, Material, PointLight};
use toytracer::ray::{hit, Ray};
use toytracer::shapes::{Shape, Sphere};
use toytracer::transform::Tr;
use toytracer::tuple::Point;
use toytracer::{file_exists, pad_filepath};

/// Number of pixels along one side of the square canvas. This only affects the "resolution" of the
/// image.
const CANVAS_PIXELS: usize = 256;
/// The z-coordinate of the wall plane.
const WALL_Z: f64 = 5.0;
/// Virtual size of one side of the wall.
const WALL_SIZE: f64 = 10.0;
/// Half the wall size, which represents the maximum x and y values for any point on the wall.
const HALF_WALL: f64 = WALL_SIZE / 2.0;
/// The size of each "virtual pixel" relative to the actual pixels on the canvas.
const PIXEL_SIZE: f64 = WALL_SIZE / CANVAS_PIXELS as f64;

fn main() {
    let filepath = env::args().nth(1).unwrap_or("./tmp/sphere.ppm".to_string());
    let filepath = pad_filepath(&filepath, file_exists);

    println!("will be writing file to {}", filepath);

    let mut canvas = Canvas::new(CANVAS_PIXELS, CANVAS_PIXELS);
    let ray_origin = Point::new(0.0, 0.0, -5.0); // where the eyeball is

    // Create a sphere at the origin with a radius of 1.0. Also give it some material.
    let s = Sphere::default()
        .with_transform(Tr::default().scale(1.0, 1.0, 1.0).into())
        .with_material(
            Material::default()
                .with_color(Color::new(1.0, 0.2, 1.0))
                .with_shininess(100.0),
        );

    // Create a light source.
    let light_position = Point::new(-10.0, 10.0, -10.0);
    let light_color = Color::new(1.0, 1.0, 1.0);
    let light = PointLight::new(light_position, light_color);

    // Now for each pixel on the canvas, cast a ray from the eye to that pixel. If the ray hits the
    // sphere at some point, we'll compute the appropriate lighting at that point and draw it onto
    // the canvas.
    for i in 0..CANVAS_PIXELS {
        let y = HALF_WALL - i as f64 * PIXEL_SIZE;
        for j in 0..CANVAS_PIXELS {
            let x = j as f64 * PIXEL_SIZE - HALF_WALL;
            let ray_end = Point::new(x, y, WALL_Z);

            let ray = Ray::new(ray_origin, (ray_end - ray_origin).normalize());
            let eyev = -ray.direction();

            match hit(s.intersect_with(ray)) {
                Some(h) => {
                    let p = ray.position_at(h.t());
                    let normalv = h.object().normal_at(p);

                    let color = lighting(h.material(), light, p, eyev, normalv, false);
                    canvas.write_to(j, i, color);
                }
                _ => (),
            }
        }
    }

    write(&filepath, canvas.to_ppm().as_bytes()).unwrap();
    println!("wrote file to {}", filepath);
}
