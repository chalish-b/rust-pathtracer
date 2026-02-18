use std::ops::{Add, AddAssign, Mul};

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b }
    }

    pub fn mix(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);

        self * t + other * (1.0 - t)
    }

    pub fn to_gamma(self) -> Color {
        fn gamma(c: f32) -> f32 {
            if c > 0.0 { c.sqrt() } else { 0.0 }
        }

        Color {
            r: gamma(self.r),
            g: gamma(self.g),
            b: gamma(self.b),
        }
    }
}

impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, rhs: f32) -> Self::Output {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl Mul<Color> for f32 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}

impl Add<Color> for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);
pub const WHITE: Color = Color::new(1.0, 1.0, 1.0);
pub const RED: Color = Color::new(1.0, 0.0, 0.0);
pub const GREEN: Color = Color::new(0.0, 1.0, 0.0);
pub const BLUE: Color = Color::new(0.0, 0.0, 1.0);
pub const YELLOW: Color = Color::new(1.0, 1.0, 0.0);
pub const CYAN: Color = Color::new(0.0, 1.0, 1.0);
pub const MAGENTA: Color = Color::new(1.0, 0.0, 1.0);
