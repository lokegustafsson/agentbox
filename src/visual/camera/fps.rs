use crate::visual::camera::{Camera, Choice};
use cgmath::{prelude::*, Matrix4, Rad, Vector3};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

const SENSITIVITY: Rad<f32> = Rad(0.0006);

pub struct FpsCamera {
    pos: Vector3<f32>,
    angle_equator: Rad<f32>,
    angle_meridian: Rad<f32>,

    forwards: Choice,
    right: Choice,
    up: Choice,
    aim_right: f32,
    aim_up: f32,
}

impl Camera for FpsCamera {
    fn new() -> Self {
        Self {
            pos: Vector3::unit_x(),
            angle_equator: Rad::zero(),
            angle_meridian: Rad::zero(),

            forwards: Choice::Neither,
            right: Choice::Neither,
            up: Choice::Neither,
            aim_right: 0.0,
            aim_up: 0.0,
        }
    }
    fn update(&mut self, delta_pos: f32) {
        let velocity = {
            let unit_right =
                Vector3::new(-self.angle_meridian.sin(), self.angle_meridian.cos(), 0.0);
            let unit_forward =
                Vector3::new(-self.angle_meridian.cos(), -self.angle_meridian.sin(), 0.0);

            self.right.f32() * unit_right
                + self.forwards.f32() * unit_forward
                + self.up.f32() * Vector3::unit_z()
        };

        if velocity.magnitude2() > 0.1 {
            self.pos += velocity.normalize() * delta_pos;
        }
        self.angle_meridian -= SENSITIVITY * self.aim_right;
        self.angle_equator -= SENSITIVITY * self.aim_up;

        self.aim_right = 0.0;
        self.aim_up = 0.0;
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
            _ => {}
        }
    }
    fn mouse_input(&mut self, x: f64, y: f64, w: u32, h: u32) {
        let (center_x, center_y) = (w as f32 / 2.0, h as f32 / 2.0);
        self.aim_right += x as f32 - center_x;
        self.aim_up -= y as f32 - center_y;
    }
    fn camera_to_world(&mut self) -> Matrix4<f32> {
        let z_image = Vector3::new(
            -self.angle_equator.cos() * self.angle_meridian.cos(),
            -self.angle_equator.cos() * self.angle_meridian.sin(),
            -self.angle_equator.sin(),
        );
        let x_image = Vector3::new(-self.angle_meridian.sin(), self.angle_meridian.cos(), 0.0);
        let y_image = Vector3::new(
            self.angle_equator.sin() * self.angle_meridian.cos(),
            self.angle_equator.sin() * self.angle_meridian.sin(),
            -self.angle_equator.cos(),
        );
        Matrix4::from_cols(
            x_image.extend(0.0),
            y_image.extend(0.0),
            z_image.extend(0.0),
            self.pos.extend(1.0),
        )
    }
}
