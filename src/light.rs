use crate::color::Color;
use crate::tuple::{Point, Vector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    position: Point,
    intensity: Color,
}

impl PointLight {
    pub fn new(position: Point, intensity: Color) -> Self {
        Self {
            position,
            intensity,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Material {
    color: Color,
    ambient: f64,
    diffuse: f64,
    specular: f64,
    shininess: f64,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
        }
    }
}

impl Material {
    pub fn color(&self) -> Color {
        self.color
    }

    pub fn with_color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn with_ambient(mut self, v: f64) -> Self {
        self.ambient = v;
        self
    }

    pub fn with_diffuse(mut self, v: f64) -> Self {
        self.diffuse = v;
        self
    }

    pub fn with_specular(mut self, v: f64) -> Self {
        self.specular = v;
        self
    }

    pub fn with_shininess(mut self, v: f64) -> Self {
        self.shininess = v;
        self
    }
}

/// Computes the appropriate color at some point.
pub fn lighting(
    m: Material,
    light: PointLight,
    point: Point,
    eyev: Vector,
    normalv: Vector,
) -> Color {
    let effective_color = m.color * light.intensity;
    let lightv = (light.position - point).normalize();
    let ambient = effective_color * m.ambient;
    let light_dot_normal = lightv.dot(normalv);
    let (diffuse, specular) = if light_dot_normal < 0.0 {
        (Color::black(), Color::black())
    } else {
        let diffuse = effective_color * m.diffuse * light_dot_normal;
        let reflectv = (-lightv).reflect(normalv);
        let reflect_dot_eye = reflectv.dot(eyev);
        let specular = if reflect_dot_eye < 0.0 {
            Color::black()
        } else {
            let factor = reflect_dot_eye.powf(m.shininess);
            light.intensity * m.specular * factor
        };
        (diffuse, specular)
    };
    ambient + diffuse + specular
}

#[cfg(test)]
mod tests {
    use super::{lighting, Material, PointLight};
    use crate::color::Color;
    use crate::tuple::{Point, Vector};

    #[test]
    fn point_light_has_position_and_intensity() {
        let intensity = Color::new(1.0, 1.0, 1.0);
        let position = Point::new(0.0, 0.0, 0.0);

        let got = PointLight::new(position, intensity);
        assert_eq!(got.intensity, intensity);
        assert_eq!(got.position, position);
    }

    #[test]
    fn default_material() {
        let m = Material::default();

        assert_eq!(m.color, Color::new(1.0, 1.0, 1.0));
        assert_eq!(m.ambient, 0.1);
        assert_eq!(m.diffuse, 0.9);
        assert_eq!(m.specular, 0.9);
        assert_eq!(m.shininess, 200.0);
    }

    macro_rules! test_lighting {
        ($($name:ident: $eyev:expr, $normalv:expr, $light:expr, $want:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let got = lighting(Material::default(), $light, Point::origin(), $eyev, $normalv);
                assert_eq!(got, $want);
            }
        )*
        };
    }

    test_lighting! {
    lighting_with_eye_between_light_and_surface:
        Vector::new(0.0, 0.0, -1.0),
        Vector::new(0.0, 0.0, -1.0),
        PointLight::new(Point::new(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)),
        Color::new(1.9, 1.9, 1.9),

    lighting_with_eye_between_light_and_surface_offset_45deg:
        Vector::new(0.0, 2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0),
        Vector::new(0.0, 0.0, -1.0),
        PointLight::new(Point::new(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)),
        Color::new(1.0, 1.0, 1.0),

    lighting_with_eye_opposite_surface_light_offset_45deg:
        Vector::new(0.0, 0.0, -1.0),
        Vector::new(0.0, 0.0, -1.0),
        PointLight::new(Point::new(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)),
        Color::new(0.7364,0.7364,0.7364),

    lighting_with_eye_in_path_of_reflection_vector:
        Vector::new(0.0, -2.0_f64.sqrt()/2.0, -2.0_f64.sqrt()/2.0),
        Vector::new(0.0, 0.0, -1.0),
        PointLight::new(Point::new(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)),
        Color::new(1.6364, 1.6364, 1.6364),

    lighting_with_light_behind_surface:
        Vector::new(0.0, 0.0, -1.0),
        Vector::new(0.0, 0.0, -1.0),
        PointLight::new(Point::new(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0)),
        Color::new(0.1, 0.1, 0.1),
    }
}
