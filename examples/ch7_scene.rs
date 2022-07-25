// Draws a scene with three balls.

use std::env;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3};
use std::fs::write;
use toytracer::camera::Camera;
use toytracer::color::Color;
use toytracer::light::Material;
use toytracer::light::PointLight;
use toytracer::shapes::Sphere;
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

    let floor = Sphere::default()
        .with_transform(Tr::default().scale(100.0, 0.01, 100.0))
        .with_material(
            Material::default()
                .with_specular(0.0)
                .with_color(Color::new(0.5, 0.5, 0.5)),
        )
        .as_object();
    let wall = Sphere::default()
        .with_transform(
            Tr::default()
                .scale(100.0, 0.01, 100.0)
                .rotate_z(FRAC_PI_2)
                .translate(-20.0, 0.0, 0.0),
        )
        .with_material(floor.material())
        .as_object();
    let s1 = Sphere::default()
        .with_transform(
            Tr::default()
                .translate(-2.0, 1.0, -0.5)
                .scale(3.0, 3.0, 3.0),
        )
        .as_object();
    let s2 = Sphere::default()
        .with_transform(Tr::default().translate(2.0, 1.0, 0.0))
        .with_material(Material::default().with_color(Color::new(1.0, 0.5, 1.0)))
        .as_object();

    let world = World::new()
        .with_light(PointLight::new(
            Point::new(18.0, 15.0, 10.0),
            Color::white(),
        ))
        .add_objects(vec![floor, wall, s1, s2]);
    let camera = Camera::new(WIDTH, HEIGHT, FOV).with_transform(view_transform(
        Point::new(15.0, 3.0, 0.0),
        Point::new(-7.0, 3.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    write(filepath, canvas.to_ppm().as_bytes()).unwrap();
}
