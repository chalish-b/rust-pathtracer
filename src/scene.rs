use crate::hittable::Hit;

pub struct Scene {
    pub hittables: Vec<Box<dyn Hit>>,
}

impl Scene {
    pub fn new() -> Self {
        Self { hittables: vec![] }
    }

    pub fn add_hittable(&mut self, hittable: impl Hit + 'static) {
        self.hittables.push(Box::new(hittable));
    }
}
