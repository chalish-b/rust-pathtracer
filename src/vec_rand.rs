use glam::Vec3;
use rand::RngExt;

pub fn random_vec(min: f32, max: f32) -> Vec3 {
    let mut rng = rand::rng();
    Vec3 {
        x: rng.random_range(min..=max),
        y: rng.random_range(min..=max),
        z: rng.random_range(min..=max),
    }
}

pub fn random_unit_vec() -> Vec3 {
    // The naive approach is to sample until we land inside the unit sphere, so that it's not biased
    // TODO: Replace this with a one shot method
    loop {
        let vec = random_vec(-1.0, 1.0);
        let len_squared = vec.length_squared();
        if 0.001 < len_squared && len_squared <= 1.0 {
            return vec.normalize();
        }
    }
}

pub fn random_in_square() -> Vec3 {
    let mut rng = rand::rng();
    Vec3 {
        x: rng.random_range(-0.5..=0.5),
        y: rng.random_range(-0.5..=0.5),
        z: 0.0,
    }
}
