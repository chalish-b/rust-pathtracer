use crate::{color::Color, hittable::Hit, ray::Ray};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Skybox {
    None,
    Dim,
    BlueGradient,
}

impl Skybox {
    pub fn label(self) -> &'static str {
        match self {
            Skybox::None => "None",
            Skybox::Dim => "Dim ambient",
            Skybox::BlueGradient => "Blue gradient",
        }
    }

    pub fn color(self, ray: Ray) -> Color {
        match self {
            Skybox::None => Color::BLACK,
            Skybox::Dim => Color::new(0.01, 0.01, 0.02),
            Skybox::BlueGradient => {
                const BLUE_SKY: Color = Color::new(0.4, 0.58, 0.92);
                const WHITE_HORIZON: Color = Color::new(0.95, 0.95, 0.98);
                let norm = ray.direction.normalize();
                let y = (norm.y + 1.0) / 2.0;
                Color::mix(BLUE_SKY, WHITE_HORIZON, y)
            }
        }
    }
}

pub struct Scene {
    pub hittables: Vec<Box<dyn Hit>>,
    pub skybox: Skybox,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            hittables: vec![],
            skybox: Skybox::None,
        }
    }

    pub fn add_hittable(&mut self, hittable: impl Hit + 'static) {
        self.hittables.push(Box::new(hittable));
    }
}
