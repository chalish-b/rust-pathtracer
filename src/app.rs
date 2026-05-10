use eframe::egui;
use glam::Vec3;
use std::io::{BufWriter, Write};
use std::sync::mpsc;
use std::time::Instant;
use std::{fs, path, thread};

use crate::{
    camera::Camera,
    color::Color,
    material::Material,
    quad::Quad,
    renderer::{self, RenderOptions},
    scene::{Scene, Skybox},
    sphere::Sphere,
};

const W: usize = 800;
const H: usize = 600;
const PREVIEW_SCALE: usize = 2; // Render at lower resolution during edits
const PREVIEW_SETTLE_MS: u128 = 300; // ms of no edits before switching to full res
const RECURSION_DEPTH: i32 = 16;
const THREAD_COUNT: usize = 10;

// ---------------------------------------------------------------------------
// Editable scene description (UI-friendly types)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum MaterialParams {
    Lambertian {
        albedo: [f32; 3],
    },
    Metal {
        albedo: [f32; 3],
        fuzz: f32,
    },
    Glass {
        albedo: [f32; 3],
        refraction_index: f32,
    },
    Emissive {
        albedo: [f32; 3],
        power: f32,
    },
}

impl MaterialParams {
    fn label(&self) -> &'static str {
        match self {
            MaterialParams::Lambertian { .. } => "Lambertian",
            MaterialParams::Metal { .. } => "Metal",
            MaterialParams::Glass { .. } => "Glass",
            MaterialParams::Emissive { .. } => "Emissive",
        }
    }

    fn albedo_mut(&mut self) -> &mut [f32; 3] {
        match self {
            MaterialParams::Lambertian { albedo } => albedo,
            MaterialParams::Metal { albedo, .. } => albedo,
            MaterialParams::Glass { albedo, .. } => albedo,
            MaterialParams::Emissive { albedo, .. } => albedo,
        }
    }

    fn to_material(&self) -> Material {
        match self {
            MaterialParams::Lambertian { albedo } => Material::Lambertian {
                albedo: Color::new(albedo[0], albedo[1], albedo[2]),
            },
            MaterialParams::Metal { albedo, fuzz } => Material::Metal {
                albedo: Color::new(albedo[0], albedo[1], albedo[2]),
                fuzz: *fuzz,
            },
            MaterialParams::Glass {
                albedo,
                refraction_index,
            } => Material::Glass {
                albedo: Color::new(albedo[0], albedo[1], albedo[2]),
                refraction_index: *refraction_index,
            },
            MaterialParams::Emissive { albedo, power } => Material::Emissive {
                albedo: Color::new(albedo[0], albedo[1], albedo[2]),
                power: *power,
            },
        }
    }
}

#[derive(Clone, Debug)]
struct SphereParams {
    name: String,
    center: [f32; 3],
    radius: f32,
    material: MaterialParams,
}

#[derive(Clone, Debug)]
struct CameraParams {
    position: [f32; 3],
    look_at: [f32; 3],
    v_fov_deg: f32,
    focus_distance: f32,
    defocus_angle: f32,
}

#[derive(Clone, Debug)]
struct SceneParams {
    spheres: Vec<SphereParams>,
    camera: CameraParams,
    recursion_depth: i32,
    skybox: Skybox,
}

// ---------------------------------------------------------------------------
// Conversion: SceneParams -> renderable Scene + Camera
// ---------------------------------------------------------------------------

