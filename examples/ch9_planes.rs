use std::env;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3};
use std::fs::write;
use toytracer::camera::Camera;
use toytracer::color::Color;
use toytracer::light::{Material, PointLight};
use toytracer::patterns::{Checkers, Pattern, Stripe};
use toytracer::shapes::{Plane, Sphere};
use toytracer::transform::{view_transform, Tr};
use toytracer::tuple::{Point, Vector};
use toytracer::world::World;
use toytracer::{file_exists, p, pad_filepath, v};

const WIDTH: usize = 16 * 128;
const HEIGHT: usize = 9 * 128;
const FOV: f64 = FRAC_PI_3;

fn main() {
    let path = env::args().nth(1).unwrap_or("./tmp/plane.ppm".to_string());
    let path = pad_filepath(&path, file_exists);

    println!("output will be written to {}", path);

    let camera = Camera::new(WIDTH, HEIGHT, FOV).with_transform(view_transform(
        p!(10.0, 4.0, -5.0),
        p!(-10.0, 1.0, 4.0),
        v!(0.0, 1.0, 0.05),
    ));
    let mut world = World::new().with_light(PointLight::new(p!(6.0, 4.0, 3.0), Color::white()));

    {
        // Set up the room.
        let floor = Plane::default()
            .with_material(
                Material::default()
                    .with_specular(0.1)
                    .with_pattern(
                        Checkers::new(Color::sh_black_coral(), Color::sh_pale_silver())
                            .with_transform(Tr::default().scale_prop(2.0))
                            .as_box(),
                    )
                    .with_reflective(0.1),
            )
            .as_object();
        let ceil = Plane::default()
            .with_transform(Tr::default().translate(0.0, 6.0, 0.0))
            .with_material(floor.material())
            .as_object();
        let wall = Plane::default()
            .with_transform(Tr::default().rotate_z(FRAC_PI_2).translate(-6.0, 0.0, 0.0))
            .with_material(floor.material())
            .as_object();
        let other_wall = Plane::default()
            .with_transform(Tr::default().rotate_z(FRAC_PI_2).translate(11.0, 0.0, 0.0))
            .with_material(floor.material())
            .as_object();

        world.add_objects(vec![floor, ceil, wall, other_wall]);
    }

    {
        let big_pink = Sphere::default()
            .with_material(
                Material::default()
                    .with_color(Color::pw_charm_pink())
                    .with_specular(0.6)
                    .with_reflective(1.0),
            )
            .with_transform(
                Tr::default()
                    .scale_prop(2.0)
                    .rotate_y(FRAC_PI_3)
                    .translate(0.0, 2.0, 1.8),
            )
            .as_object();

        world.add_objects(vec![big_pink]);
    }

    {
        let a = Sphere::default()
            .with_material(
                Material::default()
                    .with_pattern(
                        Stripe::new(Color::sh_taupe_gray(), Color::sh_slate_gray())
                            .with_transform(Tr::default().scale(0.28, 1.0, 1.0))
                            .as_box(),
                    )
                    .with_specular(0.3),
            )
            .with_transform(
                Tr::default()
                    .rotate_z(0.2)
                    .rotate_y(FRAC_PI_2)
                    .rotate_x(0.2)
                    .translate(2.0, 1.0, -4.0),
            )
            .as_object();

        let b = Sphere::default()
            .with_material(
                Material::default().with_pattern(
                    Stripe::new(Color::sh_pale_silver(), Color::sh_slate_gray())
                        .with_transform(Tr::default().scale(0.25, 1.0, 1.0))
                        .as_box(),
                ),
            )
            .with_transform(
                Tr::default()
                    .scale_prop(1.0)
                    .rotate_y(FRAC_PI_2)
                    .rotate_x(FRAC_PI_3)
                    .rotate_z(-0.1)
                    .translate(-1.0, 1.0, -4.0),
            )
            .as_object();

        let c = Sphere::default()
            .with_material(
                Material::default().with_pattern(
                    Stripe::new(Color::sh_pale_silver(), Color::sh_ash_gray())
                        .with_transform(Tr::default().scale(0.23, 1.0, 1.0))
                        .as_box(),
                ),
            )
            .with_transform(
                Tr::default()
                    .scale_prop(1.0)
                    .rotate_y(FRAC_PI_2)
                    .rotate_x(FRAC_PI_2 - 0.1)
                    .rotate_z(0.1)
                    .translate(-0.8, 1.0, -6.0),
            )
            .as_object();

        world.add_objects(vec![a, b, c]);
    }

    let canvas = camera.render(&world);
    write(path, canvas.to_ppm().as_bytes()).unwrap();
}
