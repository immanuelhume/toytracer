use crate::color::Color;
use crate::light::{is_shadowed, lighting, reflected_color, refracted_color, Material, PointLight};
use crate::ray::{hit, schlick, ItrsectnVs, Ray};
use crate::shapes::{Object, Sphere};
use crate::transform::Tr;
use crate::tuple::Point;

pub struct World {
    pub light: Option<PointLight>,
    pub objects: Vec<Object>,
}

impl World {
    /// Creates an empty world, with no light and no objects.
    pub fn new() -> Self {
        Self {
            light: None,
            objects: Vec::new(),
        }
    }

    pub fn with_light(mut self, light: PointLight) -> Self {
        self.light = Some(light);
        self
    }

    pub fn add_objects(&mut self, mut objects: Vec<Object>) {
        self.objects.append(&mut objects);
    }

    pub fn with_objects(mut self, objects: Vec<Object>) -> Self {
        self.objects = objects;
        self
    }

    pub fn map_objects<T>(mut self, f: T) -> Self
    where
        T: Fn(Object) -> Object,
    {
        self.objects = self.objects.into_iter().map(f).collect();
        self
    }

    /// Removes all objects from the world.
    pub fn clear_objects(&mut self) {
        self.objects.clear();
    }

    /// Computes the correct color at some point of intersection (between a ray and an object).
    /// This function takes into account reflection and reflection.
    pub fn shade_hit(&self, c: ItrsectnVs, limit: u16) -> Color {
        let surface = lighting(
            c.object.material(),
            &*c.object,
            self.light
                .unwrap_or(PointLight::new(Point::origin(), Color::black())),
            c.over_point,
            c.eyev,
            c.normalv,
            is_shadowed(self, c.over_point),
        );
        let reflected = reflected_color(self, &c, limit);
        let refracted = refracted_color(self, &c, limit);

        let material = c.object.material();
        if material.refractive_index() > 0.0 && material.transparency() > 0.0 {
            let reflectance = schlick(c);
            return surface + reflected * reflectance + refracted * (1.0 - reflectance);
        }
        surface + reflected + refracted
    }

    /// Given a ray, computes the color of the point which the ray hits. If the ray does not hit
    /// any point it just returns black.
    pub fn color_of_ray(&self, r: Ray, limit: u16) -> Color {
        let intersections = r.when_intersect_world(self);
        if intersections.len() == 0 {
            return Color::black();
        }
        match hit(&intersections) {
            Some(i) => self.shade_hit(i.prepare_computations(r, Some(&intersections)), limit),
            None => Color::black(),
        }
    }
}

/// Returns a sphere of radius one at the origin. Used for testing.
pub fn stock_sphere_a() -> Sphere {
    Sphere::default().with_material(
        Material::default()
            .with_color(Color::new(0.8, 1.0, 0.6))
            .with_diffuse(0.7)
            .with_specular(0.2),
    )
}

/// Returns a sphere of radius half at the origin. Used for testing.
pub fn stock_sphere_b() -> Sphere {
    Sphere::default().with_transform(Tr::default().scale(0.5, 0.5, 0.5))
}

