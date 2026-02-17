use glam::{Vec2, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub forward: Vec3,
    pub viewport_distance: f32,

    // We don't need to store a viewport size, because the viewport height will be 1, and width will be aspect
    pub aspect: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec3::ZERO,
            forward: -Vec3::Z,
            viewport_distance: 1.0,
            aspect: 1.0,
        }
    }

    // Idk if this method is even necessary, we can just set `cam.aspect = ...` directly.
    // I guess it's a nice way to initialize without making the whole var `mut`, but we will
    // mutate the camera anyway (change the position and direction) so this will be useless.
    pub fn with_aspect(mut self, aspect: f32) -> Self {
        self.aspect = aspect;
        self
    }

    pub fn viewport_size(&self) -> Vec2 {
        Vec2 {
            x: self.aspect,
            y: 1.0,
        }
    }

    /// Returns the x, y, z axes as unit vectors `(Right, Up, Forward)`
    pub fn axes(&self) -> (Vec3, Vec3, Vec3) {
        let world_up = Vec3::Y;
        let forward = self.forward.normalize();
        // TODO: If camera is looking straight up or down, cross product will be zero. Just make `right` +X in this case
        let right = forward.cross(world_up).normalize();
        let up = right.cross(forward); // No need to normalize this because right and forward are already normalized.

        (right, up, forward)
    }
}
