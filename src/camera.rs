use glam::{Vec2, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub forward: Vec3,
    pub v_fov_deg: f32, // In degrees
    pub aspect: f32,

    pub focus_distance: f32,
    pub defocus_angle: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec3::ZERO,
            forward: -Vec3::Z,
            v_fov_deg: 60.0,
            aspect: 1.0,
            focus_distance: 6.0,
            defocus_angle: 1.5,
        }
    }

    pub fn viewport_size(&self) -> Vec2 {
        let fov_rad = self.v_fov_deg.to_radians();
        // The viewport distance doesn't matter, it's just an arbitrary choice to calculate
        // the ray direction, which will be the same regardless of distance.
        // The FOV and the Aspect is the real way to calculate the viewport size
        // But to make things easier with defocus blur, we make it the same as the focus plane,
        // which actually needs a specific point and a distance instead of just direction
        // height = 2 * viewport_dist * tan(v_fov / 2)
        let viewport_height = 2.0 * self.focus_distance * f32::tan(fov_rad / 2.0);
        let viewport_width = viewport_height * self.aspect;

        Vec2 {
            x: viewport_width,
            y: viewport_height,
        }
    }

    /// Returns the x, y, z axes as unit vectors `(Right, Up, Forward)`
    pub fn axes(&self) -> (Vec3, Vec3, Vec3) {
        let world_up = Vec3::Y;
        let forward = self.forward.normalize();

        // If camera is looking straight up or down, cross product will be zero.
        // In this case, normalization will fail, so we can  fallback to using `Vec3::X`.
        let right = forward.cross(world_up).try_normalize().unwrap_or(Vec3::X);

        let up = right.cross(forward); // No need to normalize this because right and forward are already normalized.

        (right, up, forward)
    }

    pub fn look_at(&mut self, position: Vec3) {
        let forward = position - self.position;
        self.forward = forward.normalize();
    }
}
