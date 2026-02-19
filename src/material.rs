use glam::Vec3;
use rand::RngExt;

use crate::{color::Color, hittable::HitRecord, ray::Ray, vec_rand::random_unit_vec};

pub struct ScatterResult {
    pub out_ray: Ray,
    pub attenuation: Color,
}

#[derive(Debug, Copy, Clone)]
pub enum Material {
    Lambertian {
        albedo: Color,
    },
    Metal {
        albedo: Color,
        fuzz: f32,
    },
    Glass {
        albedo: Color,
        refraction_index: f32,
    },
}

impl Material {
    pub fn scatter(&self, in_ray: Ray, hit_record: HitRecord) -> Option<ScatterResult> {
        match self {
            Material::Lambertian { albedo } => {
                let scatter_dir = hit_record.normal + random_unit_vec();
                // TODO: If vector is near zero, just scatter towards the normal

                let ray_out = Ray {
                    origin: hit_record.point,
                    direction: scatter_dir,
                };

                Some(ScatterResult {
                    out_ray: ray_out,
                    attenuation: *albedo,
                })
            }
            Material::Metal { albedo, fuzz } => {
                let reflected_ray = reflect(in_ray.direction, hit_record.normal);
                let fuzzy_reflected = reflected_ray.normalize() + (fuzz * random_unit_vec());

                // When adding a random fuzz, the ray can now go opposite to the normal as well
                // We ignore those, treat them as absorbed rays
                if Vec3::dot(hit_record.normal, fuzzy_reflected) < 0.0 {
                    None
                } else {
                    Some(ScatterResult {
                        out_ray: Ray {
                            origin: hit_record.point,
                            direction: fuzzy_reflected,
                        },
                        attenuation: *albedo,
                    })
                }
            }
            Material::Glass {
                albedo,
                refraction_index,
            } => {
                let mut rng = rand::rng();

                let ri = if hit_record.is_front_face {
                    1.0 / refraction_index
                } else {
                    *refraction_index
                };

                let normalized_in = in_ray.direction.normalize();
                let cos_theta = Vec3::dot(-normalized_in, hit_record.normal);
                let sin_theta = f32::sqrt(1.0 - cos_theta * cos_theta);

                let is_total_internal_reflection = ri * sin_theta > 1.0;

                // TODO: Should the second argument here be the raw `refraction_index`, or `ri` (adjusted for back face)?
                let should_have_reflectance =
                    reflectance(cos_theta, *refraction_index) > rng.random_range(0.0..=1.0);

                let out_dir = if is_total_internal_reflection || should_have_reflectance {
                    reflect(normalized_in, hit_record.normal)
                } else {
                    refract(normalized_in, hit_record.normal, ri)
                };

                Some(ScatterResult {
                    out_ray: Ray {
                        origin: hit_record.point,
                        direction: out_dir,
                    },
                    attenuation: *albedo,
                })
            }
        }
    }
}

fn reflectance(cos: f32, refractive_index: f32) -> f32 {
    let r0 = ((1.0 - refractive_index) / (1.0 + refractive_index)).powf(2.0);

    r0 + (1.0 - r0) * (1.0 - cos).powf(5.0)
}

fn reflect(vec: Vec3, normal: Vec3) -> Vec3 {
    vec + 2.0 * normal * Vec3::dot(normal, -vec)
}

// `etai_over_etat` = n/n'
fn refract(vec: Vec3, normal: Vec3, etai_over_etat: f32) -> Vec3 {
    let normalized_in = vec.normalize();
    let cos_theta = Vec3::dot(-normalized_in, normal);
    let perpendicular = etai_over_etat * (normalized_in + normal * cos_theta);
    // -sqrt(abs(1 - |p|^2)) * normal
    let parallel = -f32::sqrt(f32::abs(1.0 - perpendicular.length_squared())) * normal;

    parallel + perpendicular
}
