use crate::{
    color::{self, Color},
    hittable::HitRecord,
    ray::Ray,
    vec_rand::random_unit_vec,
};

pub struct ScatterResult {
    pub out_ray: Ray,
    pub attenuation: Color,
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
            Material::Lambertian { albedo } => {
                let scatter_dir = hit_record.normal + random_unit_vec();
                // TODO: If vector is near zero, just scatter towards the normal
                // todo

                let ray_out = Ray {
                    origin: hit_record.point,
                    direction: scatter_dir,
                };

                Some(ScatterResult {
                    out_ray: ray_out,
                    attenuation: *albedo,
                })
            }
            Material::Metal { albedo, fuzz } => None,
        }
    }
}
