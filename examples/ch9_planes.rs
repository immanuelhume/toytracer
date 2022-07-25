use std::env;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3};
use std::fs::write;
use toytracer::camera::Camera;
use toytracer::color::Color;
use toytracer::light::{Material, PointLight};
use toytracer::shapes::{Plane, Sphere};
use toytracer::transform::{view_transform, Tr};
use toytracer::tuple::{Point, Vector};
use toytracer::world::World;
use toytracer::{file_exists, p, pad_filepath, v};

const WIDTH: usize = 1024;
const HEIGHT: usize = 512;
const FOV: f64 = FRAC_PI_3;

fn main() {
    let path = env::args().nth(1).unwrap_or("./tmp/plane.ppm".to_string());
    let path = pad_filepath(&path, file_exists);

    println!("output will be written to {}", path);

    let floor = Plane::default()
        .with_material(
            Material::default()
                .with_specular(0.1)
                .with_color(Color::new(0.3, 0.3, 0.3)),
        )
        .as_object();
    let ceil = Plane::default()
        .with_transform(Tr::default().translate(0.0, 6.0, 0.0))
        .with_material(floor.material())
        .as_object();
    let wall = Plane::default()
        .with_transform(Tr::default().rotate_z(FRAC_PI_2).translate(-5.0, 0.0, 0.0))
        .with_material(floor.material())
        .as_object();

    let s1 = Sphere::default()
        .with_material(Material::default().with_color(Color::new(0.3, 0.4, 0.5)))
        .with_transform(Tr::default().translate(5.0, 1.0, 1.0).scale(1.0, 1.0, 1.0))
        .as_object();

    let s2 = Sphere::default()
        .with_material(Material::default().with_color(Color::new(0.5, 0.4, 0.3)))
        .with_transform(Tr::default().translate(0.0, 1.0, -2.0).scale(2.0, 2.0, 2.0))
        .as_object();

    let s3 = Sphere::default()
        .with_material(Material::default().with_color(Color::white()))
        .with_transform(Tr::default().scale(0.2, 0.2, 0.2).translate(10.0, 0.2, 0.0))
        .as_object();

    let world = World::default()
        .with_objects(vec![floor, ceil, wall, s1, s2, s3])
        .with_light(PointLight::new(p!(20.0, 5.0, 10.0), Color::white()));

    let camera = Camera::new(WIDTH, HEIGHT, FOV).with_transform(view_transform(
        p!(15.0, 1.0, 0.0),
        p!(-10.0, 2.0, 0.0),
        v!(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    write(path, canvas.to_ppm().as_bytes()).unwrap();
}
