mod camera;
mod canvas;
mod color;
mod hittable;
mod interval;
mod material;
mod ray;
mod renderer;
mod scene;
mod vec_rand;

use crate::{
    camera::Camera, canvas::Canvas, color::Color, hittable::Sphere, material::Material,
    renderer::RenderOptions, scene::Scene,
};
use glam::Vec3;
use std::time::Instant;

// Change these to tweak the render settings
const W: usize = 800;
const H: usize = 600;
const SAMPLE_COUNT: i32 = 64;
const RECURSION_DEPTH: i32 = 12;
const THREAD_COUNT: usize = 12;

fn main() {
    let red_diffuse = Material::Lambertian {
        albedo: Color::new(0.98, 0.10, 0.12),
    };
    let green_diffuse = Material::Lambertian {
        albedo: Color::new(0.32, 0.94, 0.30),
    };
    let yellow_diffuse = Material::Lambertian {
        albedo: Color::new(0.92, 0.94, 0.21),
    };
    let white_diffuse = Material::Lambertian {
        albedo: Color::new(0.98, 0.92, 0.88),
    };
    let black_diffuse = Material::Lambertian {
        albedo: Color::new(0.07, 0.07, 0.07),
    };
    let blue_metal = Material::Metal {
        albedo: Color::new(0.12, 0.10, 0.92),
        fuzz: 0.1,
    };
    let white_metal = Material::Metal {
        albedo: Color::new(0.9, 0.9, 0.9),
        fuzz: 0.01,
    };
    let white_glass = Material::Glass {
        albedo: Color::new(1.0, 1.0, 1.0),
        refraction_index: 1.025,
    };
    let white_glass2 = Material::Glass {
        albedo: Color::new(1.0, 1.0, 1.0),
        refraction_index: 1.01,
    };
    let air_glass = Material::Glass {
        albedo: Color::new(1.0, 1.0, 1.0),
        refraction_index: 1.0 / 1.025,
    };

    let mut canvas = Canvas::new(W, H);

    let mut camera = Camera::new();
    camera.aspect = (W as f32) / (H as f32);
    camera.v_fov_deg = 65.0;
    camera.position = Vec3::new(0.0, 0.0, 0.0);
    camera.look_at(Vec3::new(0.0, 1.0, -7.0));

    let mut scene = Scene::new();
    scene.add_hittable(Sphere::new(Vec3::new(0.0, 0.0, -7.0), 1.0).with_material(red_diffuse));
    scene.add_hittable(Sphere::new(Vec3::new(1.6, -0.4, -6.5), 0.6).with_material(yellow_diffuse));
    scene.add_hittable(Sphere::new(Vec3::new(1.2, 2.0, -2.0), 1.2).with_material(green_diffuse));
    scene.add_hittable(Sphere::new(Vec3::new(-2.0, 5.0, -15.0), 2.0).with_material(black_diffuse));
    scene.add_hittable(Sphere::new(Vec3::new(-3.0, 0.98, -10.0), 2.0).with_material(blue_metal));
    scene.add_hittable(Sphere::new(Vec3::new(-0.3, 0.4, -3.2), 0.5).with_material(white_glass));
    scene.add_hittable(Sphere::new(Vec3::new(0.45, 0.5, -3.8), 0.35).with_material(white_glass2));
    scene.add_hittable(Sphere::new(Vec3::new(-0.3, 0.4, -3.2), 0.45).with_material(air_glass));
    scene.add_hittable(Sphere::new(Vec3::new(4.0, 1.95, -12.0), 3.0).with_material(white_metal));
    scene.add_hittable(
        Sphere::new(Vec3::new(0.0, -501.0, -7.0), 500.0).with_material(white_diffuse),
    );

    let render_config = RenderOptions {
        antialiasing: true,
        recursion_depth: RECURSION_DEPTH,
        sample_count: SAMPLE_COUNT,
        thread_count: THREAD_COUNT,
    };

    let now = Instant::now();
    renderer::render(&scene, &camera, &mut canvas, render_config);
    let elapsed = now.elapsed();
    println!("Elapsed: {elapsed:.2?}");

    canvas.save_image("output.ppm").unwrap();
}
