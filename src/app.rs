use eframe::egui;
use glam::Vec3;
use std::sync::mpsc;
use std::time::Instant;

use crate::{
    camera::Camera,
    color::Color,
    hittable::Sphere,
    material::Material,
    renderer::{self, RenderOptions},
    scene::Scene,
};

const W: usize = 800;
const H: usize = 600;
const PREVIEW_SCALE: usize = 2; // Render at lower resolution during edits
const PREVIEW_SETTLE_MS: u128 = 300; // ms of no edits before switching to full res
const RECURSION_DEPTH: i32 = 16;
const THREAD_COUNT: usize = 4;

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
    for sp in &params.spheres {
        let center = Vec3::new(sp.center[0], sp.center[1], sp.center[2]);
        scene.add_hittable(Sphere::new(center, sp.radius).with_material(sp.material.to_material()));
    }

    (scene, camera)
}

fn default_scene_params() -> SceneParams {
    SceneParams {
        recursion_depth: RECURSION_DEPTH,
        camera: CameraParams {
            position: [0.0, 0.0, 0.0],
            look_at: [0.0, 1.0, -7.0],
            v_fov_deg: 65.0,
            focus_distance: 6.0,
            defocus_angle: 0.0,
        },
        spheres: vec![
            SphereParams {
                name: "Red sphere".into(),
                center: [0.0, 0.0, -7.0],
                radius: 1.0,
                material: MaterialParams::Lambertian {
                    albedo: [0.98, 0.10, 0.12],
                },
            },
            SphereParams {
                name: "Yellow sphere".into(),
                center: [1.6, -0.4, -6.5],
                radius: 0.6,
                material: MaterialParams::Lambertian {
                    albedo: [0.92, 0.94, 0.21],
                },
            },
            SphereParams {
                name: "Green sphere".into(),
                center: [1.2, 2.0, -2.0],
                radius: 1.2,
                material: MaterialParams::Lambertian {
                    albedo: [0.32, 0.94, 0.30],
                },
            },
            SphereParams {
                name: "Black sphere".into(),
                center: [-2.0, 5.0, -15.0],
                radius: 2.0,
                material: MaterialParams::Lambertian {
                    albedo: [0.07, 0.07, 0.07],
                },
            },
            SphereParams {
                name: "Blue metal".into(),
                center: [-3.0, 0.98, -10.0],
                radius: 2.0,
                material: MaterialParams::Metal {
                    albedo: [0.12, 0.10, 0.92],
                    fuzz: 0.1,
                },
            },
            SphereParams {
                name: "Glass outer".into(),
                center: [-0.3, 0.4, -3.2],
                radius: 0.5,
                material: MaterialParams::Glass {
                    albedo: [1.0, 1.0, 1.0],
                    refraction_index: 1.025,
                },
            },
            SphereParams {
                name: "Light source".into(),
                center: [0.45, 0.5, -3.8],
                radius: 0.35,
                material: MaterialParams::Emissive {
                    albedo: [1.0, 1.0, 1.0],
                    power: 2.0,
                },
            },
            SphereParams {
                name: "Glass inner (air)".into(),
                center: [-0.3, 0.4, -3.2],
                radius: 0.45,
                material: MaterialParams::Glass {
                    albedo: [1.0, 1.0, 1.0],
                    refraction_index: 1.0 / 1.025,
                },
            },
            SphereParams {
                name: "White mirror".into(),
                center: [4.0, 1.95, -12.0],
                radius: 3.0,
                material: MaterialParams::Metal {
                    albedo: [0.9, 0.9, 0.9],
                    fuzz: 0.01,
                },
            },
            SphereParams {
                name: "Ground".into(),
                center: [0.0, -501.0, -7.0],
                radius: 500.0,
                material: MaterialParams::Lambertian {
                    albedo: [0.98, 0.92, 0.88],
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

    std::thread::spawn(move || {
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

                    if ui.button("Reset render").clicked() {
                        dirty = true;
                    }

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
                                        .show_index(ui, &mut selected, MATERIAL_TYPES.len(), |i| {
                                            MATERIAL_TYPES[i].to_string()
                                        })
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
                                    "Emissive" => MaterialParams::Emissive { albedo, power: 1.0 },
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
                                            ui.add(egui::Slider::new(refraction_index, 0.1..=3.0))
                                                .changed()
                                        })
                                        .inner;
                                }
                                MaterialParams::Emissive { power, .. } => {
                                    dirty |= ui
                                        .horizontal(|ui| {
                                            ui.label("Power:");
                                            ui.add(egui::Slider::new(power, 0.0..=3.0)).changed()
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
