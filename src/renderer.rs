use std::{
    f32,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use glam::{Vec2, Vec3};
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::{
    camera::Camera, canvas::Canvas, color::Color, hittable::HitRecord, interval::Interval,
    ray::Ray, scene::Scene, vec_rand::random_in_square,
};

#[derive(Debug, Copy, Clone)]
pub struct RenderOptions {
    pub antialiasing: bool,
    pub sample_count: i32,
    pub thread_count: usize,
    pub recursion_depth: i32,
}

const BOUNCE_EPSILON: f32 = 0.005;

/// Returns `(top_left_pixel_center, du, dv)`.
/// For rendering, we start with the top left center and keep adding du and dv for each pixel on canvas.
fn calculate_render_params(camera: &Camera, canvas: &Canvas) -> (Vec3, Vec3, Vec3) {
    let (right, up, forward) = camera.axes();
    let Vec2 { x: vw, y: vh } = camera.viewport_size();
    let cw = canvas.w as f32;
    let ch = canvas.h as f32;

    let viewport_u = right * vw;
    let viewport_v = -up * vh;
    let du = viewport_u / cw;
    let dv = viewport_v / ch;

    // Assuming viewport distance is 1 here, we don't need to multiply forward vec by distance
    let viewport_center = camera.position + forward;
    let viewport_top_left = viewport_center - (viewport_u + viewport_v) / 2.0;
    let top_left_pixel_center = viewport_top_left + (du + dv) / 2.0;

    (top_left_pixel_center, du, dv)
}

pub fn render(scene: &Scene, camera: &Camera, canvas: &mut Canvas, options: RenderOptions) {
    let (top_left_pixel_center, du, dv) = calculate_render_params(camera, canvas);

    // Set the number of threads of the thread pool
    // Calling build_global multiple times would give an error,
    // but it doesn't really matter so we ignore it with .ok()
    rayon::ThreadPoolBuilder::new()
        .num_threads(options.thread_count)
        .build_global()
        .ok();

    // Shared progress counter so threads can update it
    let total_rows = canvas.h;
    let rows_done = Arc::new(AtomicUsize::new(0));

    canvas
        .pixels
        .par_chunks_mut(canvas.w)
        .enumerate()
        .for_each(|(y, row)| {
            // The range based loop is more intuitive in this case imo
            #[allow(clippy::needless_range_loop)]
            for x in 0..canvas.w {
                let mut pixel_color = Color::BLACK;

                for _ in 0..options.sample_count {
                    let offset = if options.antialiasing {
                        random_in_square()
                    } else {
                        Vec3::ZERO
                    };

                    let pixel_center = top_left_pixel_center
                        + (((x as f32) + offset.x) * du)
                        + (((y as f32) + offset.y) * dv);

                    let ray_dir = pixel_center - camera.position;
                    let ray = Ray {
                        origin: camera.position,
                        direction: ray_dir,
                    };

                    let ray_color = shoot_ray(scene, ray, options.recursion_depth);
                    pixel_color += ray_color;
                }

                let final_color = pixel_color * (1.0 / (options.sample_count as f32));
                let gamma_corrected = final_color.to_gamma();

                row[x] = gamma_corrected;
            }

            // Update counter
            // Only on integer changes so we don't spam the output
            let prev_done = rows_done.fetch_add(1, Ordering::Relaxed);
            let prev_p = (prev_done * 100) / total_rows;
            let current_p = (prev_done + 1) * 100 / total_rows;
            if prev_p != current_p {
                println!("{current_p}%");
            }
        });
}

fn shoot_ray(scene: &Scene, ray: Ray, recursion_depth: i32) -> Color {
    if recursion_depth <= 0 {
        return Color::BLACK;
    }

    let hit_record = find_hit(scene, ray, Interval(BOUNCE_EPSILON, f32::INFINITY));
    if let Some(hit_rec) = hit_record {
        if let Some(scatter_result) = hit_rec.material.scatter(ray, hit_rec) {
            let ray_color = shoot_ray(scene, scatter_result.out_ray, recursion_depth - 1);
            return scatter_result.attenuation * ray_color;
        }

        return Color::BLACK;
    }

    skybox_color(ray)
}

fn skybox_color(ray: Ray) -> Color {
    const BLUE_SKY: Color = Color::new(0.4, 0.58, 0.92);
    const WHITE_HORIZON: Color = Color::new(0.95, 0.95, 0.98);

    let norm = ray.direction.normalize();
    let y = (norm.y + 1.0) / 2.0;

    Color::mix(BLUE_SKY, WHITE_HORIZON, y)
}

fn find_hit(scene: &Scene, ray: Ray, interval: Interval) -> Option<HitRecord> {
    let mut closest_t = interval.1;
    let mut closest_hit_record: Option<HitRecord> = None;

    for hittable in scene.hittables.iter() {
        let record = hittable.hit(ray, Interval(interval.0, closest_t));
        if let Some(hit_rec) = record {
            closest_t = hit_rec.t;
            closest_hit_record = Some(hit_rec);
        }
    }

    closest_hit_record
}
