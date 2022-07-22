// Draws a simple parabolic path.

use std::io::Write;
use std::{env, fs, io};
use toytracer::canvas::Canvas;
use toytracer::color::Color;
use toytracer::tuple::{Point, Vector};
use toytracer::{file_exists, pad_filepath};

fn main() -> Result<(), io::Error> {
    let filename = env::args()
        .nth(1)
        .unwrap_or("./tmp/projectile.ppm".to_string());
    let filename = pad_filepath(&filename, file_exists);

    let mut p = Projectile {
        position: Point::new(0.0, 1.0, 0.0),
        velocity: Vector::new(1.0, 1.8, 0.0).normalize() * 11.25,
    };
    let e = Environment {
        gravity: Vector::new(0.0, -0.1, 0.0),
        wind: Vector::new(-0.01, 0.0, 0.0),
    };
    let mut canvas = Canvas::new(900, 550);
    while p.position.y() > 0.0 {
        canvas.draw(
            p.position.x() as usize,
            canvas.height() - p.position.y() as usize,
            Color::new(1.0, 0.0, 0.0),
        );
        p = tick(&e, p);
    }
    let s = canvas.to_ppm();
    let mut file = fs::File::create(&filename)?;
    file.write_all(s.as_bytes())?;

    println!("simulation done, file written to {}", filename);

    Ok(())
}

#[derive(Debug)]
struct Projectile {
    position: Point,
    velocity: Vector,
}

struct Environment {
    gravity: Vector,
    wind: Vector,
}

fn tick(e: &Environment, p: Projectile) -> Projectile {
    Projectile {
        position: p.position + p.velocity,
        velocity: p.velocity + e.gravity + e.wind,
    }
}