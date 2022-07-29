use crate::light::Material;
use crate::shapes::Object;
use crate::transform::Tr;
use crate::tuple::{Point, Vector};
use crate::world::World;
use crate::EPSILON;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    origin: Point,
    direction: Vector,
}

impl Ray {
    pub fn new(origin: Point, direction: Vector) -> Self {
        Self { origin, direction }
    }

    /// Get the position of this ray at some time t.
    pub fn position_at(&self, t: f64) -> Point {
        self.origin + t * self.direction
    }

    /// Finds all the places where this ray intersects with stuff in a given world. The list of
    /// intersections returned will be sorted by increasing distance form the ray's origin.
    pub fn when_intersect_world(&self, w: &World) -> Vec<Itrsectn> {
        let mut res = Vec::new();
        for obj in &w.objects {
            res.append(&mut obj.intersect_with(*self));
        }

        // Sort every intersection by it's t value.
        res.sort_by(|a, b| a.t.total_cmp(&b.t));
        res
    }

    pub fn with_transform(&self, t: Tr) -> Self {
        let m = t.matrix();
        Self {
            origin: m * self.origin,
            direction: m * self.direction,
        }
    }

    pub fn origin(&self) -> Point {
        self.origin
    }

    pub fn direction(&self) -> Vector {
        self.direction
    }
}

#[derive(Debug, Clone)]
pub struct Itrsectn {
    t: f64,
    object: Object,
}

impl Itrsectn {
    pub fn new(t: f64, object: Object) -> Self {
        Self { t, object }
    }

    pub fn t(&self) -> f64 {
        self.t
    }

    /// Get the material of the object associated with this intersection.
    pub fn material(&self) -> Material {
        self.object.material()
    }

    pub fn object(&self) -> Object {
        self.object.clone()
    }

    pub fn prepare_computations(
        &self,
        r: Ray,
        intersections: Option<&Vec<Itrsectn>>,
    ) -> ItrsectnVs {
        let point = r.position_at(self.t);
        let eyev = -r.direction;
        let normalv = self.object.normal_at(point);
        let inside = eyev.dot(normalv) < 0.0;
        let normalv = if inside { -normalv } else { normalv };
        let over_point = point + normalv * EPSILON;
        let under_point = point - normalv * EPSILON;
        let reflectv = r.direction().reflect(normalv);

        let mut res = ItrsectnVs {
            t: self.t,
            object: self.object.clone(),
            point,
            eyev,
            normalv,
            inside,
            over_point,
            under_point,
            reflectv,
            n1: 1.0,
            n2: 1.0,
        };

        match intersections {
            None => res,
            Some(xs) => {
                // Now compute n1 and n2.
                let mut containers: Vec<Object> = vec![];

                for i in xs {
                    if i == self {
                        // The previous object holds n1.
                        res.n1 = containers
                            .last()
                            .map_or(1.0, |x| x.material().refractive_index())
                    }
                    match containers.iter().position(|x| *x == i.clone().object) {
                        None => {
                            containers.push(i.clone().object);
                        }
                        Some(idx) => {
                            containers.remove(idx);
                        }
                    }
                    if i == self {
                        res.n2 = containers
                            .last()
                            .map_or(1.0, |x| x.material().refractive_index());
                        break;
                    }
                }
                res
            }
        }
    }
}

impl PartialEq for Itrsectn {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && *self.object == *other.object
    }
}

/// Given a list of intersections, finds the intersection with the lowest non-negative t value.
pub fn hit(xs: &Vec<Itrsectn>) -> Option<Itrsectn> {
    let mut res: Option<Itrsectn> = None;
    for x in xs {
        if x.t < 0.0 {
            continue;
        }
        match res {
            None => res = Some(x.clone()),
            Some(y) if x.t < y.t => res = Some(x.clone()),
            _ => (),
        }
    }
    res
}

/// A utility struct with some values related to a point of intersection.
#[derive(Debug)]
pub struct ItrsectnVs {
    pub t: f64,
    pub object: Object,
    pub point: Point,
    pub eyev: Vector,
    pub normalv: Vector,
    /// Whether the intersection is from the inside of an object.
    pub inside: bool,
    /// The original point, but shifted slightly in the direction of the normal vector. This is
    /// needed to prevent fuzzy shadows.
    pub over_point: Point,
    /// A point nudged just slightly below the original point.
    pub under_point: Point,
    /// Direction of the reflected ray.
    pub reflectv: Vector,
    /// Refractive index of the material being exited.
    pub n1: f64,
    /// Refractive index of the material being entered.
    pub n2: f64,
}

