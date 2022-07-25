use crate::tuple::Tuple;
use std::ops;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color(Tuple);

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Tuple(r, g, b, 0.0))
    }

    pub fn r(&self) -> f64 {
        self.0 .0
    }

    pub fn g(&self) -> f64 {
        self.0 .1
    }

    pub fn b(&self) -> f64 {
        self.0 .2
    }

    fn hadamard_with(&self, c: Color) -> Color {
        let Tuple(r, g, b, ..) = self.0;
        let Tuple(x, y, z, ..) = c.0;
        Color(Tuple(r * x, g * y, b * z, 0.0))
    }
}

/// Generates methods for colors from RGB values.
macro_rules! add_color {
    ($($name:ident: $r:expr, $g:expr, $b:expr,)*) => {
        impl Color {
        $(
            pub fn $name() -> Self {
                Self::new($r as f64/256.0, $g as f64/256.0, $b as f64/256.0)
            }
        )*
        }
    };
}

add_color! {
    white: 256, 256, 256,
    black: 0, 0, 0,
    dolphin_gray: 116, 146, 140,
    crayola_gold: 227, 256, 236,
    watermelon: 232, 102, 137,
    boysenberry: 130, 57, 103,
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Self::Output {
        Color(self.0 + rhs.0)
    }
}

impl ops::Sub<Color> for Color {
    type Output = Color;

    fn sub(self, rhs: Color) -> Self::Output {
        Color(self.0 - rhs.0)
    }
}

impl ops::Mul<f64> for Color {
    type Output = Color;

    fn mul(self, rhs: f64) -> Self::Output {
        Color(self.0 * rhs)
    }
}

impl ops::Mul<Color> for f64 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color(self * rhs.0)
    }
}

impl ops::Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        self.hadamard_with(rhs)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            (self.r() * 255.0).clamp(0.0, 255.0).ceil(),
            (self.g() * 255.0).clamp(0.0, 255.0).ceil(),
            (self.b() * 255.0).clamp(0.0, 255.0).ceil(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn basic() {
        let c = Color::new(-0.5, 0.4, 1.7);
        assert_eq!(c.r(), -0.5);
        assert_eq!(c.g(), 0.4);
        assert_eq!(c.b(), 1.7);
    }

    #[test]
    fn add_colors() {
        let c1 = Color::new(0.9, 0.6, 0.75);
        let c2 = Color::new(0.7, 0.1, 0.25);
        assert_eq!(c1 + c2, Color::new(1.6, 0.7, 1.0));
    }

    #[test]
    fn subtract_colors() {
        let c1 = Color::new(0.9, 0.6, 0.75);
        let c2 = Color::new(0.7, 0.1, 0.25);
        assert_eq!(c1 - c2, Color::new(0.2, 0.5, 0.5));
    }

    #[test]
    fn mul_color_by_scalar() {
        let c = Color::new(0.2, 0.3, 0.4);
        assert_eq!(c * 2.0, Color::new(0.4, 0.6, 0.8));
    }

    #[test]
    fn mul_by_color() {
        let c1 = Color::new(1.0, 0.2, 0.4);
        let c2 = Color::new(0.9, 1.0, 0.1);
        assert_eq!(c1 * c2, Color::new(0.9, 0.2, 0.04));
    }

    #[test]
    fn fmt_is_ok() {
        let tests = vec![
            (Color::new(1.0, 0.0, 0.0), "255 0 0"),
            (Color::new(0.0, 1.0, 0.0), "0 255 0"),
            (Color::new(0.0, 0.0, 1.0), "0 0 255"),
            (Color::new(0.0, 0.5, 0.0), "0 128 0"),
            (Color::new(-0.5, 0.0, 0.5), "0 0 128"),
            (Color::new(1.5, 0.0, 0.0), "255 0 0"),
        ];
        for test in tests {
            assert_eq!(test.0.to_string(), test.1);
        }
    }
}
