mod app;
mod camera;
mod canvas;
mod color;
mod hittable;
mod interval;
mod material;
mod quad;
mod ray;
mod renderer;
mod scene;
mod sphere;
mod vec_rand;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Path Tracer",
        options,
        Box::new(|cc| Ok(Box::new(app::PathTracerApp::new(cc)))),
    )
}
