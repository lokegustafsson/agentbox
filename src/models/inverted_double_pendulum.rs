use crate::{
    common::{Cuboid, Cylinder, Sphere},
    models::Model,
};
use cgmath::{prelude::*, Vector2, Vector3};

#[derive(Clone)]
pub struct IDPWorld {
    pub bottom_pos: Vector2<f32>,
    pub bottom_vel: Vector2<f32>,

    pub middle_pos: Vector3<f32>,
    pub middle_vel: Vector3<f32>,
    pub top_pos: Vector3<f32>,
    pub top_vel: Vector3<f32>,
}

pub struct IDPSignals {
    pub bottom_accel: Vector2<f32>,
}

pub struct InvertedDoublePendulum;

impl Model for InvertedDoublePendulum {
    type World = IDPWorld;
    type Signals = IDPSignals;

    fn new_world() -> Self::World {
        let disturbance = Vector3::new(0.04, 0.03, -0.01);
        let unit_z = Vector3::unit_z();
        Self::World {
            bottom_pos: Zero::zero(),
            bottom_vel: Zero::zero(),

            middle_pos: unit_z + disturbance,
            middle_vel: Zero::zero(),

            top_pos: unit_z * 2.0,
            top_vel: Zero::zero(),
        }
    }
    fn new_signals() -> Self::Signals {
        Self::Signals {
            bottom_accel: Zero::zero(),
        }
    }

    fn update(world: &mut Self::World, signals: &Self::Signals) {
        todo!()
    }

    fn get_solids(world: &Self::World) -> (Vec<Sphere>, Vec<Cylinder>, Vec<Cuboid>) {
        const CONTROL_COLOR: Vector3<f32> = Vector3::new(0.0, 0.5, 0.3);
        const NODE_COLOR: Vector3<f32> = Vector3::new(0.5, 0.2, 0.3);
        const ROD_COLOR: Vector3<f32> = Vector3::new(0.0, 0.3, 0.6);

        const NODE_RADIUS: f32 = 0.15;
        const ROD_RADIUS: f32 = 0.1;

        (
            vec![
                Sphere::new(world.bottom_pos.extend(0.0), NODE_RADIUS, CONTROL_COLOR),
                Sphere::new(world.middle_pos, NODE_RADIUS, NODE_COLOR),
                Sphere::new(world.top_pos, NODE_RADIUS, NODE_COLOR),
            ],
            vec![
                Cylinder::new(
                    world.bottom_pos.extend(0.0),
                    world.middle_pos,
                    ROD_RADIUS,
                    ROD_COLOR,
                ),
                Cylinder::new(world.middle_pos, world.top_pos, ROD_RADIUS, ROD_COLOR),
            ],
            Vec::new(),
        )
    }
}
