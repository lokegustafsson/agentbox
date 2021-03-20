use crate::{
    physics::{self, Particle},
    Model, Solid,
};
use cgmath::{prelude::*, Vector2, Vector3};

#[derive(Clone)]
pub struct IDPWorld {
    pub base_pos: Vector2<f32>,
    pub base_vel: Vector2<f32>,

    pub mid_pos: Vector3<f32>,
    pub mid_vel: Vector3<f32>,
    pub top_pos: Vector3<f32>,
    pub top_vel: Vector3<f32>,
}

pub struct IDPSignals {
    pub base_accel: Vector2<f32>,
}

pub struct InvertedDoublePendulum;

impl Model for InvertedDoublePendulum {
    type World = IDPWorld;
    type Signals = IDPSignals;

    fn new_world() -> Self::World {
        let disturbance = Vector3::new(0.04, 0.03, -0.01);
        let unit_z = Vector3::unit_z();
        Self::World {
            base_pos: Zero::zero(),
            base_vel: Zero::zero(),

            mid_pos: unit_z + disturbance,
            mid_vel: Zero::zero(),

            top_pos: unit_z * 2.0,
            top_vel: Zero::zero(),
        }
    }
    fn new_signals() -> Self::Signals {
        Self::Signals {
            base_accel: Zero::zero(),
        }
    }

    fn update(w: &mut Self::World, signals: &Self::Signals) {
        let particles = [
            Particle::new(w.base_pos.extend(0.0), w.base_vel.extend(0.0)),
            Particle::new(w.mid_pos, w.mid_vel),
            Particle::new(w.top_pos, w.top_vel),
        ];
        let new = physics::runge_kutta(&particles, signals, idp_accels);

        w.base_pos = new[0].pos.truncate();
        w.base_vel = new[0].vel.truncate();
        w.mid_pos = new[1].pos;
        w.mid_vel = new[1].vel;
        w.top_pos = new[2].pos;
        w.top_vel = new[2].vel;

        fn idp_accels(particles: &[Particle], signals: &IDPSignals) -> Vec<Vector3<f32>> {
            const GRAVITY_ACCEL: f32 = 0.3;
            if let [base, mid, top] = particles {
                let mid_accel = physics::damped_spring_force(mid.pos, mid.vel, top.pos, top.vel)
                    + physics::damped_spring_force(mid.pos, mid.vel, base.pos, base.vel)
                    - Vector3::unit_z() * GRAVITY_ACCEL;

                let top_accel = physics::damped_spring_force(top.pos, top.vel, mid.pos, mid.vel)
                    - Vector3::unit_z() * GRAVITY_ACCEL;

                vec![signals.base_accel.extend(0.0), mid_accel, top_accel]
            } else {
                unreachable!()
            }
        }
    }

    fn get_solids(world: &Self::World) -> Vec<Solid> {
        const CONTROL_COLOR: Vector3<f32> = Vector3::new(0.0, 0.5, 0.3);
        const NODE_COLOR: Vector3<f32> = Vector3::new(0.5, 0.2, 0.3);
        const ROD_COLOR: Vector3<f32> = Vector3::new(0.0, 0.3, 0.6);

        const NODE_RADIUS: f32 = 0.15;
        const ROD_RADIUS: f32 = 0.1;

        vec![
            Solid::new_sphere(world.base_pos.extend(0.0), NODE_RADIUS, CONTROL_COLOR),
            Solid::new_sphere(world.mid_pos, NODE_RADIUS, NODE_COLOR),
            Solid::new_sphere(world.top_pos, NODE_RADIUS, NODE_COLOR),
            Solid::new_cylinder(
                world.base_pos.extend(0.0),
                world.mid_pos,
                ROD_RADIUS,
                ROD_COLOR,
            ),
            Solid::new_cylinder(world.mid_pos, world.top_pos, ROD_RADIUS, ROD_COLOR),
        ]
    }
}
