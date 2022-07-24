use crate::get_uid;
use crate::light::Material;
use crate::matrix::Matrix;
use crate::transform::Tr;
use crate::tuple::{Point, Tuple, Vector};

#[derive(Debug, PartialEq)]
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

impl Sphere {
    pub fn with_transform(mut self, t: Tr) -> Self {
        self.transform = t;
        self
    }

    /// Computes the normal at some point on the sphere.
    pub fn normal_at(&self, p: Point) -> Vector {
        let m: Matrix<4, 4> = self.transform.matrix();
        let object_point: Point = m.inverse().unwrap() * p;
        let object_normal = object_point - self.center;
        let Tuple(x, y, z, _) = object_normal.inner();
        let m = m.submatrix(3, 3).inverse().unwrap().transpose() * Matrix::new([[x], [y], [z]]);
        let world_normal = Vector::new(m.get(0, 0), m.get(1, 0), m.get(2, 0));
        world_normal.normalize()
    }

    pub fn set_material(&mut self, m: Material) {
        self.material = m;
    }

    pub fn with_material(mut self, m: Material) -> Self {
        self.material = m;
        self
    }

    pub fn material(&self) -> Material {
        self.material
    }

    pub fn transform(&self) -> Tr {
        self.transform
    }

    pub fn center(&self) -> Point {
        self.center
    }
}
