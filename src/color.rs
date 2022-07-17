use crate::tuple::Tuple;
use std::ops;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color(Tuple);

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Tuple(r, g, b, 0.0))
    }

    pub fn red(&self) -> f64 {
        self.0 .0
    }

    pub fn green(&self) -> f64 {
        self.0 .1
    }

    pub fn blue(&self) -> f64 {
        self.0 .2
    }

    fn hadamard_with(&self, c: Color) -> Color {
        let Tuple(r, g, b, ..) = self.0;
        let Tuple(x, y, z, ..) = c.0;
        Color(Tuple(r * x, g * y, b * z, 0.0))
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
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
            (self.red() * 255.0).clamp(0.0, 255.0).ceil(),
            (self.green() * 255.0).clamp(0.0, 255.0).ceil(),
            (self.blue() * 255.0).clamp(0.0, 255.0).ceil(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn basic() {
        let c = Color::new(-0.5, 0.4, 1.7);
        assert_eq!(c.red(), -0.5);
        assert_eq!(c.green(), 0.4);
        assert_eq!(c.blue(), 1.7);
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
