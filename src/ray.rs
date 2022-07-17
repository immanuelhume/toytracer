use crate::get_uid;
use crate::light::Material;
use crate::matrix::Matrix;
use crate::tuple::{Point, Tuple, Vector};
use std::f64::EPSILON;

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

    pub fn when_intersect_sphere<'a>(
        &self,
        s: &'a Sphere,
    ) -> Option<(Intersection<'a>, Intersection<'a>)> {
        let ray = self.transform(s.transform.inverse().expect("could not transform ray"));
        let sphere_to_ray = ray.origin - s.center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * ray.direction.dot(sphere_to_ray);
        let c = sphere_to_ray.dot(sphere_to_ray) - 1.0;
        let mut discr = b * b - 4.0 * a * c;
        if discr < -EPSILON {
            return None;
        }
        discr = if discr.abs() < EPSILON { 0.0 } else { discr };
        let t1 = (-b - discr.sqrt()) / (2.0 * a);
        let t2 = (-b + discr.sqrt()) / (2.0 * a);
        Some((Intersection::new(t1, s), Intersection::new(t2, s)))
    }

    fn transform(&self, m: Matrix<4, 4>) -> Self {
        Self {
            origin: m * self.origin,
            direction: m * self.direction,
        }
    }

    pub fn direction(&self) -> Vector {
        self.direction
    }
}

#[derive(Debug, PartialEq)]
pub struct Sphere {
    id: usize,
    center: Point,
    transform: Matrix<4, 4>,
    material: Material,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            id: get_uid(),
            center: Point::new(0.0, 0.0, 0.0),
            transform: Matrix::<4, 4>::ident(),
            material: Material::default(),
        }
    }
}

impl Sphere {
    pub fn set_transform(&mut self, m: Matrix<4, 4>) {
        self.transform = m;
    }

    /// Computes the normal at some point on the sphere.
    pub fn normal_at(&self, p: Point) -> Vector {
        let object_point = self.transform.inverse().unwrap() * p;
        let object_normal = object_point - self.center;
        let Tuple(x, y, z, _) = object_normal.inner();
        let m = self
            .transform
            .submatrix(3, 3)
            .inverse()
            .unwrap()
            .transpose()
            * Matrix::new([[x], [y], [z]]);
        let world_normal = Vector::new(m.get(0, 0), m.get(1, 0), m.get(2, 0));
        world_normal.normalize()
    }

    pub fn set_material(&mut self, m: Material) {
        self.material = m;
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Intersection<'a> {
    t: f64,
    object: &'a Sphere,
}

impl<'a> Intersection<'a> {
    fn new(t: f64, object: &'a Sphere) -> Self {
        Self { t, object }
    }

    pub fn t(&self) -> f64 {
        self.t
    }

    /// Get the material of the object associated with this intersection.
    pub fn material(&self) -> Material {
        self.object.material
    }

    pub fn object(&self) -> &Sphere {
        self.object
    }
}

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

#[cfg(test)]
mod tests {
    use super::{hit, Intersection, Ray, Sphere};
    use crate::light::Material;
    use crate::matrix::Matrix;
    use crate::transformation::{rotation_z, scaling, translation};
    use crate::tuple::{Point, Vector};
    use std::f64::consts::FRAC_PI_4;

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
    fn ray_intersecting_spheres() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let (a, b) = r.when_intersect_sphere(&s).unwrap();
        let got = (a.t, b.t);
        let want = (4.0, 6.0);
        assert_eq!(got, want);
    }

