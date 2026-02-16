use crate::{color, interval::Interval, material::Material, ray::Ray};
use glam::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct HitRecord {
    pub normal: Vec3,
    pub point: Vec3,
    pub t: f32,
    pub is_front_face: bool,
    pub material: Material,
}

pub trait Hit {
    /// Checks whether a ray hits this object, returns the hit record if it does
    fn hit(&self, ray: Ray, interval: Interval) -> Option<HitRecord>;
}

// Concrete types (sphere only for now)
pub struct Sphere {
    center: Vec3,
    radius: f32,
    material: Material,
}

const DEFAULT_MAT: Material = Material::Lambertian {
    albedo: color::WHITE,
};

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
        let c = Vec3::dot(oc, oc) - self.radius.powf(2.0);

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
        };

        let point = ray.at(t);
        let mut normal = (point - self.center) / self.radius;
        let mut is_front_face = true;

        if Vec3::dot(ray.direction, normal) > 0.0 {
            is_front_face = false;
            normal = -normal;
        };

        Some(HitRecord {
            t,
            is_front_face,
            normal,
            point,
            material: self.material,
        })
    }
}
