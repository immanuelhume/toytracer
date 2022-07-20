use crate::color::Color;
use crate::light::{lighting, Material, PointLight};
use crate::ray::{hit, IntersectionVals, Ray, Sphere};
use crate::transformation::scaling;
use crate::tuple::Point;

pub struct World {
    light: Option<PointLight>,
    pub objects: Vec<Sphere>,
}

impl World {
    fn new() -> Self {
        Self {
            light: None,
            objects: Vec::new(),
        }
    }

    fn shade_hit(&self, c: IntersectionVals) -> Color {
        lighting(
            c.object.material(),
            self.light
                .unwrap_or(PointLight::new(Point::origin(), Color::black())),
            c.point,
            c.eyev,
            c.normalv,
        )
    }

    fn color_at(&self, r: Ray) -> Color {
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
        let mut s1 = Sphere::default();
        s1.set_material(
            Material::default()
                .set_color(Color::new(0.8, 1.0, 0.6))
                .set_diffuse(0.7)
                .set_specular(0.2),
        );
        let mut s2 = Sphere::default();
        s2.set_transform(scaling(0.5, 0.5, 0.5));
        Self {
            light: Some(light),
            objects: vec![s1, s2],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::color::Color;
    use crate::light::{Material, PointLight};
    use crate::ray::{Intersection, Ray, Sphere};
    use crate::transformation::scaling;
    use crate::tuple::{Point, Vector};

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
        let shape = &w.objects[0];
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
        let shape = &w.objects[1];
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
        let mut w = World::default();
        {
            let outer = &mut w.objects[0];
            outer.set_material(outer.material().set_ambient(1.0));
            let inner = &mut w.objects[1];
            inner.set_material(inner.material().set_ambient(1.0));
        }
        let r = Ray::new(Point::new(0.0, 0.0, 0.75), Vector::new(0.0, 0.0, -1.0));

        let got = w.color_at(r);
        let want = w.objects[1].material().color();
        assert_eq!(got, want);
    }
}
