use glam::Vec3;

use crate::{
    camera::Camera,
    canvas::Canvas,
    color::Color,
    hittable::Sphere,
    material::Material,
    renderer::{RenderOptions, render},
    scene::Scene,
};

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

const W: usize = 800;
const H: usize = 600;
const ASPECT: f32 = (W as f32) / (H as f32);

fn main() {
    let mat1 = Material::Lambertian {
        albedo: Color::new(0.89, 0.11, 0.14),
    };
    let mat2 = Material::Lambertian {
        albedo: Color::new(0.18, 0.92, 0.14),
    };

    // Emissive material test. It kinda works (it illuminates nearby objects)
    // but it also gets colored by nearby objects. I guess the solution is that once a ray hits
    // an emissive material, it can't bounce any further. Idk if there is anything else to consider.
    //  That should be an easy fix if we create a separate emissive material type.
    // let mat3 = Material::Lambertian {
    //     albedo: Color::new(2.5, 2.5, 2.5),
    // };

    let mut canvas = Canvas::new(W, H);

    // Camera set up
    let mut camera = Camera::new().with_aspect(ASPECT);
    camera.position = Vec3 {
        x: 0.0,
        y: 1.5,
        z: 0.0,
    };
    camera.look_at(Vec3 {
        x: 0.0,
        y: 0.0,
        z: -30.0,
    });

    let mut scene = Scene::new();
    scene.add_hittable(Sphere::new(Vec3::new(-1.0, 1.0, -5.0), 1.0).with_material(mat1));
    scene.add_hittable(Sphere::new(Vec3::new(2.0, 2.0, -8.0), 2.0).with_material(mat2));
    // scene.add_hittable(Sphere::new(Vec3::new(0.0, 2.0, -6.0), 0.5).with_material(mat3));
    scene.add_hittable(Sphere::new(Vec3::new(0.0, -2000.0, -0.0), 2000.0));

    let render_config = RenderOptions::new();

    renderer::render(&scene, &camera, &mut canvas, render_config);
    canvas.save_image("output.ppm").unwrap();
}
