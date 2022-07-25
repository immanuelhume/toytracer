use crate::color::Color;
use crate::shapes::Shape;
use crate::transform::Tr;
use crate::tuple::Point;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

pub type PatternX = Arc<Option<Box<dyn Pattern>>>;
pub trait Pattern: Send + Sync + Any + Debug {
    fn color_at(&self, p: Point) -> Color;
    /// Given a shape and a point on that shape (in world space), returns the correct color for
    /// that point.
    fn color_at_object(&self, s: &dyn Shape, p: Point) -> Color {
        let object_p = s.transform().inverse().matrix() * p; // the point, in object space
        let pattern_p = self.transform().inverse().matrix() * object_p; // the point, in pattern space
        self.color_at(pattern_p)
    }
    /// Converts to the any trait object.
    fn as_any(&self) -> &dyn Any;
    /// Takes an arbitrary trait object and attempts to downcast it, then check equality.
    fn eqx(&self, other: &dyn Any) -> bool;
    fn as_box(self) -> Box<dyn Pattern>;
    fn transform(&self) -> Tr;
    fn set_transform(&mut self, t: Tr);
}

impl PartialEq for dyn Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.eqx(other.as_any())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stripe {
    a: Color,
    b: Color,

    transform: Tr,
}

impl Stripe {
    pub fn new(a: Color, b: Color) -> Self {
        Self {
            a,
            b,
            transform: Tr::default(),
        }
    }

    pub fn with_transform(mut self, t: Tr) -> Self {
        self.transform = t;
        self
    }
}

impl Pattern for Stripe {
    fn color_at(&self, p: Point) -> Color {
        if p.x().floor() % 2.0 == 0.0 {
            self.a
        } else {
            self.b
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eqx(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>().map_or(false, |a| a == self)
    }

    fn as_box(self) -> Box<dyn Pattern> {
        Box::new(self)
    }

    fn transform(&self) -> Tr {
        self.transform
    }

    fn set_transform(&mut self, t: Tr) {
        self.transform = t;
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;
    use super::Stripe;
    use crate::color::Color;
    use crate::p;
    use crate::shapes::Sphere;
    use crate::transform::Tr;
    use crate::tuple::Point;

    #[test]
    fn creating_a_stripe_pattern() {
        let pattern = Stripe::new(Color::white(), Color::black());
        assert_eq!(pattern.a, Color::white());
        assert_eq!(pattern.b, Color::black());
    }

    macro_rules! test_stripe {
        ($($name:ident: $($point:expr, $color:expr,)+;)*) => {
            $(
            #[test]
            fn $name() {
                let pattern = Stripe::new(Color::white(), Color::black());
                $(assert_eq!(pattern.color_at($point), $color);)*
            }
            )*
        };
    }

    test_stripe! {
        stripe_is_constant_in_y:
            p!(0.0, 0.0, 0.0), Color::white(),
            p!(0.0, 1.0, 0.0), Color::white(),
            p!(0.0, 2.0, 0.0), Color::white(),
        ;
        stripe_is_constant_in_z:
            p!(0.0, 0.0, 0.0), Color::white(),
            p!(0.0, 0.0, 1.0), Color::white(),
            p!(0.0, 0.0, 2.0), Color::white(),
        ;
        stripe_alternates_in_x:
            p!(0.0, 0.0, 0.0), Color::white(),
            p!(0.9, 0.0, 0.0), Color::white(),
            p!(1.0, 0.0, 0.0), Color::black(),
            p!(-0.1, 0.0, 0.0), Color::black(),
            p!(-1.0, 0.0, 0.0), Color::black(),
            p!(-1.1, 0.0, 0.0), Color::white(),
        ;
    }

    #[test]
    fn stripes_with_an_object_transformation() {
        let object = Sphere::default().with_transform(Tr::default().scale(2.0, 2.0, 2.0));
        let pattern = Stripe::new(Color::white(), Color::black());

        let c = pattern.color_at_object(&object, p!(1.5, 0.0, 0.0));
        assert_eq!(c, Color::white());
    }

    #[test]
    fn stripes_with_a_pattern_transformation() {
        let object = Sphere::default();
        let pattern = Stripe::new(Color::white(), Color::black())
            .with_transform(Tr::default().scale(2.0, 2.0, 2.0));

        let c = pattern.color_at_object(&object, p!(1.5, 0.0, 0.0));
        assert_eq!(c, Color::white());
    }

    #[test]
    fn stripes_with_object_and_pattern_transformation() {
        let object = Sphere::default().with_transform(Tr::default().scale(2.0, 2.0, 2.0));
        let pattern = Stripe::new(Color::white(), Color::black())
            .with_transform(Tr::default().translate(0.5, 0.0, 0.0));

        let c = pattern.color_at_object(&object, p!(2.5, 0.0, 0.0));
        assert_eq!(c, Color::white());
    }
}