/// Computes the reflectance, which is the fraction of light reflected from a surface.
pub fn schlick(comps: ItrsectnVs) -> f64 {
    let mut cos = comps.eyev.dot(comps.normalv);
    if comps.n1 > comps.n2 {
        // We have total internal reflection.
        let n = comps.n1 / comps.n2;
        let sin2_t = n * n * (1.0 - cos * cos);
        if sin2_t > 1.0 {
            return 1.0;
        }

        cos = (1.0 - sin2_t).sqrt();
    }

    let r0 = ((comps.n1 - comps.n2) / (comps.n1 + comps.n2)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos).powi(5)
}

#[cfg(test)]
mod tests {
    use super::{hit, schlick, Itrsectn, Ray};
    use crate::light::Material;
    use crate::shapes::Sphere;
    use crate::transform::Tr;
    use crate::tuple::{Point, Vector};
    use crate::{assert_f64_eq, p, v, EPSILON};
    use std::f64::consts::SQRT_2;
    use std::sync::Arc;

    #[test]
    fn creating_and_querying_ray() {
        let origin = Point::new(1.0, 2.0, 3.0);
        let direction = Vector::new(4.0, 5.0, 6.0);
        let r = Ray::new(origin, direction);

        assert_eq!(r.origin, origin);
        assert_eq!(r.direction, direction);
    }

    #[test]
    fn point_from_distance() {
        let r = Ray::new(Point::new(2.0, 3.0, 4.0), Vector::new(1.0, 0.0, 0.0));

        assert_eq!(r.position_at(0.0), Point::new(2.0, 3.0, 4.0));
        assert_eq!(r.position_at(1.0), Point::new(3.0, 3.0, 4.0));
        assert_eq!(r.position_at(-1.0), Point::new(1.0, 3.0, 4.0));
        assert_eq!(r.position_at(2.5), Point::new(4.5, 3.0, 4.0));
    }

    #[test]
    fn create_intersection() {
        let s = Sphere::default().as_object();
        let i = Itrsectn::new(3.5, s.clone());

        assert_eq!(i.t, 3.5);
        assert_eq!(*i.object, *s);
    }

    // #[test]
    // fn aggregating_intersections() {
    //     let s = Sphere::default();
    //     let i1 = Intersection::new(1.0, &s);
    //     let i2 = Intersection::new(2.0, &s);

    //     let intersections = vec![i1, i2];
    // }

    #[test]
    fn intersect_sets_object() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default().as_object();

