use crate::color::Color;
use crate::patterns::{Graphic, Pattern};
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
    transparency: f64,
    refractive_index: f64,
    /// The pattern on the material. This overrides the color, if it is not None.
    pattern: Graphic,
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
            transparency: 0.0,
            refractive_index: 1.0,
            pattern: None,
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

    pub fn with_pattern(mut self, p: Arc<dyn Pattern>) -> Self {
        self.pattern = Some(p);
        self
    }

    pub fn with_reflective(mut self, reflective: f64) -> Self {
        self.reflective = reflective;
        self
    }

    pub fn with_transparency(mut self, transparency: f64) -> Self {
        self.transparency = transparency;
        self
    }

    pub fn with_refractive_index(mut self, ri: f64) -> Self {
        self.refractive_index = ri;
        self
    }

    pub fn refractive_index(&self) -> f64 {
        self.refractive_index
    }

    pub fn transparency(&self) -> f64 {
        self.transparency
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
    let effective_color = match m.pattern {
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
            let h = hit(&intersections);
            match h {
                None => false,
                Some(i) => i.t() < distance,
            }
        }
    }
}

/// Computes the reflected component of the color at some intersection.
pub fn reflected_color(w: &World, comps: &IntersectionVals, limit: u16) -> Color {
    // If we've already reached the recursion limit, just assume that it is gonna reflect forever,
    // and return white.
    if limit <= 0 {
        return Color::white();
    }
    if comps.object.material().reflective == 0.0 {
        return Color::black();
    }
    // Reflect the ray, and find out what color the reflected ray's intersection ends up producing.
    let reflect_ray = Ray::new(comps.over_point, comps.reflectv);
    let color = w.color_of_ray(reflect_ray, limit - 1);
    color * comps.object.material().reflective
}

/// Computes the refracted component of the color at some intersection.
pub fn refracted_color(w: &World, comps: &IntersectionVals, limit: u16) -> Color {
    if limit <= 0 || comps.object.material().transparency == 0.0 {
        return Color::black();
    }

    // Not too sure what's going on here. But it's like how we compute the reflected color --
    // generate the refracted ray, then find out the color that ray produces.
    let n_ratio = comps.n1 / comps.n2;
    let cos_i = comps.eyev.dot(comps.normalv);
    let sin2_t = n_ratio * n_ratio * (1.0 - cos_i * cos_i);
    if sin2_t > 1.0 {
        return Color::white();
    }

    let cos_t = (1.0 - sin2_t).sqrt();
    let direction = comps.normalv * (n_ratio * cos_i - cos_t) - comps.eyev * n_ratio;
    let refract_ray = Ray::new(comps.under_point, direction);

    w.color_of_ray(refract_ray, limit - 1) * comps.object.material().transparency
}

#[cfg(test)]
mod tests {
    use super::{is_shadowed, lighting, reflected_color, refracted_color, Material, PointLight};
    use crate::color::Color;
    use crate::patterns::{Pattern, Stripe};
    use crate::ray::{Intersection, Ray};
    use crate::shapes::{Plane, Sphere};
    use crate::transform::Tr;
    use crate::tuple::{Point, Vector};
    use crate::world::{stock_sphere_a, stock_sphere_b, World};
    use crate::{p, v, MAX_BOUNCE};
    use std::f64::consts::SQRT_2;
    use std::sync::Arc;

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
            .with_pattern(Arc::new(Stripe::new(Color::white(), Color::black())))
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
        let comps = i.prepare_computations(ray, None);

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
        let comps = i.prepare_computations(r, None);
        let color = reflected_color(&w, &comps, 1);
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

        let comps = i.prepare_computations(r, None);
        let color = reflected_color(&w, &comps, MAX_BOUNCE);
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

