use crate::{color::Color, interval::Interval, material::Material, ray::Ray};
use glam::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct HitRecord {
    pub normal: Vec3,
    pub ray: Ray,
    pub t: f32,
    pub point: Vec3, // This one might be redundant since ray.origin + t * ray.direction = point
    pub is_front_face: bool,
    pub material: Material,
}

impl HitRecord {
    // is_front_face will be automatically determined and the surface normal will be
    // flipped accordingly. Don't do it manually, just pass the surface normal as is.
    pub fn new(surface_normal: Vec3, ray: Ray, t: f32, material: Material) -> Self {
        let mut normal = surface_normal;
        let mut is_front_face = true;

        if Vec3::dot(ray.direction, normal) > 0.0 {
            is_front_face = false;
            normal = -normal;
        }

        Self {
            normal,
            ray,
            t,
            point: ray.origin + t * ray.direction,
            is_front_face,
            material,
        }
    }
}

// Hit trait that objects need to implement
pub trait Hit: Send + Sync {
    /// Checks whether a ray hits this object, returns the hit record if it does
    fn hit(&self, ray: Ray, interval: Interval) -> Option<HitRecord>;
}
