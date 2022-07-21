use crate::canvas::Canvas;
use crate::matrix::Matrix;
use crate::ray::Ray;
use crate::tuple::Point;
use crate::world::World;

struct Camera {
    hsize: usize,
    vsize: usize,
    field_of_view: f64,
    transform: Matrix<4, 4>,

    half_width: f64,
    half_height: f64,
    /// The width of one square pixel.
    pixel_size: f64,
}

impl Camera {
    fn new(hsize: usize, vsize: usize, field_of_view: f64) -> Self {
        let half_view = (field_of_view / 2.0).tan();
        let aspect = hsize as f64 / vsize as f64;
        let (half_width, half_height);
        if aspect >= 1.0 {
            half_width = half_view;
            half_height = half_view / aspect;
        } else {
            half_width = half_view * aspect;
            half_height = half_view;
        }
        let pixel_size = (2.0 * half_width) / hsize as f64;
        Self {
            hsize,
            vsize,
            field_of_view,
            transform: Matrix::<4, 4>::ident(),
            half_width,
            half_height,
            pixel_size,
        }
    }

    fn ray_for_pixel(&self, x: usize, y: usize) -> Ray {
        let xoffset = (x as f64 + 0.5) * self.pixel_size;
        let yoffset = (y as f64 + 0.5) * self.pixel_size;

        let world_x = self.half_width - xoffset;
        let world_y = self.half_height - yoffset;

        let transform = self.transform.inverse().unwrap();
        let pixel = transform * Point::new(world_x, world_y, -1.0);
        let origin = transform * Point::origin();
        let direction = (pixel - origin).normalize();

        Ray::new(origin, direction)
    }

    fn set_transform(&mut self, transform: Matrix<4, 4>) {
        self.transform = transform;
    }

    fn render(&self, world: World) -> Canvas {
        let mut image = Canvas::new(self.hsize as usize, self.vsize as usize);
        for y in 0..self.vsize {
            for x in 0..self.hsize {
                let ray = self.ray_for_pixel(x, y);
                let color = world.color_at(ray);
                image.draw(x, y, color);
            }
        }
        image
    }
}

#[cfg(test)]
mod tests {
    use super::Camera;
    use crate::assert_f64_eq;
    use crate::color::Color;
    use crate::matrix::Matrix;
    use crate::transformation::{rotation_y, translation, view_transform};
    use crate::tuple::{Point, Vector};
    use crate::world::World;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

    #[test]
    fn constructing_a_camera() {
        let hsize = 160;
        let vsize = 120;
        let field_of_view = FRAC_PI_2;

        let c = Camera::new(hsize, vsize, field_of_view);
        assert_eq!(c.hsize, hsize);
        assert_eq!(c.vsize, vsize);
        assert_eq!(c.field_of_view, field_of_view);
        assert_eq!(c.transform, Matrix::<4, 4>::ident());
    }

    #[test]
    fn pixel_size_horizontal_canvas() {
        let c = Camera::new(200, 125, FRAC_PI_2);
        assert_f64_eq!(c.pixel_size, 0.01);
    }

    #[test]
    fn pixel_size_vertical_canvas() {
        let c = Camera::new(125, 200, FRAC_PI_2);
        assert_f64_eq!(c.pixel_size, 0.01);
    }

    #[test]
    fn construct_ray_through_center_of_canvas() {
        let c = Camera::new(201, 101, FRAC_PI_2);
        let r = c.ray_for_pixel(100, 50);

        assert_eq!(r.origin(), Point::new(0.0, 0.0, 0.0));
        assert_eq!(r.direction(), Vector::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn construct_ray_through_corner_of_canvas() {
        let c = Camera::new(201, 101, FRAC_PI_2);
        let r = c.ray_for_pixel(0, 0);

        assert_eq!(r.origin(), Point::new(0.0, 0.0, 0.0));
        assert_eq!(r.direction(), Vector::new(0.66519, 0.33259, -0.66851));
    }

    #[test]
    fn construct_ray_through_transformed_camera() {
        let mut c = Camera::new(201, 101, FRAC_PI_2);
        c.set_transform(rotation_y(FRAC_PI_4) * translation(0.0, -2.0, 5.0));
        let r = c.ray_for_pixel(100, 50);

        assert_eq!(r.origin(), Point::new(0.0, 2.0, -5.0));
        assert_eq!(
            r.direction(),
            Vector::new(2.0_f64.sqrt() / 2.0, 0.0, -2.0_f64.sqrt() / 2.0)
        );
    }

    #[test]
    fn rendering_a_world_with_camera() {
        let w = World::default();
        let mut c = Camera::new(11, 11, FRAC_PI_2);
        let from = Point::new(0.0, 0.0, -5.0);
        let to = Point::origin();
        let up = Vector::new(0.0, 1.0, 0.0);
        c.set_transform(view_transform(from, to, up));

        let got = c.render(w);
        assert_eq!(got.pixel_at(5, 5), Color::new(0.38066, 0.47583, 0.2855));
    }
}