        let comps = i.prepare_computations(r, None);
        let color = w.shade_hit(comps, MAX_BOUNCE);
        assert_eq!(color, Color::new(0.87675, 0.92434, 0.82917));
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
        w.color_of_ray(r, 8); // just want to check that it doesn't run infinitely
    }

    #[test]
    fn transparency_and_refractive_index_for_default_material() {
        let m = Material::default();
        assert_eq!(m.transparency, 0.0);
        assert_eq!(m.refractive_index, 1.0);
    }

    /// A helper function to generate, well, a glass sphere.
    fn glass_sphere() -> Sphere {
        Sphere::default().with_material(
            Material::default()
                .with_transparency(1.0)
                .with_refractive_index(1.5),
        )
    }

    #[test]
    fn glass_sphere_helper() {
        let s = glass_sphere().as_object();
        assert_eq!(s.transform(), Tr::default());
        assert_eq!(s.material().transparency, 1.0);
        assert_eq!(s.material().refractive_index, 1.5);
    }

    #[test]
    fn finding_n1_and_n2_at_intersections() {
        let a = glass_sphere()
            .with_transform(Tr::new().scale(2.0, 2.0, 2.0))
            .map_material(|m| m.with_refractive_index(1.5))
            .as_object();
        let b = glass_sphere()
            .with_transform(Tr::new().translate(0.0, 0.0, -0.25))
            .map_material(|m| m.with_refractive_index(2.0))
            .as_object();
        let c = glass_sphere()
            .with_transform(Tr::new().translate(0.0, 0.0, 0.25))
            .map_material(|m| m.with_refractive_index(2.5))
            .as_object();
        let r = Ray::new(p!(0, 0, -4), v!(0, 0, 1));
        let xs = vec![
            Intersection::new(2.0, a.clone()),
            Intersection::new(2.75, b.clone()),
            Intersection::new(3.25, c.clone()),
            Intersection::new(4.75, b.clone()),
            Intersection::new(5.25, c.clone()),
            Intersection::new(6.0, a.clone()),
        ];
        let tests = vec![
            (1.0, 1.5),
            (1.5, 2.0),
            (2.0, 2.5),
            (2.5, 2.5),
            (2.5, 1.5),
            (1.5, 1.0),
        ];
        for (idx, test) in tests.into_iter().enumerate() {
            let (n1, n2) = test;
            let comps = xs[idx].prepare_computations(r, Some(&xs));
            assert_eq!(comps.n1, n1);
            assert_eq!(comps.n2, n2);
        }
    }

    #[test]
    fn refracted_color_with_opaque_surface() {
        let w = World::default();
        let shape = w.objects[0].clone();
        let r = Ray::new(p!(0, 0, -5), v!(0, 0, 1));
        let xs = vec![
            Intersection::new(4.0, shape.clone()),
            Intersection::new(6.0, shape.clone()),
        ];

        let comps = xs[0].prepare_computations(r, Some(&xs));
        let c = refracted_color(&w, &comps, MAX_BOUNCE);
        assert_eq!(c, Color::black());
    }

    #[test]
    fn refracted_color_at_max_recursive_depth() {
        let w = World::default();
        let shape = stock_sphere_a()
            .map_material(|m| m.with_transparency(1.0).with_refractive_index(1.5))
            .as_object();
        let w = w.with_objects(vec![shape.clone()]);
        let r = Ray::new(p!(0, 0, -5), v!(0, 0, 1));
        let xs = vec![
            Intersection::new(4.0, shape.clone()),
            Intersection::new(6.0, shape.clone()),
        ];
        let comps = xs[0].prepare_computations(r, Some(&xs));
        let c = refracted_color(&w, &comps, 0);
        assert_eq!(c, Color::black());
    }

    #[test]
    fn refracted_color_under_total_internal_refraction() {
        let w = World::default();
        let shape = stock_sphere_a()
            .map_material(|m| m.with_transparency(1.0).with_refractive_index(1.5))
            .as_object();
        let w = w.with_objects(vec![shape.clone()]);
        let r = Ray::new(p!(0, 0, SQRT_2 / 2.0), v!(0, 1, 0));
        let xs = vec![
            Intersection::new(-SQRT_2 / 2.0, shape.clone()),
            Intersection::new(SQRT_2 / 2.0, shape.clone()),
        ];
        let comps = xs[1].prepare_computations(r, Some(&xs));
        let c = refracted_color(&w, &comps, MAX_BOUNCE);
        assert_eq!(c, Color::white());
    }

    #[derive(Debug)]
    struct TestPattern {
        transform: Tr,
    }

    impl TestPattern {
        fn new() -> Self {
            Self {
                transform: Tr::default(),
            }
        }
    }

    impl Pattern for TestPattern {
        fn color_at(&self, p: Point) -> Color {
            Color::new(p.x(), p.y(), p.z())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            todo!()
        }

        fn eqx(&self, _: &dyn std::any::Any) -> bool {
            todo!()
        }

        fn transform(&self) -> Tr {
            self.transform
        }

        fn inv_transform(&self) -> Tr {
            self.transform.inverse()
        }

        fn set_transform(&mut self, t: Tr) {
            self.transform = t;
        }
    }

    #[test]
    fn refracted_color_with_refracted_ray() {
        let w = World::default();
        let a = stock_sphere_a()
            .map_material(|m| {
                m.with_ambient(1.0)
                    .with_pattern(Arc::new(TestPattern::new()))
            })
            .as_object();
        let b = stock_sphere_b()
            .map_material(|m| m.with_transparency(1.0).with_refractive_index(1.5))
            .as_object();
        let w = w.with_objects(vec![a.clone(), b.clone()]);
        let r = Ray::new(p!(0, 0, 0.1), v!(0, 1, 0));
        let xs = vec![
            Intersection::new(-0.9899, a.clone()),
            Intersection::new(-0.4899, b.clone()),
            Intersection::new(0.4899, b.clone()),
            Intersection::new(0.9899, a.clone()),
        ];
        let comps = xs[2].prepare_computations(r, Some(&xs));
        let c = refracted_color(&w, &comps, MAX_BOUNCE);
        assert_eq!(c, Color::new(0.0, 0.99887, 0.04721));
    }

    #[test]
    fn shade_hit_with_transparent_material() {
        let mut w = World::default();
        let floor = Plane::default()
            .with_transform(Tr::new().translate(0.0, -1.0, 0.0))
            .with_material(
                Material::default()
                    .with_transparency(0.5)
                    .with_refractive_index(1.5),
            )
            .as_object();
        let ball = Sphere::default()
            .with_material(Material::default().with_color(Color::new(1.0, 0.0, 0.0)))
            .with_transform(Tr::new().translate(0.0, -3.5, -0.5))
            .as_object();
        w.add_objects(vec![floor.clone(), ball.clone()]);
        let r = Ray::new(p!(0, 0, -3), v!(0, -SQRT_2 / 2.0, SQRT_2 / 2.0));
        let xs = vec![Intersection::new(SQRT_2, floor.clone())];

        let comps = xs[0].prepare_computations(r, Some(&xs));
        let color = w.shade_hit(comps, MAX_BOUNCE);
        assert_eq!(color, Color::new(0.93642, 0.68642, 0.68642));
    }
}
