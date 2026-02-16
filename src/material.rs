use crate::{
    color::{self, Color},
    hittable::HitRecord,
    ray::Ray,
};

pub struct ScatterResult {
    out_ray: Ray,
    attenuation: Color,
}

#[derive(Debug, Copy, Clone)]
pub enum Material {
    Lambertian { albedo: Color },
    Metal { albedo: Color, fuzz: f32 },
    // Dielectric { ior: f32 },
}

impl Material {
    // Just a test function to get the raw color before we implement any lighting
    pub fn color(&self) -> Color {
        match self {
            Material::Lambertian { albedo } => *albedo,
            Material::Metal { albedo, fuzz } => *albedo,
        }
    }

    pub fn scatter(&self, in_ray: Ray, hit_record: HitRecord) -> Option<ScatterResult> {
        match self {
            Material::Lambertian { albedo } => None,
            Material::Metal { albedo, fuzz } => None,
        }
    }
}
