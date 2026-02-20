use glam::Vec3;
use rand::RngExt;
use std::f32;

// Not needed anymore since it was only required by the old random_unit_vec algo
pub fn _random_vec(min: f32, max: f32) -> Vec3 {
    let mut rng = rand::rng();
    Vec3 {
        x: rng.random_range(min..=max),
        y: rng.random_range(min..=max),
        z: rng.random_range(min..=max),
    }
}

pub fn random_unit_vec() -> Vec3 {
    let mut rng = rand::rng();

    // Sampling uniformly on a sphere:
    // - Pick a z coordinate (not the polar angle) uniformly from [-1, 1]
    // - Pick theta (azimuthal angle) uniformly from [0, 2pi]
    let z = rng.random_range(-1.0..=1.0f32);
    let theta = rng.random_range(0.0..(2.0 * f32::consts::PI));

    // This is the radius on the xy plane
    let r = (1.0 - z * z).sqrt();

    Vec3 {
        x: r * theta.cos(),
        y: r * theta.sin(),
        z,
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

pub fn random_in_disk() -> Vec3 {
    let mut rng = rand::rng();

    let theta = rng.random_range(0.0..(2.0 * f32::consts::PI));
    let r = rng.random_range(0.0..1.0f32).sqrt();

    Vec3 {
        x: r * theta.cos(),
        y: r * theta.sin(),
        z: 0.0,
    }
}
