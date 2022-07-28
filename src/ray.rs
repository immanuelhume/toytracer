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
    pub fn when_intersect_world(&self, w: &World) -> Vec<Intersection> {
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
pub struct Intersection {
    t: f64,
    object: Object,
}

impl Intersection {
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

    pub fn prepare_computations(&self, r: Ray) -> IntersectionVals {
        let point = r.position_at(self.t);
        let eyev = -r.direction;
        let normalv = self.object.normal_at(point);
        let inside = eyev.dot(normalv) < 0.0;
        let normalv = if inside { -normalv } else { normalv };
        let over_point = point + normalv * EPSILON;
        let reflectv = r.direction().reflect(normalv);
        return IntersectionVals {
            t: self.t,
            object: self.object.clone(),
            point,
            eyev,
            normalv,
            inside,
            over_point,
            reflectv,
        };
    }
}

impl PartialEq for Intersection {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && *self.object == *other.object
    }
}

/// Given a list of intersections, finds the intersection with the lowest non-negative t value.
pub fn hit(xs: Vec<Intersection>) -> Option<Intersection> {
    let mut res: Option<Intersection> = None;
    for x in xs {
        if x.t < 0.0 {
            continue;
        }
        match res {
            None => res = Some(x),
            Some(y) if x.t < y.t => res = Some(x),
            _ => (),
        }
    }
    res
}

/// A utility struct with some values related to a point of intersection.
pub struct IntersectionVals {
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
    /// Direction of the reflected ray.
    pub reflectv: Vector,
}

#[cfg(test)]
mod tests {
    use super::{hit, Intersection, Ray};
    use crate::shapes::Sphere;
    use crate::transform::Tr;
    use crate::tuple::{Point, Vector};
    use crate::EPSILON;
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
        let i = Intersection::new(3.5, s.clone());

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
        let i1 = Intersection::new(1.0, s.clone());
        let i2 = Intersection::new(2.0, s);
        let xs = vec![i1.clone(), i2];

        let got = hit(xs).unwrap();
        let want = i1;
        assert_eq!(got, want);
    }

    #[test]
    fn hit_some_negative() {
        let s = Sphere::default().as_object();
        let i1 = Intersection::new(-1.0, s.clone());
        let i2 = Intersection::new(1.0, s);
        let xs = vec![i1, i2.clone()];

        let got = hit(xs).unwrap();
        let want = i2;
        assert_eq!(got, want);
    }

    #[test]
    fn hit_all_negative() {
        let s = Sphere::default().as_object();
        let i1 = Intersection::new(-2.0, s.clone());
        let i2 = Intersection::new(-1.0, s);
        let xs = vec![i1, i2];

        let got = hit(xs);
        assert!(got.is_none());
    }

    #[test]
    fn hit_always_lowest_nonneg() {
        let s = Sphere::default().as_object();
        let i1 = Intersection::new(5.0, s.clone());
        let i2 = Intersection::new(7.0, s.clone());
        let i3 = Intersection::new(-3.0, s.clone());
        let i4 = Intersection::new(2.0, s);
        let xs = vec![i1, i2, i3, i4.clone()];

        let got = hit(xs).unwrap();
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
        let i = Intersection::new(4.0, shape);
        let comps = i.prepare_computations(r);
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
        let i = Intersection::new(4.0, shape);
        let comps = i.prepare_computations(r);
        assert_eq!(comps.inside, false);
    }

    #[test]
    fn when_intersect_occurs_on_inside() {
        let r = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Arc::new(Sphere::default());
        let i = Intersection::new(1.0, shape);
        let comps = i.prepare_computations(r);
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
        let i = Intersection::new(5.0, shape);
        let comps = i.prepare_computations(r);

        assert!(comps.over_point.z() < -EPSILON / 2.0);
        assert!(comps.point.z() > comps.over_point.z());
    }
}
