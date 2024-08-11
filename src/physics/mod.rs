use std::ops::Mul;

use glm::Vec3;

struct Physic3D {
    velocity: Vec3,
    acceleration: Vec3,
    position: Vec3,
    mass: u32,
}

impl Physic3D {
    pub fn update_position(&mut self, delta: f32) {
        self.velocity = self.velocity + self.acceleration.mul(delta);
        self.position = self.position + self.velocity.mul(delta);
    }
}
