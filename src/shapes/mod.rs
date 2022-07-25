mod plane;
mod sphere;
pub use plane::Plane;
pub use sphere::Sphere;

use crate::light::Material;
use crate::matrix::Matrix;
use crate::ray::{Intersection, Ray};
use crate::transform::Tr;
use crate::tuple::{Point, Tuple, Vector};
use std::fmt::Debug;
use std::sync::Arc;

/// An object is how we actually represent and use a shape in scenes.
pub type Object = Arc<dyn Shape>;
pub trait Shape: Send + Sync + Debug {
    fn transform(&self) -> Tr;
    fn set_transform(&mut self, t: Tr);
    fn material(&self) -> Material;
    fn set_material(&mut self, m: Material);

    /// Finds the intersections that some ray has with this shape. Note that this method should not
    /// be implemented manually. Instead, implement only `local_intersect_with`.
    fn intersect_with(&self, r: Ray) -> Vec<Intersection> {
        let r = r.with_transform(self.transform().inverse());
        self.local_intersect_with(r)
    }
    /// Finds the intersections that some *normalized* ray has with this shape.
    fn local_intersect_with(&self, r: Ray) -> Vec<Intersection>;

    /// Finds the normal vector at some point on the surface of this shape. Note that this method
    /// should not be implemented manually. Instead, implement only `local_normal_at`.
    fn normal_at(&self, p: Point) -> Vector {
        let local_point = self.transform().inverse().matrix() * p;
        let local_normal = self.local_normal_at(local_point);
        let Tuple(x, y, z, _) = local_normal.inner();
        let m = self
            .transform()
            .matrix()
            .submatrix(3, 3)
            .inverse()
            .unwrap()
            .transpose()
            * Matrix::new([[x], [y], [z]]);
        let world_normal = Vector::new(m.get(0, 0), m.get(1, 0), m.get(2, 0));
        world_normal.normalize()
    }
    /// Finds the normal vector at some point, where the point is given in object space.
    fn local_normal_at(&self, p: Point) -> Vector;

    /// Every shape must have a unique ID.
    fn id(&self) -> usize;
}

impl PartialEq for dyn Shape {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[cfg(test)]
mod tests {
    use super::Shape;
    use crate::get_uid;
    use crate::light::Material;
    use crate::ray::{Intersection, Ray};
    use crate::transform::Tr;
    use crate::tuple::{Point, Vector};
    use std::f64::consts::PI;
    use std::sync::Mutex;

    /// Just a struct used to test the Shape trait.
    #[derive(Debug)]
    struct TestShape {
        id: usize,
        transform: Tr,
        material: Material,
        /// The transformed ray when computing an intersection. Just used to test internal
        /// behavior.
        saved_ray: Mutex<Option<Ray>>,
    }

    impl TestShape {
        fn new() -> Self {
            Self {
                id: get_uid(),
                transform: Tr::default(),
                material: Material::default(),
                saved_ray: Mutex::new(None),
            }
        }

        fn with_transform(mut self, t: Tr) -> Self {
            self.transform = t;
            self
        }

        fn with_material(mut self, m: Material) -> Self {
            self.material = m;
            self
        }
    }

    impl Shape for TestShape {
        fn transform(&self) -> Tr {
            self.transform
        }

        fn material(&self) -> Material {
            self.material.clone()
        }

        fn local_intersect_with(&self, r: Ray) -> Vec<Intersection> {
            *self.saved_ray.lock().unwrap() = Some(r);
            vec![]
        }

        fn local_normal_at(&self, p: Point) -> Vector {
            Vector::new(p.x(), p.y(), p.z())
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

    #[test]
    fn the_default_transformation() {
        let s = TestShape::new();
        assert_eq!(s.transform(), Tr::default());
    }

    #[test]
    fn assigning_a_transformation() {
        let t = Tr::default().translate(2.0, 3.0, 4.0);
        let s = TestShape::new().with_transform(t);
        assert_eq!(s.transform(), t);
    }

    #[test]
    fn the_default_material() {
        let s = TestShape::new();
        let m = s.material();
        assert_eq!(m, Material::default());
    }

    #[test]
    fn assigning_a_material() {
        let m = Material::default().with_ambient(1.0);
        let s = TestShape::new().with_material(m.clone());
        let got = s.material();
        assert_eq!(got, m);
    }

    #[test]
    fn intersect_a_scaled_shape_with_ray() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = TestShape::new().with_transform(Tr::default().scale(2.0, 2.0, 2.0));

        let _ = s.intersect_with(r);
        let r = s.saved_ray.lock().unwrap().unwrap();
        assert_eq!(r.origin(), Point::new(0.0, 0.0, -2.5));
        assert_eq!(r.direction(), Vector::new(0.0, 0.0, 0.5));
    }

    #[test]
    fn intersect_a_translated_shape_with_ray() {
        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let s = TestShape::new().with_transform(Tr::default().translate(5.0, 0.0, 0.0));

        let _ = s.intersect_with(r);
        let r = s.saved_ray.lock().unwrap().unwrap();
        assert_eq!(r.origin(), Point::new(-5.0, 0.0, -5.0));
        assert_eq!(r.direction(), Vector::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn computing_normal_on_translated_shape() {
        let s = TestShape::new().with_transform(Tr::default().translate(0.0, 1.0, 0.0));
        let got = s.normal_at(Point::new(0.0, 1.70711, -0.70711));
        let want = Vector::new(0.0, 0.70711, -0.70711);
        assert_eq!(got, want);
    }

    #[test]
    fn computing_normal_on_transformed_shape() {
        let s =
            TestShape::new().with_transform(Tr::default().rotate_z(PI / 5.0).scale(1.0, 0.5, 1.0));
        let got = s.normal_at(Point::new(0.0, 2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0));
        let want = Vector::new(0.0, 0.97014, -0.24254);
        assert_eq!(got, want);
    }
}
