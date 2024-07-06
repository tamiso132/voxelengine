use std::{
    collections::HashMap,
    mem::transmute,
    ops::{Add, Mul, Sub},
};

use ash::vk::{self, Extent2D};
use glm::{Mat4, Vec3};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, SmolStr},
};

pub struct Controls {
    letters: [bool; 193],
}
impl Controls {
    pub fn new() -> Controls {
        let letters = [false; 193];

        Self { letters }
    }

    pub fn update_key(&mut self, letter: KeyCode, state: bool) {
        self.letters[letter as usize] = state;
    }

    pub fn get_state(&self, letter: KeyCode) -> bool {
        self.letters[letter as usize]
    }

    pub fn reset_state(&mut self) {
        self.letters = [false; 193];
    }
}

struct Plane {
    normal: Vec3,
    distance: f32,
}

impl Plane {
    pub fn new(p1: Vec3, norm: Vec3) -> Self {
        let distance = norm.dot(p1);
        Self { normal: norm, distance }
    }

    pub fn new_with_distance(distance: f32, norm: Vec3) -> Self {
        Self { normal: norm, distance }
    }

    pub fn get_signed_distance(&self, point: glm::Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }
}

pub struct Frustum {
    right: Plane,
    left: Plane,
    top: Plane,
    bot: Plane,
    far: Plane,
    near: Plane,
}

impl Frustum {
    pub fn new(cam: &Camera) -> Frustum {
        let cam_right = cam.front.cross(cam.up).normalized();
        let up = cam_right.cross(cam.front).normalized();

        let aspect = cam.aspect;

        let h_near_h = cam.near * (cam.fovy * 0.5).tan();
        let w_near_h = h_near_h * aspect;

        // NEAR points
        let near_center = cam.pos + (cam.near) * cam.front;

        // FAR points

        let far_center = cam.pos + cam.far * cam.front;

        // PLANES

        let right_point = near_center + cam_right * w_near_h;
        let right_dir = up.cross((right_point - cam.pos).normalized());

        let left_point = near_center - cam_right * w_near_h;
        let left_dir = (left_point - cam.pos).normalized().cross(up);

        let top_point = near_center + up * h_near_h;
        let top_dir = (top_point - cam.pos).normalized().cross(cam_right);

        let bot_point = top_point - up * h_near_h * 2.0;
        let bot_dir = cam_right.cross((bot_point - cam.pos).normalized());

        let near = Plane::new(near_center, cam.front);
        let far = Plane::new(far_center, -cam.front);
        let right = Plane::new(right_point, right_dir);
        let left = Plane::new(left_point, left_dir);
        let top = Plane::new(top_point, top_dir);
        let bot = Plane::new(bot_point, bot_dir);

        Self { right, left, top, bot, far, near }
    }

    fn in_plane(plane: &Plane, pos: Vec3) -> bool {
        let mut p = Vec3::new(pos.x - 0.5, pos.y - 0.5, pos.z - 0.5);

        let mut n = Vec3::new(pos.x + 0.5, pos.y + 0.5, pos.z + 0.5);
        //let from_center = 0.5;
        // let points = vec![
        //     Vec3::new(pos.x - from_center, pos.y + from_center, pos.z + from_center),
        //     Vec3::new(pos.x + from_center, pos.y + from_center, pos.z + from_center),
        //     Vec3::new(pos.x - from_center, pos.y - from_center, pos.z + from_center),
        //     Vec3::new(pos.x + from_center, pos.y - from_center, pos.z + from_center),
        //     Vec3::new(pos.x - from_center, pos.y + from_center, pos.z - from_center),
        //     Vec3::new(pos.x + from_center, pos.y + from_center, pos.z - from_center),
        //     Vec3::new(pos.x - from_center, pos.y - from_center, pos.z - from_center),
        //     Vec3::new(pos.x + from_center, pos.y - from_center, pos.z - from_center),
        //     Vec3::new(pos.x + from_center, pos.y + from_center, pos.z - from_center),
        //     Vec3::new(pos.x + from_center, pos.y + from_center, pos.z + from_center),
        //     Vec3::new(pos.x + from_center, pos.y - from_center, pos.z - from_center),
        //     Vec3::new(pos.x + from_center, pos.y - from_center, pos.z + from_center),
        //     Vec3::new(pos.x - from_center, pos.y + from_center, pos.z - from_center),
        //     Vec3::new(pos.x - from_center, pos.y + from_center, pos.z + from_center),
        //     Vec3::new(pos.x - from_center, pos.y - from_center, pos.z - from_center),
        //     Vec3::new(pos.x - from_center, pos.y - from_center, pos.z + from_center),
        // ];

        if plane.normal.x >= 0.0 {
            p.x += 1.0;
            n.x -= 1.0;
        }

        if plane.normal.y >= 0.0 {
            p.z += 1.0;
            n.z -= 1.0;
        }
        if plane.normal.z >= 0.0 {
            p.z += 1.0;
            n.z -= 1.0;
        }

        // if !(plane.get_signed_distance(n) < 0.0) {
        //     return true;
        // }
        if !(plane.get_signed_distance(p) < 0.0) {
            return true;
        }

        return false;
    }

