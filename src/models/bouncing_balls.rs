use crate::{
    physics::{self, Particle, Plane},
    Model, Solid,
};
use cgmath::{prelude::*, Quaternion, Vector3};

#[derive(Clone)]
pub struct BouncingWorld {
    pub first: Particle,
    pub second: Particle,
}

pub struct BouncingSignals {
    pub b: bool,
}

pub struct BouncingBalls;

const RADIUS: f32 = 0.3;

impl Model for BouncingBalls {
    type World = BouncingWorld;
    type Signals = BouncingSignals;

    fn new_world() -> Self::World {
        Self::World {
            first: Particle::new(
                Vector3::new(-6.0, 4.0, 5.0),
                0.5f32 * Vector3::unit_x(),
                RADIUS,
            ),
            second: Particle::new(
                Vector3::new(0.0, 6.0, 10.0),
                -0.5f32 * Vector3::unit_y(),
                RADIUS,
            ),
        }
    }
    fn new_signals() -> Self::Signals {
        Self::Signals { b: true }
    }

    fn update(world: &mut Self::World, _signals: &Self::Signals) {
        let mut new = physics::time_step_with_rk4(&[world.first, world.second], &(), accels);
        new = physics::time_step_with_rk4(&new, &(), accels);
        new = physics::time_step_with_rk4(&new, &(), accels);
        new = physics::time_step_with_rk4(&new, &(), accels);
        new = physics::time_step_with_rk4(&new, &(), accels);

        world.first = new[0];
        world.second = new[1];

        fn accels(particles: &[Particle], _: &()) -> Vec<Vector3<f32>> {
            const GRAVITY: f32 = 4.0;

            if let &[first, second] = particles {
                vec![
                    Plane::FLOOR.collide_with(&first) - GRAVITY * Vector3::unit_z(),
                    Plane::FLOOR.collide_with(&second) - GRAVITY * Vector3::unit_z(),
                ]
            } else {
                unreachable!()
            }
        }
    }

    fn get_solids(world: &Self::World) -> Vec<Solid> {
        const COLOR: Vector3<f32> = Vector3::new(0.5, 0.5, 0.2);
        const GROUND_COLOR: Vector3<f32> = Vector3::new(0.9, 0.9, 0.9);
        vec![
            Solid::new_sphere(world.first.pos, RADIUS, COLOR),
            Solid::new_sphere(world.second.pos, RADIUS, COLOR),
            Solid::new_rectangular_cuboid(
                Vector3::new(10.0, 10.0, 0.1),
                Vector3::unit_z() * (-0.05),
                Quaternion::one(),
                GROUND_COLOR,
            ),
        ]
    }
}