fn build_scene_from_params(params: &SceneParams) -> (Scene, Camera) {
    let cp = &params.camera;
    let mut camera = Camera::new();
    camera.aspect = (W as f32) / (H as f32);
    camera.v_fov_deg = cp.v_fov_deg;
    camera.position = Vec3::new(cp.position[0], cp.position[1], cp.position[2]);
    camera.look_at(Vec3::new(cp.look_at[0], cp.look_at[1], cp.look_at[2]));
    camera.focus_distance = cp.focus_distance;
    camera.defocus_angle = cp.defocus_angle;

    let mut scene = Scene::new();
    scene.skybox = params.skybox;
    for sp in &params.spheres {
        let center = Vec3::new(sp.center[0], sp.center[1], sp.center[2]);
        scene.add_hittable(Sphere::new(center, sp.radius).with_material(sp.material.to_material()));
    }

    // Static test quads
    // Tall lambertian quad on the left, like a back-left wall
    scene.add_hittable(
        Quad::new(
            Vec3::new(-3.5, 0.0, -2.0),
            Vec3::new(0.0, 0.0, 20.0),
            Vec3::new(0.0, 20.0, 0.0),
        )
        .with_material(Material::Lambertian {
            albedo: Color::new(0.55, 0.25, 0.65),
        }),
    );

    // Metal quad on the right, tilted slightly — acts like a mirror panel
    scene.add_hittable(
        Quad::new(
            Vec3::new(3.4, 0.0, -1.8),
            Vec3::new(-0.4, 0.0, -3.6),
            Vec3::new(0.0, 2.6, 0.0),
        )
        .with_material(Material::Metal {
            albedo: Color::new(0.85, 0.85, 0.9),
            fuzz: 0.05,
        }),
    );

    // Emissive ceiling panel above the spheres
    scene.add_hittable(
        Quad::new(
            Vec3::new(-1.5, 4.2, -4.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.8),
        )
        .with_material(Material::Emissive {
            albedo: Color::new(1.0, 0.95, 0.85),
            power: 2.0,
        }),
    );

    // Small lambertian quad on the floor in front, like a tile
    scene.add_hittable(
        Quad::new(
            Vec3::new(-0.5, 0.01, -0.5),
            Vec3::new(1.2, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.2),
        )
        .with_material(Material::Lambertian {
            albedo: Color::new(0.18, 0.65, 0.35),
        }),
    );

    (scene, camera)
}

