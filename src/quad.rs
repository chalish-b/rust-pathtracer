use glam::Vec3;

use crate::{
    hittable::{Hit, HitRecord},
    interval::Interval,
    material::{DEFAULT_MAT, Material},
    ray::Ray,
};

pub struct Quad {
    corner: Vec3,
    u: Vec3,
    v: Vec3,
    material: Material,
}

impl Quad {
    pub fn new(corner: Vec3, u: Vec3, v: Vec3) -> Self {
        Self {
            corner,
            u,
            v,
            material: DEFAULT_MAT,
        }
    }

    pub fn with_material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }

    pub fn normal(&self) -> Vec3 {
        Vec3::cross(self.u, self.v)
            .try_normalize()
            .unwrap_or(Vec3::ZERO)
    }

    pub fn plane_d(&self) -> f32 {
        Vec3::dot(self.normal(), self.corner)
    }
}

impl Hit for Quad {
    fn hit(&self, ray: Ray, interval: Interval) -> Option<HitRecord> {
        let n = self.normal();
        let d = self.plane_d();

        // t = (d - dot(n, ray_origin)) / (dot(n, ray_dir))
        // If denominator (dot(n, ray_dir)) is near zero, it means ray is parallel to the quad.
        let denominator = n.dot(ray.direction);
        if denominator.abs() < f32::EPSILON {
            return None;
        }

        let t = (d - n.dot(ray.origin)) / denominator;
        if !interval.contains(t) {
            return None;
        }

        // After we know it hits the plane, check whether that point is in the quad
        let hit_point = ray.at(t);
        let p = hit_point - self.corner;

        // Using unnormalized n is important here instead of self.normal() which is a unit vector
        let unnormalized_n = self.u.cross(self.v);
        let w = unnormalized_n / unnormalized_n.dot(unnormalized_n);
        let alpha = w.dot(p.cross(self.v));
        let beta = w.dot(self.u.cross(p));

        if !((0.0..=1.0).contains(&alpha) && (0.0..=1.0).contains(&beta)) {
            return None;
        }

        Some(HitRecord::new(n, ray, t, self.material))
    }
}