    pub fn is_inside(&self, position: Vec3) -> bool {
        if !Self::in_plane(&self.right, position) {
            return false;
        }

        if !Self::in_plane(&self.left, position) {
            return false;
        }

        if !Self::in_plane(&self.top, position) {
            return false;
        }

        if !Self::in_plane(&self.bot, position) {
            return false;
        }
        if !Self::in_plane(&self.far, position) {
            return false;
        }
        if !Self::in_plane(&self.near, position) {
            return false;
        }
        return true;
    }
}

#[repr(C, align(16))]
pub struct GPUCamera {
    viewproj: Mat4,
    pos: Vec3,
}
#[derive(Debug)]
pub struct Camera {
    pub pos: glm::Vec3,
    front: glm::Vec3,
    up: glm::Vec3,

    pub extent: vk::Extent2D,
    projection: glm::Mat4,

    yaw: f32,
    pitch: f32,

    pub fovy: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(extent: vk::Extent2D) -> Self {
        let aspect = extent.width as f32 / extent.height as f32;
        let fovy = f32::from(45.0).to_radians();
        let near = 0.1;
        let far = 200.0;
        let yaw = 0.0;
        let pitch = 0.0;

        let projection: glm::Mat4 = glm::projection::perspective_vk(fovy, aspect, near, far);

        Self {
            pos: Vec3::new(0.0, 0.0, 0.0),
            front: Vec3::new(0.0, 0.0, 1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            extent,
            projection,
            yaw,
            pitch,
            fovy,
            near,
            far,
            aspect,
        }
    }

    pub fn resize_window(&mut self, extent: Extent2D) {
        self.aspect = extent.width as f32 / extent.height as f32;
        self.extent = extent;
        self.projection = glm::projection::perspective_vk(self.fovy, self.aspect, self.near, self.far);

        println!("extent {:?}\n", extent);
    }

    pub fn process_keyboard(&mut self, controls: &Controls, delta_time: f64) {
        let mut speed_mul = 6.0;

        if controls.get_state(KeyCode::ControlLeft) {
            speed_mul = 20.0;
        }

        let cam_speed = Vec3::new(speed_mul * delta_time as f32, speed_mul * delta_time as f32, speed_mul * delta_time as f32);
        if controls.get_state(KeyCode::KeyW) {
            self.pos += cam_speed * self.front;
        }

        if controls.get_state(KeyCode::KeyS) {
            self.pos -= cam_speed * self.front;
        }

        if controls.get_state(KeyCode::KeyD) {
            self.pos += self.front.cross(self.up).normalized() * cam_speed;
        }
        if controls.get_state(KeyCode::KeyA) {
            self.pos -= self.front.cross(self.up).normalized() * cam_speed;
        }
    }

    pub fn process_mouse(&mut self, mut mouse_delta: (f64, f64)) {
        let sensitivity = 0.06;

        mouse_delta = (mouse_delta.0 * sensitivity, mouse_delta.1 * sensitivity);

        self.yaw += mouse_delta.0 as f32;
        self.pitch += mouse_delta.1 as f32 * -1.0;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        } else if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        self.front.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        self.front.y = self.pitch.to_radians().sin();
        self.front.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();

        self.front.normalize();
    }

    pub fn get_view(&self) -> glm::Mat4 {
        Mat4::look_at(self.pos, self.pos + self.front, self.up)
    }

    pub fn ortho(max_right: f32, max_top: f32) -> glm::Mat4 {
        glm::projection::orthographic_vk(0.0, max_right, 0.0, max_top, -1.0, 1.0)
    }

    pub fn get_projection(&self) -> glm::Mat4 {
        self.projection
    }

    pub fn get_pos(&self) -> glm::Vec3 {
        self.pos
    }

    pub fn get_gpu_camera(&self) -> GPUCamera {
        let viewproj = self.get_projection().mul(self.get_view());

        GPUCamera { viewproj, pos: self.pos }
    }

    pub fn get_shader_format(&self) -> GPUCamera {
        let view = self.get_view();

        let view_proj = view * self.projection;

        GPUCamera { viewproj: view_proj, pos: self.pos }
    }
}
