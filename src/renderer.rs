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
    camera::Camera,
    canvas::Canvas,
    color::Color,
    hittable::HitRecord,
    interval::Interval,
    ray::Ray,
    scene::Scene,
    vec_rand::{random_in_disk, random_in_square},
};

#[derive(Debug, Copy, Clone)]
pub struct RenderOptions {
    pub antialiasing: bool,
    pub sample_count: i32,
    pub thread_count: usize,
    pub recursion_depth: i32,
}

const BOUNCE_EPSILON: f32 = 0.005;

#[derive(Debug, Copy, Clone)]
struct ViewportParams {
    pub top_left_px_center: Vec3,
    pub du: Vec3,
    pub dv: Vec3,
    pub defocus_du: Vec3,
    pub defocus_dv: Vec3,
}

// For rendering, we start with the top left center and keep adding du and dv
// for each pixel on canvas.
fn calculate_viewport_params(camera: &Camera, width: usize, height: usize) -> ViewportParams {
    let (right, up, forward) = camera.axes();
    let Vec2 { x: vw, y: vh } = camera.viewport_size();
    let cw = width as f32;
    let ch = height as f32;

    let viewport_u = right * vw;
    let viewport_v = -up * vh;
    let du = viewport_u / cw;
    let dv = viewport_v / ch;

    let defocus_r = camera.focus_distance * (camera.defocus_angle / 2.0).to_radians().tan();
    let defocus_du = right * defocus_r;
    let defocus_dv = up * defocus_r;

    // Here, we're actually using viewport distance to be focus distance, because
    // that's basically where we wanna shoot the rays at
    let viewport_center = camera.position + camera.focus_distance * forward;
    let viewport_top_left = viewport_center - (viewport_u + viewport_v) / 2.0;
    let top_left_px_center = viewport_top_left + (du + dv) / 2.0;

    ViewportParams {
        top_left_px_center,
        du,
        dv,
        defocus_du,
        defocus_dv,
    }
}

pub fn render(scene: &Scene, camera: &Camera, canvas: &mut Canvas, options: RenderOptions) {
    let ViewportParams {
        top_left_px_center,
        du,
        dv,
        defocus_du,
        defocus_dv,
    } = calculate_viewport_params(camera, canvas.w, canvas.h);

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

                    let pixel_center = top_left_px_center
                        + (((x as f32) + offset.x) * du)
                        + (((y as f32) + offset.y) * dv);

                    let ray_origin_offset = if camera.defocus_angle <= 0.0 {
                        Vec3::ZERO
                    } else {
                        let random = random_in_disk();
                        random.x * defocus_du + random.y * defocus_dv
                    };
                    let ray_origin = camera.position + ray_origin_offset;
                    let ray_dir = pixel_center - ray_origin;
                    let ray = Ray {
                        origin: ray_origin,
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

// Renders exactly 1 sample per pixel and returns raw linear colors (no gamma
// correction). Used for progressive rendering: call this repeatedly,
// accumulate the results, then average and gamma-correct for display.
pub fn render_single_sample(
    scene: &Scene,
    camera: &Camera,
    width: usize,
    height: usize,
    options: &RenderOptions,
) -> Vec<Color> {
    let ViewportParams {
        top_left_px_center,
        du,
        dv,
        defocus_du,
        defocus_dv,
    } = calculate_viewport_params(camera, width, height);

    let mut pixels = vec![Color::BLACK; width * height];

    pixels
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            #[allow(clippy::needless_range_loop)]
            for x in 0..width {
                let offset = if options.antialiasing {
                    random_in_square()
                } else {
                    Vec3::ZERO
                };

                let pixel_center = top_left_px_center
                    + (((x as f32) + offset.x) * du)
                    + (((y as f32) + offset.y) * dv);

                let ray_origin_offset = if camera.defocus_angle <= 0.0 {
                    Vec3::ZERO
                } else {
                    let random = random_in_disk();
                    random.x * defocus_du + random.y * defocus_dv
                };
                let ray_origin = camera.position + ray_origin_offset;
                let ray_dir = pixel_center - ray_origin;
                let ray = Ray {
                    origin: ray_origin,
                    direction: ray_dir,
                };

                row[x] = shoot_ray(scene, ray, options.recursion_depth);
            }
        });

    pixels
}

fn shoot_ray(scene: &Scene, ray: Ray, recursion_depth: i32) -> Color {
    if recursion_depth <= 0 {
        return Color::BLACK;
    }

    let hit_record = find_hit(scene, ray, Interval(BOUNCE_EPSILON, f32::INFINITY));
    if let Some(hit_rec) = hit_record {
        let emission_color = hit_rec.material.emission();

        // If material scatters
        if let Some(scatter_result) = hit_rec.material.scatter(ray, hit_rec) {
            let ray_color = shoot_ray(scene, scatter_result.out_ray, recursion_depth - 1);
            let scatter_color = scatter_result.attenuation * ray_color;
            return emission_color + scatter_color;
        }

        // If material doesn't scatter, just return emission color (which is black for
        // non-emissive materials so it all works out)
        return emission_color;
    }

    // If we don't hit anything, return skybox / ambient color
    skybox_color(ray)
}

fn skybox_color(ray: Ray) -> Color {
    // Disable skylight to test emissive materials
    return Color::BLACK;

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
