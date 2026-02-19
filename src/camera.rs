use glam::{Vec2, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub forward: Vec3,
    pub v_fov: f32, // In degrees
    pub aspect: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec3::ZERO,
            forward: -Vec3::Z,
            v_fov: 60.0,
            aspect: 1.0,
        }
    }

    pub fn viewport_size(&self) -> Vec2 {
        let fov_rad = self.v_fov.to_radians();
        // The viewport distance doesn't matter, it's just an arbitrary choice.
        // The FOV and the Aspect is the real way to calculate the viewport size
        // So we just pick the distance as 1 to simplify things.
        // height = 2 * viewport_dist * tan(v_fov / 2)
        let viewport_height = 2.0 * f32::tan(fov_rad / 2.0);
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
        // TODO: If camera is looking straight up or down, cross product will be zero. Just make `right = Vec3::X` in this case
        let right = forward.cross(world_up).normalize();
        let up = right.cross(forward); // No need to normalize this because right and forward are already normalized.

        (right, up, forward)
    }

    pub fn look_at(&mut self, position: Vec3) {
        let forward = position - self.position;
        self.forward = forward.normalize();
    }
}
