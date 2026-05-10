use crate::hittable::{Hit, HitRecord};
use crate::interval::Interval;
use crate::material::{DEFAULT_MAT, Material};
use crate::ray::Ray;
use glam::Vec3;

// Concrete types (sphere only for now)
pub struct Sphere {
    center: Vec3,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self {
            center,
            radius,
            material: DEFAULT_MAT,
        }
    }

    pub fn with_material(mut self, mat: Material) -> Self {
        self.material = mat;
        self
    }
}

impl Hit for Sphere {
    fn hit(&self, ray: Ray, interval: Interval) -> Option<HitRecord> {
        let oc = self.center - ray.origin;
        let a = Vec3::dot(ray.direction, ray.direction);
        let b = -2.0 * Vec3::dot(oc, ray.direction);
        let c = Vec3::dot(oc, oc) - self.radius.powi(2);

        let delta = b * b - 4.0 * a * c;
        if delta < 0.0 {
            return None;
        }

        let t1 = (-b - delta.sqrt()) / (2.0 * a);
        let t2 = (-b + delta.sqrt()) / (2.0 * a);

        let mut t = t1;
        if !interval.contains(t) {
            t = t2;
            if !interval.contains(t) {
                return None;
            }
        }

        let point = ray.at(t);
        let normal = (point - self.center) / self.radius;
        Some(HitRecord::new(normal, ray, t, self.material))
    }
}
