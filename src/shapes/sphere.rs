use crate::light::Material;
use crate::ray::{Intersection, Ray};
use crate::shapes::{Object, Shape};
use crate::transform::Tr;
use crate::tuple::{Point, Vector};
use crate::{get_uid, EPSILON};
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
pub struct Sphere {
    id: usize,
    center: Point,
    transform: Tr,
    material: Material,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            id: get_uid(),
            center: Point::new(0.0, 0.0, 0.0),
            transform: Tr::default(),
            material: Material::default(),
        }
    }
}

impl Shape for Sphere {
    fn transform(&self) -> Tr {
        self.transform
    }

    fn material(&self) -> Material {
        self.material.clone()
    }

    fn local_intersect_with(&self, r: Ray) -> Vec<Intersection> {
        let sphere_to_ray = r.origin() - self.center;
        let a = r.direction().dot(r.direction());
        let b = 2.0 * r.direction().dot(sphere_to_ray);
        let c = sphere_to_ray.dot(sphere_to_ray) - 1.0;
        let mut discr = b * b - 4.0 * a * c;
        if discr < -EPSILON {
            return vec![];
        }
        discr = if discr.abs() < EPSILON { 0.0 } else { discr };
        let t1 = (-b - discr.sqrt()) / (2.0 * a);
        let t2 = (-b + discr.sqrt()) / (2.0 * a);
        let s = self.clone().as_object();
        vec![
            Intersection::new(t1, s.clone()),
            Intersection::new(t2, s.clone()),
        ]
    }

    fn local_normal_at(&self, p: Point) -> Vector {
        p - self.center
    }

    fn id(&self) -> usize {
        self.id
    }

    fn set_transform(&mut self, t: Tr) {
        self.transform = t;
    }

    fn set_material(&mut self, m: Material) {
        self.material = m;
    }
}

impl Sphere {
    pub fn center(&self) -> Point {
        self.center
    }

    pub fn with_transform(mut self, t: Tr) -> Self {
        self.transform = t;
        self
    }

    pub fn with_material(mut self, m: Material) -> Self {
        self.material = m;
        self
    }

    pub fn as_object(self) -> Object {
        Arc::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::Sphere;
    use crate::light::Material;
    use crate::ray::Ray;
    use crate::shapes::Shape;
    use crate::transform::Tr;
    use crate::tuple::{Point, Vector};
    use std::f64::consts::FRAC_PI_4;

    #[test]
    fn ray_intersecting_spheres() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let got: Vec<f64> = s.intersect_with(r).into_iter().map(|x| x.t()).collect();
        let want = vec![4.0, 6.0];
        assert_eq!(got, want);
    }

    #[test]
    fn ray_intersects_sphere_at_tangent() {
        let r = Ray::new(Point::new(0.0, 1.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let got: Vec<f64> = s.intersect_with(r).into_iter().map(|x| x.t()).collect();
        let want = vec![5.0, 5.0];
        assert_eq!(got, want);
    }

    #[test]
    fn ray_misses_sphere() {
        let r = Ray::new(Point::new(0.0, 2.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let got: Vec<f64> = s.intersect_with(r).into_iter().map(|x| x.t()).collect();
        let want = vec![];
        assert_eq!(got, want);
    }

    #[test]
    fn ray_inside_sphere() {
        let r = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let got: Vec<f64> = s.intersect_with(r).into_iter().map(|x| x.t()).collect();
        let want = vec![-1.0, 1.0];
        assert_eq!(got, want);
    }

    #[test]
    fn sphere_behind_ray() {
        let r = Ray::new(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default();

        let got: Vec<f64> = s.intersect_with(r).into_iter().map(|x| x.t()).collect();
        let want = vec![-6.0, -4.0];
        assert_eq!(got, want);
    }

    #[test]
    fn sphere_default_transform() {
        let s = Sphere::default();
        assert_eq!(s.transform(), Tr::default());
    }

    #[test]
    fn changing_sphere_transformation() {
        let t = Tr::default().translate(2.0, 3.0, 4.0);
        let s = Sphere::default().with_transform(t);
        assert_eq!(s.transform(), t);
    }

    #[test]
    fn intersecting_scaled_sphere_with_ray() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default().with_transform(Tr::default().scale(2.0, 2.0, 2.0));

        let got: Vec<f64> = s.intersect_with(r).into_iter().map(|x| x.t()).collect();
        let want = vec![3.0, 7.0];
        assert_eq!(got, want);
    }

    #[test]
    fn intersecting_translated_sphere_with_ray() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = Sphere::default().with_transform(Tr::default().translate(5.0, 0.0, 0.0));

        let got: Vec<f64> = s.intersect_with(r).into_iter().map(|x| x.t()).collect();
        let want = vec![];
        assert_eq!(got, want);
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
        let s = Sphere::default().with_transform(Tr::default().translate(0.0, 1.0, 0.0));

        let got = s.normal_at(Point::new(0.0, 1.70711, -0.70711));
        let want = Vector::new(0.0, 0.70711, -0.70711);
        assert_eq!(got, want);
    }

    #[test]
    fn normal_on_transformed_sphere() {
        let s = Sphere::default()
            .with_transform(Tr::default().rotate_z(FRAC_PI_4).scale(1.0, 0.5, 1.0));

        let got = s.normal_at(Point::new(0.0, 2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0));
        let want = Vector::new(0.0, 0.97014, -0.24253);
        assert_eq!(got, want);
    }

    #[test]
    fn sphere_has_a_default_material() {
        let s = Sphere::default();
        assert_eq!(s.material(), Material::default());
    }
}
