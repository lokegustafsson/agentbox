use crate::{common::Solid, models::Model};
use cgmath::{prelude::*, Vector3};

#[derive(Clone)]
pub struct SimpleWorld {
    pub pos: Vector3<f32>,
    pub vel: Vector3<f32>,
    pub color: Vector3<f32>,
}

pub struct SimpleSignals {
    pub accel: Vector3<f32>,
    pub target_color: Vector3<f32>,
}

pub struct SimpleModel;

impl Model for SimpleModel {
    type World = SimpleWorld;
    type Signals = SimpleSignals;

    fn new_world() -> Self::World {
        Self::World {
            pos: Vector3::zero(),
            vel: Vector3::zero(),
            color: Vector3::unit_x(),
        }
    }
    fn new_signals() -> Self::Signals {
        Self::Signals {
            accel: Vector3::zero(),
            target_color: Vector3::unit_y(),
        }
    }

    fn update(world: &mut Self::World, signals: &Self::Signals) {
        let dt = 0.01;

        world.pos += world.vel * dt + signals.accel * (dt * dt / 2.0);
        world.vel += dt * signals.accel;
        world.color = (1.0 - dt) * world.color + dt * signals.target_color;
    }

    fn get_solids(world: &Self::World) -> Vec<Solid> {
        const RADIUS: f32 = 1.0;
        vec![Solid::new_sphere(world.pos, RADIUS, world.color)]
    }
}
