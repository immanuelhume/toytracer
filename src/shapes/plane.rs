use crate::light::Material;
use crate::ray::{Intersection, Ray};
use crate::shapes::{Object, Shape};
use crate::transform::Tr;
use crate::tuple::{Point, Vector};
use crate::v;
use crate::{get_uid, EPSILON};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Plane {
    id: usize,
    transform: Tr,
    material: Material,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            id: get_uid(),
            transform: Tr::default(),
            material: Material::default(),
        }
    }
}

impl Shape for Plane {
    fn transform(&self) -> Tr {
        self.transform
    }

    fn set_transform(&mut self, t: Tr) {
        self.transform = t;
    }

    fn material(&self) -> Material {
        self.material.clone()
    }

    fn set_material(&mut self, m: Material) {
        self.material = m;
    }

    fn local_intersect_with(&self, r: Ray) -> Vec<Intersection> {
        if r.direction().y().abs() < EPSILON {
            return vec![];
        }
        let t = -r.origin().y() / r.direction().y();
        vec![Intersection::new(t, Arc::new(self.clone()))]
    }

    fn local_normal_at(&self, _: Point) -> Vector {
        v!(0.0, 1.0, 0.0)
    }

    fn id(&self) -> usize {
        self.id
    }
}

impl Plane {
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
    use super::Plane;
    use crate::ray::Ray;
    use crate::shapes::Shape;
    use crate::tuple::{Point, Vector};
    use crate::{p, v};

    #[test]
    fn normal_of_plane_constant_everywhere() {
        let p = Plane::default();
        let n1 = p.local_normal_at(p!(0.0, 0.0, 0.0));
        let n2 = p.local_normal_at(p!(10.0, 0.0, -0.0));
        let n3 = p.local_normal_at(p!(-5.0, 0.0, 150.0));
        let n = v!(0.0, 1.0, 0.0);
        assert_eq!(n1, n);
        assert_eq!(n2, n);
        assert_eq!(n3, n);
    }

    #[test]
    fn intersect_with_ray_parallel_to_plane() {
        let p = Plane::default();
        let r = Ray::new(p!(0.0, 10.0, 0.0), v!(0.0, 0.0, 1.0));
        let xs = p.local_intersect_with(r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn intersect_with_coplanar_ray() {
        let p = Plane::default();
        let r = Ray::new(p!(0.0, 0.0, 0.0), v!(0.0, 0.0, 1.0));
        let xs = p.local_intersect_with(r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn ray_intersecting_plane_from_above() {
        let p = Plane::default().as_object();
        let r = Ray::new(p!(0.0, 1.0, 0.0), v!(0.0, -1.0, 0.0));
        let xs = p.local_intersect_with(r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t(), 1.0);
        assert_eq!(*xs[0].object(), *p);
    }

    #[test]
    fn ray_intersecting_plane_from_below() {
        let p = Plane::default().as_object();
        let r = Ray::new(p!(0.0, -1.0, 0.0), v!(0.0, 1.0, 0.0));
        let xs = p.local_intersect_with(r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t(), 1.0);
        assert_eq!(*xs[0].object(), *p);
    }
}
