use crate::color::Color;
use crate::shapes::Shape;
use crate::transform::Tr;
use crate::tuple::Point;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

/// Trait object for a pattern. But wrapped in some shit so we can send it across threads.
pub type PatternX = Arc<Option<Box<dyn Pattern>>>;
pub trait Pattern: Send + Sync + Any + Debug {
    fn color_at(&self, p: Point) -> Color;
    /// Given a shape and a point on that shape (in world space), returns the correct color for
    /// that point. This method should not be implemented manually.
    fn color_on_object(&self, s: &dyn Shape, p: Point) -> Color {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gradient {
    a: Color,
    b: Color,
    transform: Tr,
}

impl Gradient {
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

impl Pattern for Gradient {
    fn color_at(&self, p: Point) -> Color {
        let d = self.b - self.a;
        let f = p.x() - p.x().floor();
        self.a + d * f
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    a: Color,
    b: Color,
    transform: Tr,
}

impl Ring {
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

impl Pattern for Ring {
    fn color_at(&self, p: Point) -> Color {
        if p.x().hypot(p.z()).floor() % 2.0 == 0.0 {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Checkers {
    a: Color,
    b: Color,
    transform: Tr,
}

impl Checkers {
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

impl Pattern for Checkers {
    fn color_at(&self, p: Point) -> Color {
        if (p.x().floor() + p.y().floor() + p.z().floor()) % 2.0 == 0.0 {
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
    use super::{Checkers, Gradient, Pattern, Ring, Stripe};
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

        let c = pattern.color_on_object(&object, p!(1.5, 0.0, 0.0));
        assert_eq!(c, Color::white());
    }

    #[test]
    fn stripes_with_a_pattern_transformation() {
        let object = Sphere::default();
        let pattern = Stripe::new(Color::white(), Color::black())
            .with_transform(Tr::default().scale(2.0, 2.0, 2.0));

        let c = pattern.color_on_object(&object, p!(1.5, 0.0, 0.0));
        assert_eq!(c, Color::white());
    }

    #[test]
    fn stripes_with_object_and_pattern_transformation() {
        let object = Sphere::default().with_transform(Tr::default().scale(2.0, 2.0, 2.0));
        let pattern = Stripe::new(Color::white(), Color::black())
            .with_transform(Tr::default().translate(0.5, 0.0, 0.0));

        let c = pattern.color_on_object(&object, p!(2.5, 0.0, 0.0));
        assert_eq!(c, Color::white());
    }

    #[derive(Debug)]
    struct TestPattern {
        transform: Tr,
    }

    impl TestPattern {
        fn new() -> Self {
            Self {
                transform: Tr::default(),
            }
        }
    }

    impl Pattern for TestPattern {
        fn color_at(&self, p: Point) -> Color {
            Color::new(p.x(), p.y(), p.z())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            todo!()
        }

        fn eqx(&self, _: &dyn std::any::Any) -> bool {
            todo!()
        }

        fn as_box(self) -> Box<dyn Pattern> {
            todo!()
        }

        fn transform(&self) -> Tr {
            self.transform
        }

        fn set_transform(&mut self, t: Tr) {
            self.transform = t;
        }
    }

    #[test]
    fn default_pattern_transformation() {
        let pattern = &TestPattern::new() as &dyn Pattern;
        assert_eq!(pattern.transform(), Tr::default());
    }

    #[test]
    fn assigning_a_transformation() {
        let pattern = &mut TestPattern::new() as &mut dyn Pattern;
        let t = Tr::default().translate(1.0, 2.0, 3.0);
        pattern.set_transform(t);
        assert_eq!(pattern.transform(), t);
    }

    #[test]
    fn pattern_with_object_transformation() {
        let pattern = &TestPattern::new() as &dyn Pattern;
        let object = Sphere::default().with_transform(Tr::default().scale(2.0, 2.0, 2.0));

        let got = pattern.color_on_object(&object, p!(2.0, 3.0, 4.0));
        let want = Color::new(1.0, 1.5, 2.0);
        assert_eq!(got, want);
    }

    #[test]
    fn pattern_with_pattern_transformation() {
        let pattern = &mut TestPattern::new() as &mut dyn Pattern;
        pattern.set_transform(Tr::default().scale(2.0, 2.0, 2.0));
        let object = Sphere::default();

        let got = pattern.color_on_object(&object, p!(2.0, 3.0, 4.0));
        let want = Color::new(1.0, 1.5, 2.0);
        assert_eq!(got, want);
    }

    #[test]
    fn pattern_with_both_object_and_pattern_transformation() {
        let shape = Sphere::default().with_transform(Tr::default().scale(2.0, 2.0, 2.0));
        let pattern = &mut TestPattern::new() as &mut dyn Pattern;
        pattern.set_transform(Tr::default().translate(0.5, 1.0, 1.5));

        let got = pattern.color_on_object(&shape, p!(2.5, 3.0, 3.5));
        let want = Color::new(0.75, 0.5, 0.25);
        assert_eq!(got, want);
    }

    #[test]
    fn gradient_linearly_interpolates_between_colors() {
        let pattern = Gradient::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(p!(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(
            pattern.color_at(p!(0.25, 0.0, 0.0)),
            Color::new(0.75, 0.75, 0.75)
        );
        assert_eq!(
            pattern.color_at(p!(0.5, 0.0, 0.0)),
            Color::new(0.5, 0.5, 0.5)
        );
        assert_eq!(
            pattern.color_at(p!(0.75, 0.0, 0.0)),
            Color::new(0.25, 0.25, 0.25)
        );
    }

    #[test]
    fn ring_should_extend_in_both_x_and_z() {
        let pattern = Ring::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(p!(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(p!(1.0, 0.0, 0.0)), Color::black());
        assert_eq!(pattern.color_at(p!(0.0, 0.0, 1.0)), Color::black());
        assert_eq!(pattern.color_at(p!(0.708, 0.0, 0.708)), Color::black());
    }

    #[test]
    fn checkers_repeat_in_x() {
        let pattern = Checkers::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(p!(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(p!(0.99, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(p!(1.01, 0.0, 0.0)), Color::black());
    }

    #[test]
    fn checkers_repeat_in_y() {
        let pattern = Checkers::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(p!(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(p!(0.0, 0.99, 0.0)), Color::white());
        assert_eq!(pattern.color_at(p!(0.0, 1.01, 0.0)), Color::black());
    }

    #[test]
    fn checkers_repeat_in_z() {
        let pattern = Checkers::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(p!(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(p!(0.0, 0.0, 0.99)), Color::white());
        assert_eq!(pattern.color_at(p!(0.0, 0.0, 1.01)), Color::black());
    }
}