impl Default for World {
    fn default() -> Self {
        let light = PointLight::new(Point::new(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        Self {
            light: Some(light),
            objects: vec![stock_sphere_a().as_object(), stock_sphere_b().as_object()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::color::Color;
    use crate::light::{Material, PointLight};
    use crate::ray::{Intersection, Ray};
    use crate::shapes::{Plane, Sphere};
    use crate::transform::Tr;
    use crate::tuple::{Point, Vector};
    use crate::{p, v, MAX_REFLECTION};
    use std::f64::consts::SQRT_2;

    #[test]
    fn creating_a_world() {
        let w = World::new();
        assert_eq!(w.light, None);
        assert_eq!(w.objects.len(), 0);
    }

    #[test]
    fn the_default_world() {
        let light = PointLight::new(Point::new(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));

        let w = World::default();
        assert_eq!(w.light.unwrap(), light);

        // We can't assert equality on the world's objects here, because every sphere is supposed
        // to have a unique ID.
        assert_eq!(w.objects.len(), 2);
    }

    #[test]
    fn intersect_world_with_ray() {
        let w = World::default();
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));

        let got = r.when_intersect_world(&w);
        assert_eq!(got.len(), 4);

        let got: Vec<f64> = got.into_iter().map(|x| x.t()).collect();
        let want = vec![4.0, 4.5, 5.5, 6.0];
        assert_eq!(got, want);
    }

    #[test]
    fn shading_an_intersection() {
        let w = World::default();
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let shape = w.objects[0].clone();
        let i = Intersection::new(4.0, shape);
        let comps = i.prepare_computations(r, None);

        let got = w.shade_hit(comps, MAX_REFLECTION);
        let want = Color::new(0.38066, 0.47583, 0.2855);
        assert_eq!(got, want);
    }

    #[test]
    fn shading_an_intersection_from_the_inside() {
        let mut w = World::default();
        w.light = Some(PointLight::new(
            Point::new(0.0, 0.25, 0.0),
            Color::new(1.0, 1.0, 1.0),
        ));
        let r = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let shape = w.objects[1].clone();
        let i = Intersection::new(0.5, shape);
        let comps = i.prepare_computations(r, None);

        let got = w.shade_hit(comps, MAX_REFLECTION);
        let want = Color::new(0.90498, 0.90498, 0.90498);
        assert_eq!(got, want);
    }

    #[test]
    fn color_when_ray_misses() {
        let w = World::default();
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 1.0, 0.0));

        let got = w.color_of_ray(r, MAX_REFLECTION);
        let want = Color::new(0.0, 0.0, 0.0);
        assert_eq!(got, want);
    }

    #[test]
    fn color_when_ray_hits() {
        let w = World::default();
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));

        let got = w.color_of_ray(r, MAX_REFLECTION);
        let want = Color::new(0.38066, 0.47583, 0.2855);
        assert_eq!(got, want);
    }

    #[test]
    fn color_when_intersection_behind_ray() {
        let w = World::default().with_objects(vec![
            Sphere::default()
                .with_material(
                    Material::default()
                        .with_color(Color::new(0.8, 1.0, 0.6))
                        .with_ambient(1.0)
                        .with_diffuse(0.7)
                        .with_specular(0.2),
                )
                .as_object(),
            Sphere::default()
                .with_transform(Tr::default().scale(0.5, 0.5, 0.5))
                .with_material(Material::default().with_ambient(1.0))
                .as_object(),
        ]);
        let r = Ray::new(Point::new(0.0, 0.0, 0.75), Vector::new(0.0, 0.0, -1.0));

        let got = w.color_of_ray(r, MAX_REFLECTION);
        let want = w.objects[1].material().color();
        assert_eq!(got, want);
    }

    #[test]
    fn shade_hit_with_intersection_in_shadow() {
        let mut w = World::default().with_light(PointLight::new(
            Point::new(0.0, 0.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        ));
        w.add_objects(vec![
            Sphere::default().as_object(),
            Sphere::default()
                .with_transform(Tr::default().translate(0.0, 0.0, 10.0))
                .as_object(),
        ]);
        let r = Ray::new(p!(0.0, 0.0, 5.0), v!(0.0, 0.0, 1.0));
        let i = Intersection::new(4.0, w.objects[1].clone());
        let comps = i.prepare_computations(r, None);

        let got = w.shade_hit(comps, MAX_REFLECTION);
        let want = Color::new(0.1, 0.1, 0.1);
        assert_eq!(got, want);
    }

    #[test]
    fn shade_hit_with_reflective_transparent_material() {
        let mut w = World::default();
        let r = Ray::new(p!(0, 0, -3), v!(0, -SQRT_2 / 2.0, SQRT_2 / 2.0));
        let floor = Plane::default()
            .with_transform(Tr::new().translate(0.0, -1.0, 0.0))
            .with_material(
                Material::default()
                    .with_reflective(0.5)
                    .with_transparency(0.5)
                    .with_refractive_index(1.5),
            )
            .as_object();
        let ball = Sphere::default()
            .with_transform(Tr::new().translate(0.0, -3.5, -0.5))
            .with_material(
                Material::default()
                    .with_color(Color::new(1.0, 0.0, 0.0))
                    .with_ambient(0.5),
            )
            .as_object();
        w.add_objects(vec![floor.clone(), ball]);
        let xs = vec![Intersection::new(SQRT_2, floor.clone())];
        let comps = xs[0].prepare_computations(r, Some(&xs));
        let color = w.shade_hit(comps, MAX_REFLECTION);
        assert_eq!(color, Color::new(0.93391, 0.69643, 0.69243));
    }
}