fn default_scene_params() -> SceneParams {
    SceneParams {
        recursion_depth: RECURSION_DEPTH,
        skybox: Skybox::None,
        camera: CameraParams {
            position: [6.0, 2.2, 5.0],
            look_at: [0.0, 0.7, -2.8],
            v_fov_deg: 34.0,
            focus_distance: 9.5,
            defocus_angle: 0.35,
        },
        spheres: vec![
            SphereParams {
                name: "Ground".into(),
                center: [0.0, -1000.0, 0.0],
                radius: 1000.0,
                material: MaterialParams::Lambertian {
                    albedo: [0.26, 0.24, 0.22],
                },
            },
            SphereParams {
                name: "Backdrop".into(),
                center: [0.0, 52.0, -130.0],
                radius: 50.0,
                material: MaterialParams::Lambertian {
                    albedo: [0.18, 0.22, 0.29],
                },
            },
            SphereParams {
                name: "Center glass shell".into(),
                center: [0.0, 1.05, -2.9],
                radius: 1.05,
                material: MaterialParams::Glass {
                    albedo: [1.0, 1.0, 1.0],
                    refraction_index: 1.5,
                },
            },
            SphereParams {
                name: "Center air bubble".into(),
                center: [0.0, 1.05, -2.9],
                radius: 0.9,
                material: MaterialParams::Glass {
                    albedo: [1.0, 1.0, 1.0],
                    refraction_index: 1.0 / 1.5,
                },
            },
            SphereParams {
                name: "Left brushed metal".into(),
                center: [-2.35, 0.8, -3.6],
                radius: 0.8,
                material: MaterialParams::Metal {
                    albedo: [0.72, 0.78, 0.86],
                    fuzz: 0.22,
                },
            },
            SphereParams {
                name: "Right polished copper".into(),
                center: [2.15, 0.9, -3.5],
                radius: 0.9,
                material: MaterialParams::Metal {
                    albedo: [0.92, 0.62, 0.38],
                    fuzz: 0.03,
                },
            },
            SphereParams {
                name: "Rear red matte".into(),
                center: [-0.8, 0.65, -5.25],
                radius: 0.65,
                material: MaterialParams::Lambertian {
                    albedo: [0.88, 0.18, 0.17],
                },
            },
            SphereParams {
                name: "Rear teal matte".into(),
                center: [1.0, 0.55, -5.05],
                radius: 0.55,
                material: MaterialParams::Lambertian {
                    albedo: [0.16, 0.70, 0.63],
                },
            },
            SphereParams {
                name: "Warm key light".into(),
                center: [-2.2, 4.6, -2.4],
                radius: 0.65,
                material: MaterialParams::Emissive {
                    albedo: [1.0, 0.92, 0.82],
                    power: 5.0,
                },
            },
            SphereParams {
                name: "Cool rim light".into(),
                center: [3.2, 3.2, -5.6],
                radius: 0.52,
                material: MaterialParams::Emissive {
                    albedo: [0.72, 0.84, 1.0],
                    power: 3.6,
                },
            },
            SphereParams {
                name: "Tiny accent light".into(),
                center: [0.2, 2.3, -1.7],
                radius: 0.16,
                material: MaterialParams::Emissive {
                    albedo: [1.0, 0.72, 0.34],
                    power: 14.0,
                },
            },
            SphereParams {
                name: "Foreground pearl".into(),
                center: [-1.05, 0.35, -1.5],
                radius: 0.35,
                material: MaterialParams::Metal {
                    albedo: [0.88, 0.86, 0.82],
                    fuzz: 0.08,
                },
            },
            SphereParams {
                name: "Foreground violet matte".into(),
                center: [1.05, 0.28, -1.25],
                radius: 0.28,
                material: MaterialParams::Lambertian {
                    albedo: [0.45, 0.24, 0.62],
                },
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

pub struct PathTracerApp {
    texture: Option<egui::TextureHandle>,
    accum_buffer: Vec<Color>,
    sample_count: u32,
    sample_receiver: mpsc::Receiver<Vec<Color>>,
    render_width: usize,
    render_height: usize,
    preview: bool,
    last_edit: Instant,
    start_time: Instant,
    scene_params: SceneParams,
}

impl PathTracerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        rayon::ThreadPoolBuilder::new()
            .num_threads(THREAD_COUNT)
            .build_global()
            .ok();

        let scene_params = default_scene_params();
        let (scene, camera) = build_scene_from_params(&scene_params);
        let receiver = spawn_render_thread(scene, camera, W, H, scene_params.recursion_depth);

        PathTracerApp {
            texture: None,
            accum_buffer: vec![Color::BLACK; W * H],
            sample_count: 0,
            sample_receiver: receiver,
            render_width: W,
            render_height: H,
            preview: false,
            last_edit: Instant::now(),
            start_time: Instant::now(),
            scene_params,
        }
    }

    fn save_current_image(&self) {
        if self.sample_count == 0 {
            return;
        }

        let mut n = 1u32;
        let filename = loop {
            let name = format!("render_{n:03}.ppm");
            if !path::Path::new(&name).exists() {
                break name;
            }
            n += 1;
        };

        let inv_count = 1.0 / self.sample_count as f32;
        let file = fs::File::create(&filename).expect("Failed to create file");
        let mut w = BufWriter::new(file);
        writeln!(w, "P3").unwrap();
        writeln!(w, "{} {}", self.render_width, self.render_height).unwrap();
        writeln!(w, "255").unwrap();
        for c in &self.accum_buffer {
            let gc = (*c * inv_count).to_gamma();
            let r = (gc.r * 255.0).clamp(0.0, 255.0) as u8;
            let g = (gc.g * 255.0).clamp(0.0, 255.0) as u8;
            let b = (gc.b * 255.0).clamp(0.0, 255.0) as u8;
            writeln!(w, "{r} {g} {b}").unwrap();
        }

        println!(
            "Saved {} ({}x{}, {} samples)",
            filename, self.render_width, self.render_height, self.sample_count
        );
    }

    // Reset accumulation and respawn the render thread at the given resolution
    fn restart_render(&mut self, width: usize, height: usize) {
        let (scene, camera) = build_scene_from_params(&self.scene_params);
        // Dropping the old receiver causes the old render thread's send() to fail
        // thread exits.
        self.sample_receiver = spawn_render_thread(
            scene,
            camera,
            width,
            height,
            self.scene_params.recursion_depth,
        );
        self.render_width = width;
        self.render_height = height;
        self.accum_buffer = vec![Color::BLACK; width * height];
        self.sample_count = 0;
        self.start_time = Instant::now();
    }
}

fn spawn_render_thread(
    scene: Scene,
    camera: Camera,
    width: usize,
    height: usize,
    recursion_depth: i32,
) -> mpsc::Receiver<Vec<Color>> {
    let (sender, receiver) = mpsc::channel::<Vec<Color>>();
    let render_options = RenderOptions {
        antialiasing: true,
        recursion_depth,
        sample_count: 1,
        thread_count: THREAD_COUNT,
    };

    thread::spawn(move || {
        loop {
            let sample =
                renderer::render_single_sample(&scene, &camera, width, height, &render_options);
            if sender.send(sample).is_err() {
                break;
            }
        }
    });

    receiver
}

// ---------------------------------------------------------------------------
// UI
// ---------------------------------------------------------------------------

const MATERIAL_TYPES: [&str; 4] = ["Lambertian", "Metal", "Glass", "Emissive"];

impl eframe::App for PathTracerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain all available samples from the render thread
        let mut new_samples = false;
        while let Ok(sample) = self.sample_receiver.try_recv() {
            for (accum, s) in self.accum_buffer.iter_mut().zip(sample.iter()) {
                *accum += *s;
            }
            self.sample_count += 1;
            new_samples = true;
        }

        // Only rebuild texture if we got new data
        if new_samples && self.sample_count > 0 {
            let inv_count = 1.0 / self.sample_count as f32;
            let display: Vec<egui::Color32> = self
                .accum_buffer
                .iter()
                .map(|c| {
                    let avg = *c * inv_count;
                    let g = avg.to_gamma();
                    egui::Color32::from_rgb(
                        (g.r * 255.0).clamp(0.0, 255.0) as u8,
                        (g.g * 255.0).clamp(0.0, 255.0) as u8,
                        (g.b * 255.0).clamp(0.0, 255.0) as u8,
                    )
                })
                .collect();

            let image = egui::ColorImage {
                size: [self.render_width, self.render_height],
                pixels: display,
                source_size: egui::Vec2::new(self.render_width as f32, self.render_height as f32),
            };

            match &mut self.texture {
                Some(tex) => tex.set(image, egui::TextureOptions::LINEAR),
                None => {
                    self.texture =
                        Some(ctx.load_texture("render", image, egui::TextureOptions::LINEAR));
                }
            }
        }

        // ----- Side panel with controls -----
        let mut dirty = false;

        egui::SidePanel::right("controls")
            .default_width(260.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Stats
                    let elapsed = self.start_time.elapsed().as_secs_f64();
                    let sps = if elapsed > 0.0 {
                        self.sample_count as f64 / elapsed
                    } else {
                        0.0
                    };
                    ui.label(format!(
                        "Samples: {} | {:.1} spp/sec",
                        self.sample_count, sps
                    ));

                    ui.horizontal(|ui| {
                        if ui.button("Reset render").clicked() {
                            dirty = true;
                        }
                        if ui.button("Save image").clicked() {
                            self.save_current_image();
                        }
                    });

                    dirty |= ui
                        .horizontal(|ui| {
                            ui.label("Bounces:");
                            ui.add(egui::Slider::new(
                                &mut self.scene_params.recursion_depth,
                                1..=50,
                            ))
                            .changed()
                        })
                        .inner;

                    dirty |= ui
                        .horizontal(|ui| {
                            ui.label("Skybox:");
                            let sky = &mut self.scene_params.skybox;
                            let before = *sky;
                            egui::ComboBox::from_id_salt("skybox")
                                .selected_text(sky.label())
                                .show_ui(ui, |ui| {
                                    for option in [Skybox::None, Skybox::Dim, Skybox::BlueGradient]
                                    {
                                        ui.selectable_value(sky, option, option.label());
                                    }
                                });
                            *sky != before
                        })
                        .inner;

                    ui.separator();

                    // Camera controls
                    egui::CollapsingHeader::new("Camera")
                        .default_open(false)
                        .show(ui, |ui| {
                            let cam = &mut self.scene_params.camera;

                            ui.label("Position:");
                            ui.horizontal(|ui| {
                                dirty |= ui
                                    .add(
                                        egui::DragValue::new(&mut cam.position[0])
                                            .prefix("x: ")
                                            .speed(0.05),
                                    )
                                    .changed();
                                dirty |= ui
                                    .add(
                                        egui::DragValue::new(&mut cam.position[1])
                                            .prefix("y: ")
                                            .speed(0.05),
                                    )
                                    .changed();
                                dirty |= ui
                                    .add(
                                        egui::DragValue::new(&mut cam.position[2])
                                            .prefix("z: ")
                                            .speed(0.05),
                                    )
                                    .changed();
                            });

                            ui.label("Look at:");
                            ui.horizontal(|ui| {
                                dirty |= ui
                                    .add(
                                        egui::DragValue::new(&mut cam.look_at[0])
                                            .prefix("x: ")
                                            .speed(0.05),
                                    )
                                    .changed();
                                dirty |= ui
                                    .add(
                                        egui::DragValue::new(&mut cam.look_at[1])
                                            .prefix("y: ")
                                            .speed(0.05),
                                    )
                                    .changed();
                                dirty |= ui
                                    .add(
                                        egui::DragValue::new(&mut cam.look_at[2])
                                            .prefix("z: ")
                                            .speed(0.05),
                                    )
                                    .changed();
                            });

                            dirty |= ui
                                .horizontal(|ui| {
                                    ui.label("FOV:");
                                    ui.add(
                                        egui::Slider::new(&mut cam.v_fov_deg, 10.0..=120.0)
                                            .suffix("°"),
                                    )
                                    .changed()
                                })
                                .inner;

                            dirty |= ui
                                .horizontal(|ui| {
                                    ui.label("Focus dist:");
                                    ui.add(
                                        egui::DragValue::new(&mut cam.focus_distance)
                                            .speed(0.05)
                                            .range(0.1..=100.0),
                                    )
                                    .changed()
                                })
                                .inner;

                            dirty |= ui
                                .horizontal(|ui| {
                                    ui.label("Defocus angle:");
                                    ui.add(
                                        egui::Slider::new(&mut cam.defocus_angle, 0.0..=10.0)
                                            .suffix("°"),
                                    )
                                    .changed()
                                })
                                .inner;
                        });

                    ui.separator();

                    // Per-sphere controls
                    let mut delete_idx: Option<usize> = None;

                    for (i, sphere) in self.scene_params.spheres.iter_mut().enumerate() {
                        egui::CollapsingHeader::new(&sphere.name)
                            .id_salt(format!("sphere_{i}"))
                            .default_open(false)
                            .show(ui, |ui| {
                                // Name (cosmetic only — no re-render needed)
                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    ui.text_edit_singleline(&mut sphere.name);
                                });

                                // Position
                                ui.label("Position:");
                                ui.horizontal(|ui| {
                                    dirty |= ui
                                        .add(
                                            egui::DragValue::new(&mut sphere.center[0])
                                                .prefix("x: ")
                                                .speed(0.05),
                                        )
                                        .changed();
                                    dirty |= ui
                                        .add(
                                            egui::DragValue::new(&mut sphere.center[1])
                                                .prefix("y: ")
                                                .speed(0.05),
                                        )
                                        .changed();
                                    dirty |= ui
                                        .add(
                                            egui::DragValue::new(&mut sphere.center[2])
                                                .prefix("z: ")
                                                .speed(0.05),
                                        )
                                        .changed();
                                });

                                // Radius
                                dirty |= ui
                                    .horizontal(|ui| {
                                        ui.label("Radius:");
                                        ui.add(
                                            egui::DragValue::new(&mut sphere.radius)
                                                .speed(0.02)
                                                .range(0.01..=1000.0),
                                        )
                                        .changed()
                                    })
                                    .inner;

                                // Material type selector
                                let current_label = sphere.material.label();
                                let mut selected = MATERIAL_TYPES
                                    .iter()
                                    .position(|&t| t == current_label)
                                    .unwrap_or(0);
                                let mat_changed = ui
                                    .horizontal(|ui| {
                                        ui.label("Material:");
                                        egui::ComboBox::from_id_salt(format!("mat_{i}"))
                                            .selected_text(MATERIAL_TYPES[selected])
                                            .show_index(
                                                ui,
                                                &mut selected,
                                                MATERIAL_TYPES.len(),
                                                |i| MATERIAL_TYPES[i].to_string(),
                                            )
                                            .changed()
                                    })
                                    .inner;

                                if mat_changed {
                                    // Preserve albedo when switching material type
                                    let albedo = *sphere.material.albedo_mut();
                                    sphere.material = match MATERIAL_TYPES[selected] {
                                        "Lambertian" => MaterialParams::Lambertian { albedo },
                                        "Metal" => MaterialParams::Metal { albedo, fuzz: 0.1 },
                                        "Glass" => MaterialParams::Glass {
                                            albedo,
                                            refraction_index: 1.5,
                                        },
                                        "Emissive" => {
                                            MaterialParams::Emissive { albedo, power: 1.0 }
                                        }
                                        _ => unreachable!(),
                                    };
                                    dirty = true;
                                }

                                // Albedo color picker
                                dirty |= ui
                                    .horizontal(|ui| {
                                        ui.label("Color:");
                                        ui.color_edit_button_rgb(sphere.material.albedo_mut())
                                            .changed()
                                    })
                                    .inner;

                                // Material-specific params
                                match &mut sphere.material {
                                    MaterialParams::Lambertian { .. } => {}
                                    MaterialParams::Metal { fuzz, .. } => {
                                        dirty |= ui
                                            .horizontal(|ui| {
                                                ui.label("Fuzz:");
                                                ui.add(egui::Slider::new(fuzz, 0.0..=1.0)).changed()
                                            })
                                            .inner;
                                    }
                                    MaterialParams::Glass {
                                        refraction_index, ..
                                    } => {
                                        dirty |= ui
                                            .horizontal(|ui| {
                                                ui.label("IOR:");
                                                ui.add(egui::Slider::new(
                                                    refraction_index,
                                                    0.1..=3.0,
                                                ))
                                                .changed()
                                            })
                                            .inner;
                                    }
                                    MaterialParams::Emissive { power, .. } => {
                                        dirty |= ui
                                            .horizontal(|ui| {
                                                ui.label("Power:");
                                                ui.add(egui::Slider::new(power, 0.0..=3.0))
                                                    .changed()
                                            })
                                            .inner;
                                    }
                                }

                                // Delete button
                                if ui.button("Delete").clicked() {
                                    delete_idx = Some(i);
                                }

                                ui.add_space(4.0);
                            });
                    }

                    if let Some(idx) = delete_idx {
                        self.scene_params.spheres.remove(idx);
                        dirty = true;
                    }

                    ui.separator();

                    if ui.button("Add Sphere").clicked() {
                        self.scene_params.spheres.push(SphereParams {
                            name: format!("Sphere {}", self.scene_params.spheres.len() + 1),
                            center: [0.0, 0.0, -5.0],
                            radius: 0.5,
                            material: MaterialParams::Lambertian {
                                albedo: [0.8, 0.8, 0.8],
                            },
                        });
                        dirty = true;
                    }
                });
            });

        // ----- Central panel: rendered image -----
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex) = &self.texture {
                let available = ui.available_size();
                let aspect = W as f32 / H as f32;
                let size = if available.x / available.y > aspect {
                    egui::vec2(available.y * aspect, available.y)
                } else {
                    egui::vec2(available.x, available.x / aspect)
                };
                ui.centered_and_justified(|ui| {
                    ui.image(egui::load::SizedTexture::new(tex.id(), size));
                });
            }
        });

        // ----- Apply dirty changes -----
        if dirty {
            // Restart at low resolution for fast feedback while editing
            let pw = W / PREVIEW_SCALE;
            let ph = H / PREVIEW_SCALE;
            self.restart_render(pw, ph);
            self.preview = true;
            self.last_edit = Instant::now();
        } else if self.preview && self.last_edit.elapsed().as_millis() >= PREVIEW_SETTLE_MS {
            // User stopped editing — switch to full resolution
            self.restart_render(W, H);
            self.preview = false;
        }

        ctx.request_repaint();
    }
}
