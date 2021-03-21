use crate::visual::camera::{Camera, Choice};
use cgmath::{prelude::*, Matrix4, Quaternion, Rad, Vector3};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

const ROLL_RATE: f32 = 0.4;
const SENSITIVITY: f32 = 0.001;

pub struct AircraftCamera {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,

    forwards: Choice,
    right: Choice,
    up: Choice,
    roll_right: Choice,
    pitch_up: f32,
    yaw_right: f32,
}

impl Camera for AircraftCamera {
    fn new() -> Self {
        Self {
            position: -2.0f32 * Vector3::unit_y(),
            rotation: Quaternion::from_arc(Vector3::unit_z(), Vector3::unit_y(), None),
            forwards: Choice::Neither,
            right: Choice::Neither,
            up: Choice::Neither,
            roll_right: Choice::Neither,
            pitch_up: 0.0,
            yaw_right: 0.0,
        }
    }
    fn update(&mut self, delta_pos: f32) {
        // Velocity in camera space (x - right, y - down, z - forwards)
        let velocity = Vector3::new(self.right.f32(), -self.up.f32(), self.forwards.f32());

        if velocity.magnitude2() > 0.1 {
            self.position += self
                .rotation
                .rotate_vector(velocity.normalize() * delta_pos);
        }
        self.rotation = self.rotation
            * Quaternion::from_axis_angle(
                Vector3::unit_z(),
                Rad(ROLL_RATE * self.roll_right.f32() * delta_pos),
            )
            * Quaternion::from_axis_angle(Vector3::unit_x(), Rad(self.pitch_up))
            * Quaternion::from_axis_angle(Vector3::unit_y(), Rad(self.yaw_right));
        self.pitch_up = 0.0;
        self.yaw_right = 0.0;
    }
    fn key_input(&mut self, key: KeyboardInput) {
        use VirtualKeyCode::*;
        if key.virtual_keycode.is_none() {
            return;
        }
        let active = key.state == ElementState::Pressed;
        match key.virtual_keycode.unwrap() {
            W => self.forwards.go(active),
            S => self.forwards.reverse(active),
            D => self.right.go(active),
            A => self.right.reverse(active),
            Space => self.up.go(active),
            LShift => self.up.reverse(active),
            E => self.roll_right.go(active),
            Q => self.roll_right.go(active),
            _ => {}
        }
    }
    fn mouse_input(&mut self, x: f64, y: f64, w: u32, h: u32) {
        let (mx, my) = (w as f32 / 2.0, h as f32 / 2.0);
        self.pitch_up -= SENSITIVITY * (y as f32 - my);
        self.yaw_right += SENSITIVITY * (x as f32 - mx);
    }
    fn camera_to_world(&mut self) -> Matrix4<f32> {
        let rot = Matrix4::from(self.rotation);
        let trans = Matrix4::from_translation(self.position);
        trans * rot
    }
}