    #[test]
    fn ray_intersects_sphere_at_tangent() {
        let r = Ray::new(Point::new(0.0, 1.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let (a, b) = r.when_intersect_sphere(&s).unwrap();
        let got = (a.t, b.t);
        let want = (5.0, 5.0);
        assert_eq!(got, want);
    }

    #[test]
    fn ray_misses_sphere() {
        let r = Ray::new(Point::new(0.0, 2.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let got = r.when_intersect_sphere(&s);
        let want = None;
        assert_eq!(got, want);
    }

    #[test]
    fn ray_inside_sphere() {
        let r = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let (a, b) = r.when_intersect_sphere(&s).unwrap();
        let got = (a.t, b.t);
        let want = (-1.0, 1.0);
        assert_eq!(got, want);
    }

    #[test]
    fn sphere_behind_ray() {
        let r = Ray::new(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let (a, b) = r.when_intersect_sphere(&s).unwrap();
        let got = (a.t, b.t);
        let want = (-6.0, -4.0);
        assert_eq!(got, want);
    }

    #[test]
    fn create_intersection() {
        let s = Sphere::default();
        let i = Intersection::new(3.5, &s);

        assert_eq!(i.t, 3.5);
        assert_eq!(*i.object, s);
    }

    #[test]
    fn aggregating_intersections() {
        let s = Sphere::default();
        let i1 = Intersection::new(1.0, &s);
        let i2 = Intersection::new(2.0, &s);

        let intersections = vec![i1, i2];
    }

    #[test]
    fn intersect_sets_object() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let got = r.when_intersect_sphere(&s).unwrap();
        assert_eq!(*got.0.object, s);
        assert_eq!(*got.1.object, s);
    }

    #[test]
    fn hit_all_intersections_positive() {
        let s = Sphere::default();
        let i1 = Intersection::new(1.0, &s);
        let i2 = Intersection::new(2.0, &s);
        let xs = vec![i1, i2];

        let got = hit(xs).unwrap();
        let want = i1;
        assert_eq!(got, want);
    }

    #[test]
    fn hit_some_negative() {
        let s = Sphere::default();
        let i1 = Intersection::new(-1.0, &s);
        let i2 = Intersection::new(1.0, &s);
        let xs = vec![i1, i2];

        let got = hit(xs).unwrap();
        let want = i2;
        assert_eq!(got, want);
    }

    #[test]
    fn hit_all_negative() {
        let s = Sphere::default();
        let i1 = Intersection::new(-2.0, &s);
        let i2 = Intersection::new(-1.0, &s);
        let xs = vec![i1, i2];

        let got = hit(xs);
        assert!(got.is_none());
    }

    #[test]
    fn hit_always_lowest_nonneg() {
        let s = Sphere::default();
        let i1 = Intersection::new(5.0, &s);
        let i2 = Intersection::new(7.0, &s);
        let i3 = Intersection::new(-3.0, &s);
        let i4 = Intersection::new(2.0, &s);
        let xs = vec![i1, i2, i3, i4];

        let got = hit(xs).unwrap();
        let want = i4;
        assert_eq!(got, want);
    }

    #[test]
    fn translate_a_ray() {
        let r = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.0, 1.0, 0.0));
        let m = translation(3.0, 4.0, 5.0);

        let got = r.transform(m);
        assert_eq!(got.origin, Point::new(4.0, 6.0, 8.0));
        assert_eq!(got.direction, Vector::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn scale_a_ray() {
        let r = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.0, 1.0, 0.0));
        let m = scaling(2.0, 3.0, 4.0);

        let got = r.transform(m);
        assert_eq!(got.origin, Point::new(2.0, 6.0, 12.0));
        assert_eq!(got.direction, Vector::new(0.0, 3.0, 0.0));
    }

    #[test]
    fn sphere_default_transform() {
        let s = Sphere::default();
        assert_eq!(s.transform, Matrix::<4, 4>::ident());
    }

    #[test]
    fn changing_sphere_transformation() {
        let mut s = Sphere::default();
        let t = translation(2.0, 3.0, 4.0);
        s.set_transform(t);
        assert_eq!(s.transform, t);
    }

    #[test]
    fn intersecting_scaled_sphere_with_ray() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let mut s = Sphere::default();
        let t = scaling(2.0, 2.0, 2.0);
        s.set_transform(t);

        let (a, b) = r.when_intersect_sphere(&s).unwrap();
        assert_eq!(a.t, 3.0);
        assert_eq!(b.t, 7.0);
    }

    #[test]
    fn intersecting_translated_sphere_with_ray() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let mut s = Sphere::default();
        let t = translation(5.0, 0.0, 0.0);
        s.set_transform(t);

        let got = r.when_intersect_sphere(&s);
        assert!(got.is_none());
    }

    #[test]
    fn normals_on_sphere() {
        let s = Sphere::default();
        let tests = vec![
            (Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0)),
            (Point::new(0.0, 1.0, 0.0), Vector::new(0.0, 1.0, 0.0)),
            (Point::new(0.0, 0.0, 1.0), Vector::new(0.0, 0.0, 1.0)),
            (
                Point::new(
                    3.0_f64.sqrt() / 3.0,
                    3.0_f64.sqrt() / 3.0,
                    3.0_f64.sqrt() / 3.0,
                ),
                Vector::new(
                    3.0_f64.sqrt() / 3.0,
                    3.0_f64.sqrt() / 3.0,
                    3.0_f64.sqrt() / 3.0,
                ),
            ),
        ];

        for test in tests {
            let (p, want) = test;
            let got = s.normal_at(p);
            assert_eq!(got, want);
        }
    }

    #[test]
    fn normal_on_translated_sphere() {
        let mut s = Sphere::default();
        s.set_transform(translation(0.0, 1.0, 0.0));

        let got = s.normal_at(Point::new(0.0, 1.70711, -0.70711));
        let want = Vector::new(0.0, 0.70711, -0.70711);
        assert_eq!(got, want);
    }

    #[test]
    fn normal_on_transformed_sphere() {
        let mut s = Sphere::default();
        let m = scaling(1.0, 0.5, 1.0) * rotation_z(FRAC_PI_4);
        s.set_transform(m);

        let got = s.normal_at(Point::new(0.0, 2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0));
        let want = Vector::new(0.0, 0.97014, -0.24253);
        assert_eq!(got, want);
    }

    #[test]
    fn sphere_has_a_default_material() {
        let s = Sphere::default();
        assert_eq!(s.material, Material::default());
    }
}
