use crate::color::Color;
use crate::patterns::{Pattern, PatternX};
use crate::ray::{hit, IntersectionVals, Ray};
use crate::shapes::Shape;
use crate::tuple::{Point, Vector};
use crate::world::World;
use std::sync::Arc;

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

#[derive(Debug, PartialEq, Clone)]
pub struct Material {
    color: Color,
    ambient: f64,
    diffuse: f64,
    specular: f64,
    shininess: f64,
    reflective: f64,
    pattern: PatternX,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
            reflective: 0.0,
            pattern: Arc::new(None),
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

    pub fn with_pattern(mut self, p: Box<dyn Pattern>) -> Self {
        self.pattern = Arc::new(Some(p));
        self
    }

    pub fn with_reflective(mut self, reflective: f64) -> Self {
        self.reflective = reflective;
        self
    }
}

/// Computes the appropriate color at some point. This is the main function responsible for
/// figuring out the appropriate color for some pixel.
///
/// Note that the point passed to this function should be the `over_point`, nudged slightly away
/// from the surface.
pub fn lighting(
    m: Material,
    obj: &dyn Shape,
    light: PointLight,
    p: Point,
    eyev: Vector,
    normalv: Vector,
    in_shadow: bool,
) -> Color {
    // Check if the material has a pattern. If there is a pattern, we'll derive the color from the
    // pattern instead of the material's default color.
    let effective_color = match &*m.pattern {
        None => m.color * light.intensity,
        Some(pat) => pat.color_on_object(obj, p) * light.intensity,
    };
    let ambient = effective_color * m.ambient;
    // If the point is in shadow, then only the ambient contributes to its color.
    if in_shadow {
        return ambient;
    }
    let lightv = (light.position - p).normalize();
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

/// Determines if some point in the world is in a shadow.
pub fn is_shadowed(w: &World, p: Point) -> bool {
    match w.light {
        None => true,
        Some(l) => {
            let v = l.position - p;
            let distance = v.magnitude();
            let direction = v.normalize();

            let r = Ray::new(p, direction);
            let intersections = r.when_intersect_world(w);
            let h = hit(intersections);
            match h {
                None => false,
                Some(i) => i.t() < distance,
            }
        }
    }
}

/// Computes the reflected component of the color at some intersection.
pub fn reflected_color(w: &World, comps: IntersectionVals, limit: u16) -> Color {
    if limit <= 0 {
        return Color::white();
    }
    if comps.object.material().reflective == 0.0 {
        return Color::black();
    }
    let reflect_ray = Ray::new(comps.over_point, comps.reflectv);
    let color = w.color_at(reflect_ray, limit - 1);
    color * comps.object.material().reflective
}

#[cfg(test)]
mod tests {
    use super::{is_shadowed, lighting, reflected_color, Material, PointLight};
    use crate::color::Color;
    use crate::patterns::{Pattern, Stripe};
    use crate::ray::{Intersection, Ray};
    use crate::shapes::{Plane, Sphere};
    use crate::transform::Tr;
    use crate::tuple::{Point, Vector};
    use crate::world::World;
    use crate::{p, v, MAX_REFLECTION};
    use std::f64::consts::SQRT_2;

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

    /// Helps to test the lighting function.
    macro_rules! test_lighting {
        ($($name:ident: $eyev:expr, $normalv:expr, $light:expr, $in_shadow:expr, $want:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let got = lighting(Material::default(), &Sphere::default(), $light, Point::origin(), $eyev, $normalv, $in_shadow);
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
        false,
        Color::new(1.9, 1.9, 1.9),

    lighting_with_eye_between_light_and_surface_offset_45deg:
        Vector::new(0.0, 2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0),
        Vector::new(0.0, 0.0, -1.0),
        PointLight::new(Point::new(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)),
        false,
        Color::new(1.0, 1.0, 1.0),

    lighting_with_eye_opposite_surface_light_offset_45deg:
        Vector::new(0.0, 0.0, -1.0),
        Vector::new(0.0, 0.0, -1.0),
        PointLight::new(Point::new(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)),
        false,
        Color::new(0.7364,0.7364,0.7364),

    lighting_with_eye_in_path_of_reflection_vector:
        Vector::new(0.0, -2.0_f64.sqrt()/2.0, -2.0_f64.sqrt()/2.0),
        Vector::new(0.0, 0.0, -1.0),
        PointLight::new(Point::new(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)),
        false,
        Color::new(1.6364, 1.6364, 1.6364),

    lighting_with_light_behind_surface:
        Vector::new(0.0, 0.0, -1.0),
        Vector::new(0.0, 0.0, -1.0),
        PointLight::new(Point::new(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0)),
        false,
        Color::new(0.1, 0.1, 0.1),

    lighting_with_surface_in_shadow:
        Vector::new(0.0, 0.0, -1.0),
        Vector::new(0.0, 0.0, -1.0),
        PointLight::new(Point::new(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)),
        true,
        Color::new(0.1, 0.1, 0.1),
    }

    /// Tests if a point is correctly determined to be in the shadow, checked against the default
    /// world.
    macro_rules! test_is_shadowed {
        ($($name:ident: $p:expr, $want:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let w = World::default();
                let got = is_shadowed(&w, $p);
                assert_eq!(got, $want);
            }
        )*
        };
    }

    test_is_shadowed! {
    no_shadow_when_nothing_collinear_with_point_and_light:
        Point::new(0.0, 10.0, 0.0),
        false,

    has_shadow_when_object_between_light_and_point:
        Point::new(10.0, -10.0, 10.0),
        true,

    no_shadow_when_object_behind_light:
        Point::new(-20.0, 20.0, -20.0),
        false,

    no_shadow_when_object_behind_point:
        Point::new(-2.0, -2.0, -2.0),
        false,
    }

    #[test]
    fn lighting_with_a_pattern_applied() {
        let m = Material::default()
            .with_pattern(Stripe::new(Color::white(), Color::black()).as_box())
            .with_ambient(1.0)
            .with_diffuse(0.0)
            .with_specular(0.0);
        let eyev = v!(0.0, 0.0, -1.0);
        let normalv = v!(0.0, 0.0, -1.0);
        let light = PointLight::new(p!(0.0, 0.0, -10.0), Color::white());

        let c1 = lighting(
            m.clone(),
            &Sphere::default(),
            light,
            p!(0.9, 0.0, 0.0),
            eyev,
            normalv,
            false,
        );
        assert_eq!(c1, Color::white());

        let c2 = lighting(
            m,
            &Sphere::default(),
            light,
            p!(1.1, 0.0, 0.0),
            eyev,
            normalv,
            false,
        );
        assert_eq!(c2, Color::black());
    }

    #[test]
    fn default_material_is_not_reflective() {
        let m = Material::default();
        assert_eq!(m.reflective, 0.0);
    }

    #[test]
    fn precompute_reflection_vector() {
        let shape = Plane::default().as_object();
        let ray = Ray::new(p!(0, 1, -1), v!(0, -SQRT_2 / 2.0, SQRT_2 / 2.0));
        let i = Intersection::new(SQRT_2, shape);
        let comps = i.prepare_computations(ray);

        let got = comps.reflectv;
        let want = v!(0, SQRT_2 / 2.0, SQRT_2 / 2.0);
        assert_eq!(got, want);
    }

    #[test]
    fn reflected_color_for_nonreflective_material() {
        let s = Sphere::default()
            .with_transform(Tr::default().scale(0.5, 0.5, 0.5))
            .with_material(Material::default().with_ambient(1.0))
            .as_object();
        let mut w = World::default();
        w.clear_objects();
        w.add_objects(vec![s.clone()]);
        let i = Intersection::new(1.0, s);

        let r = Ray::new(Point::origin(), v!(0, 0, 1));
        let comps = i.prepare_computations(r);
        let color = reflected_color(&w, comps, 1);
        assert_eq!(color, Color::black());
    }

    #[test]
    fn reflected_color_for_reflective_material() {
        let mut w = World::default();
        let shape = Plane::default()
            .with_material(Material::default().with_reflective(0.5))
            .with_transform(Tr::default().translate(0.0, -1.0, 0.0))
            .as_object();
        w.add_objects(vec![shape.clone()]);
        let r = Ray::new(p!(0, 0, -3), v!(0, -SQRT_2 / 2.0, SQRT_2 / 2.0));
        let i = Intersection::new(SQRT_2, shape);

        let comps = i.prepare_computations(r);
        let color = reflected_color(&w, comps, MAX_REFLECTION);
        assert_eq!(color, Color::new(0.19033, 0.23792, 0.14275));
    }

    #[test]
    fn shade_hit_with_reflective_material() {
        let mut w = World::default();
        let shape = Plane::default()
            .with_material(Material::default().with_reflective(0.5))
            .with_transform(Tr::default().translate(0.0, -1.0, 0.0))
            .as_object();
        w.add_objects(vec![shape.clone()]);
        let r = Ray::new(p!(0, 0, -3), v!(0, -SQRT_2 / 2.0, SQRT_2 / 2.0));
        let i = Intersection::new(SQRT_2, shape);

        let comps = i.prepare_computations(r);
        let color = w.shade_hit(comps, MAX_REFLECTION);
        assert_eq!(color, Color::new(0.87677, 0.92436, 0.82918));
    }

    #[test]
    fn color_at_with_mutually_reflective_surfaces() {
        // Bounce a ray between two maximally reflective planes to ensure that we don't do infinite
        // recursion.
        let mut w = World::new().with_light(PointLight::new(p!(0, 0, 0), Color::white()));
        let lower = Plane::default()
            .with_material(Material::default().with_reflective(1.0))
            .with_transform(Tr::new().translate(0.0, -1.0, 0.0))
            .as_object();
        let upper = Plane::default()
            .with_material(Material::default().with_reflective(1.0))
            .with_transform(Tr::new().translate(0.0, 1.0, 0.0))
            .as_object();
        w.add_objects(vec![lower, upper]);
        let r = Ray::new(p!(0, 0, 0), v!(0, 1, 0));
        w.color_at(r, 8); // just want to check that it doesn't run infinitely
    }
}
