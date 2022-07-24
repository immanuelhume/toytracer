// Draws a scene with three balls.

use std::env;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4};
use std::fs::write;
use toytracer::camera::Camera;
use toytracer::color::Color;
use toytracer::light::Material;
use toytracer::light::PointLight;
use toytracer::sphere::Sphere;
use toytracer::transform::{view_transform, Tr};
use toytracer::tuple::{Point, Vector};
use toytracer::world::World;
use toytracer::{file_exists, pad_filepath};

const WIDTH: usize = 1024;
const HEIGHT: usize = 512;
const FOV: f64 = FRAC_PI_3;

fn main() {
    let filepath = env::args().nth(1).unwrap_or("./tmp/scene.ppm".to_string());
    let filepath = pad_filepath(&filepath, file_exists);

    println!("output will be written to {}", filepath);

    // Make the floor.
    let floor = Sphere::default()
        .with_transform(Tr::default().scale(10.0, 0.01, 10.0))
        .with_material(
            Material::default()
                .with_color(Color::new(1.0, 0.9, 0.9))
                .with_specular(0.0),
        );

    // Make the left wall.
    let left_wall = Sphere::default()
        .with_transform(
            Tr::default()
                .scale(10.0, 0.01, 10.0)
                .rotate_x(FRAC_PI_2)
                .rotate_y(-FRAC_PI_4)
                .translate(0.0, 0.0, 5.0),
        )
        .with_material(floor.material());

    // Make the right wall.
    let right_wall = Sphere::default()
        .with_transform(
            Tr::default()
                .scale(10.0, 0.01, 10.0)
                .rotate_x(FRAC_PI_2)
                .rotate_y(FRAC_PI_4)
                .translate(0.0, 0.0, 5.0),
        )
        .with_material(floor.material());

    // A chonky green sphere in the middle.
    let middle = Sphere::default()
        .with_transform(Tr::default().translate(-0.5, 1.0, 0.5))
        .with_material(
            Material::default()
                .with_color(Color::new(0.1, 1.0, 0.5))
                .with_diffuse(0.7)
                .with_specular(0.3),
        );

    let right = Sphere::default()
        .with_transform(Tr::default().scale(0.5, 0.5, 0.5).translate(1.5, 0.5, -0.5))
        .with_material(
            Material::default()
                .with_color(Color::new(0.5, 1.0, 0.1))
                .with_diffuse(0.7)
                .with_specular(0.3),
        );

    let left = Sphere::default()
        .with_transform(
            Tr::default()
                .scale(0.33, 0.33, 0.33)
                .translate(-1.5, 0.33, -0.75),
        )
        .with_material(
            Material::default()
                .with_color(Color::new(1.0, 0.8, 0.1))
                .with_diffuse(0.7)
                .with_specular(0.3),
        );

    // Set up the world with our objects.
    let light = PointLight::new(Point::new(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
    let world = World::new()
        .with_light(light)
        .with_objects(vec![floor, left_wall, right_wall, middle, right, left]);

    // And finally the camera.
    let camera = Camera::new(WIDTH, HEIGHT, FOV).with_transform(view_transform(
        Point::new(0.0, 1.5, -5.0),
        Point::new(0.0, 1.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(world);

    write(filepath, canvas.to_ppm().as_bytes()).unwrap();
}