        let got = s.intersect_with(r);
        assert_eq!(got.len(), 2);
        assert_eq!(*got[0].object, *s);
        assert_eq!(*got[1].object, *s);
    }

    #[test]
    fn hit_all_intersections_positive() {
        let s = Sphere::default().as_object();
        let i1 = Itrsectn::new(1.0, s.clone());
        let i2 = Itrsectn::new(2.0, s);
        let xs = vec![i1.clone(), i2];

        let got = hit(&xs).unwrap();
        let want = i1;
        assert_eq!(got, want);
    }

    #[test]
    fn hit_some_negative() {
        let s = Sphere::default().as_object();
        let i1 = Itrsectn::new(-1.0, s.clone());
        let i2 = Itrsectn::new(1.0, s);
        let xs = vec![i1, i2.clone()];

        let got = hit(&xs).unwrap();
        let want = i2;
        assert_eq!(got, want);
    }

    #[test]
    fn hit_all_negative() {
        let s = Sphere::default().as_object();
        let i1 = Itrsectn::new(-2.0, s.clone());
        let i2 = Itrsectn::new(-1.0, s);
        let xs = vec![i1, i2];

        let got = hit(&xs);
        assert!(got.is_none());
    }

    #[test]
    fn hit_always_lowest_nonneg() {
        let s = Sphere::default().as_object();
        let i1 = Itrsectn::new(5.0, s.clone());
        let i2 = Itrsectn::new(7.0, s.clone());
        let i3 = Itrsectn::new(-3.0, s.clone());
        let i4 = Itrsectn::new(2.0, s);
        let xs = vec![i1, i2, i3, i4.clone()];

        let got = hit(&xs).unwrap();
        let want = i4;
        assert_eq!(got, want);
    }

    #[test]
    fn translate_a_ray() {
        let r = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.0, 1.0, 0.0));
        let t = Tr::default().translate(3.0, 4.0, 5.0);

        let got = r.with_transform(t);
        assert_eq!(got.origin, Point::new(4.0, 6.0, 8.0));
        assert_eq!(got.direction, Vector::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn scale_a_ray() {
        let r = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.0, 1.0, 0.0));
        let t = Tr::default().scale(2.0, 3.0, 4.0);

        let got = r.with_transform(t);
        assert_eq!(got.origin, Point::new(2.0, 6.0, 12.0));
        assert_eq!(got.direction, Vector::new(0.0, 3.0, 0.0));
    }

    #[test]
    fn precomputing_state_of_an_intersection() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Arc::new(Sphere::default());
        let i = Itrsectn::new(4.0, shape);
        let comps = i.prepare_computations(r, None);
        assert_eq!(comps.t, i.t);
        assert_eq!(*comps.object, *i.object);
        assert_eq!(comps.point, Point::new(0.0, 0.0, -1.0));
        assert_eq!(comps.eyev, Vector::new(0.0, 0.0, -1.0));
        assert_eq!(comps.normalv, Vector::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn when_intersect_occurs_on_outside() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Arc::new(Sphere::default());
        let i = Itrsectn::new(4.0, shape);
        let comps = i.prepare_computations(r, None);
        assert_eq!(comps.inside, false);
    }

    #[test]
    fn when_intersect_occurs_on_inside() {
        let r = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Arc::new(Sphere::default());
        let i = Itrsectn::new(1.0, shape);
        let comps = i.prepare_computations(r, None);
        assert_eq!(comps.point, Point::new(0.0, 0.0, 1.0));
        assert_eq!(comps.eyev, Vector::new(0.0, 0.0, -1.0));
        assert_eq!(comps.inside, true);
        assert_eq!(comps.normalv, Vector::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn hit_should_offset_point() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let shape =
            Arc::new(Sphere::default().with_transform(Tr::default().translate(0.0, 0.0, 1.0)));
        let i = Itrsectn::new(5.0, shape);
        let comps = i.prepare_computations(r, None);

        assert!(comps.over_point.z() < -EPSILON / 2.0);
        assert!(comps.point.z() > comps.over_point.z());
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
    fn under_point_is_offset_below_surface() {
        let r = Ray::new(p!(0, 0, 0), v!(0, 0, 1));
        let shape = glass_sphere()
            .with_transform(Tr::new().translate(0.0, 0.0, 1.0))
            .as_object();
        let i = Itrsectn::new(5.0, shape);

        let comps = i.prepare_computations(r, None);
        assert!(comps.under_point.z() > EPSILON / 2.0);
        assert!(comps.point.z() < comps.under_point.z());
    }

    #[test]
    fn schlick_approx_under_total_internal_reflection() {
        let shape = glass_sphere().as_object();
        let r = Ray::new(p!(0, 0, SQRT_2 / 2.0), v!(0, 1, 0));
        let xs = vec![
            Itrsectn::new(-SQRT_2 / 2.0, shape.clone()),
            Itrsectn::new(SQRT_2 / 2.0, shape.clone()),
        ];
        let comps = xs[1].prepare_computations(r, Some(&xs));
        let reflectance = schlick(comps);
        assert_f64_eq!(reflectance, 1.0);
    }

    #[test]
    fn schlick_approx_with_perpendicular_viewing_angle() {
        let shape = glass_sphere().as_object();
        let r = Ray::new(p!(0, 0, 0), v!(0, 1, 0));
        let xs = vec![
            Itrsectn::new(-1.0, shape.clone()),
            Itrsectn::new(1.0, shape.clone()),
        ];
        let comps = xs[1].prepare_computations(r, Some(&xs));
        let reflectance = schlick(comps);
        assert_f64_eq!(reflectance, 0.04);
    }

    #[test]
    fn schlick_approx_with_small_angle_and_n2_ge_n1() {
        let shape = glass_sphere().as_object();
        let r = Ray::new(p!(0, 0.99, -2), v!(0, 0, 1));
        let xs = vec![Itrsectn::new(1.8589, shape.clone())];
        let comps = xs[0].prepare_computations(r, Some(&xs));
        let reflectance = schlick(comps);
        assert_f64_eq!(reflectance, 0.48873, EPSILON);
    }
}
