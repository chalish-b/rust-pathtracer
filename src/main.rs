use glam::Vec3;

use crate::{
    camera::Camera, canvas::Canvas, color::Color, hittable::Sphere, material::Material,
    renderer::RenderOptions, scene::Scene,
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
    let mat1 = Material::Lambertian { albedo: color::RED };
    let mat2 = Material::Lambertian {
        albedo: color::GREEN,
    };

    let mut canvas = Canvas::new(W, H);
    let camera = Camera::new().with_aspect(ASPECT);
    let mut scene = Scene::new();
    scene.add_hittable(Sphere::new(Vec3::new(0.0, 1.0, -10.0), 1.0).with_material(mat1));
    let render_config = RenderOptions::new();

    renderer::render(&scene, &camera, &mut canvas, render_config);
    canvas.save_image("output.ppm").unwrap();
}
