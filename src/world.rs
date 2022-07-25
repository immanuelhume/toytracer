use crate::color::Color;
use crate::light::{is_shadowed, lighting, Material, PointLight};
use crate::ray::{hit, IntersectionVals, Ray};
use crate::shapes::{Object, Sphere};
use crate::transform::Tr;
use crate::tuple::Point;
use std::sync::Arc;

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

    pub fn add_objects(mut self, mut objects: Vec<Object>) -> Self {
        self.objects.append(&mut objects);
        self
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

    fn shade_hit(&self, c: IntersectionVals) -> Color {
        let shadowed = is_shadowed(&self, c.over_point);
        lighting(
            c.object.material(),
            self.light
                .unwrap_or(PointLight::new(Point::origin(), Color::black())),
            c.point,
            c.eyev,
            c.normalv,
            shadowed,
        )
    }

    pub fn color_at(&self, r: Ray) -> Color {
        let intersections = r.when_intersect_world(self);
        if intersections.len() == 0 {
            return Color::black();
        }
        match hit(intersections) {
            Some(i) => self.shade_hit(i.prepare_computations(r)),
            None => Color::black(),
        }
    }
}

impl Default for World {
    fn default() -> Self {
        let light = PointLight::new(Point::new(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let s1 = Sphere::default().with_material(
            Material::default()
                .with_color(Color::new(0.8, 1.0, 0.6))
                .with_diffuse(0.7)
                .with_specular(0.2),
        );
        let s2 = Sphere::default().with_transform(Tr::default().scale(0.5, 0.5, 0.5));
        Self {
            light: Some(light),
            objects: vec![Arc::new(s1), Arc::new(s2)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::color::Color;
    use crate::light::{Material, PointLight};
    use crate::ray::{Intersection, Ray};
    use crate::shapes::Sphere;
    use crate::transform::Tr;
    use crate::tuple::{Point, Vector};
    use std::sync::Arc;

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
        let comps = i.prepare_computations(r);

        let got = w.shade_hit(comps);
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
        let comps = i.prepare_computations(r);

        let got = w.shade_hit(comps);
        let want = Color::new(0.90498, 0.90498, 0.90498);
        assert_eq!(got, want);
    }

    #[test]
    fn color_when_ray_misses() {
        let w = World::default();
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 1.0, 0.0));

        let got = w.color_at(r);
        let want = Color::new(0.0, 0.0, 0.0);
        assert_eq!(got, want);
    }

    #[test]
    fn color_when_ray_hits() {
        let w = World::default();
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));

        let got = w.color_at(r);
        let want = Color::new(0.38066, 0.47583, 0.2855);
        assert_eq!(got, want);
    }

    #[test]
    fn color_when_intersection_behind_ray() {
        let w = World::default().with_objects(vec![
            Arc::new(
                Sphere::default().with_material(
                    Material::default()
                        .with_color(Color::new(0.8, 1.0, 0.6))
                        .with_ambient(1.0)
                        .with_diffuse(0.7)
                        .with_specular(0.2),
                ),
            ),
            Arc::new(
                Sphere::default()
                    .with_transform(Tr::default().scale(0.5, 0.5, 0.5))
                    .with_material(Material::default().with_ambient(1.0)),
            ),
        ]);
        let r = Ray::new(Point::new(0.0, 0.0, 0.75), Vector::new(0.0, 0.0, -1.0));

        let got = w.color_at(r);
        let want = w.objects[1].material().color();
        assert_eq!(got, want);
    }

    #[test]
    fn shade_hit_with_intersection_in_shadow() {
        let w = World::default()
            .with_light(PointLight::new(
                Point::new(0.0, 0.0, -10.0),
                Color::new(1.0, 1.0, 1.0),
            ))
            .add_objects(vec![
                Arc::new(Sphere::default()),
                Arc::new(Sphere::default().with_transform(Tr::default().translate(0.0, 0.0, 10.0))),
            ]);
        let r = Ray::new(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, 1.0));
        let i = Intersection::new(4.0, w.objects[1].clone());
        let comps = i.prepare_computations(r);

        let got = w.shade_hit(comps);
        let want = Color::new(0.1, 0.1, 0.1);
        assert_eq!(got, want);
    }
}
