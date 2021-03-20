//! A toolbox for implementing the update part of a model.

use cgmath::{prelude::*, Vector3};

const DT: f32 = 0.01;

#[derive(Clone)]
pub struct Particle {
    pub pos: Vector3<f32>,
    pub vel: Vector3<f32>,
}
impl Particle {
    pub fn new(pos: Vector3<f32>, vel: Vector3<f32>) -> Self {
        Self { pos, vel }
    }
    pub fn spring_accel_from(&self, other: &Particle, spring: &Spring) -> Vector3<f32> {
        let rel_pos = self.pos - other.pos;
        let radial_distance = rel_pos.magnitude();
        let inverse_radial_distance = 1.0 / radial_distance;
        let radial_vel = (self.vel - other.vel).dot(rel_pos) * inverse_radial_distance;
        let radial_force =
            spring.stiffness * (spring.rest_length - radial_distance) - spring.damping * radial_vel;

        (radial_force * inverse_radial_distance) * rel_pos
    }
}
impl Default for Particle {
    fn default() -> Self {
        Self {
            pos: Vector3::zero(),
            vel: Vector3::zero(),
        }
    }
}

pub struct Spring {
    pub stiffness: f32,
    pub damping: f32,
    pub rest_length: f32,
}

impl Spring {
    pub const UNIT_ROD: &'static Self = &Self {
        stiffness: 1000.0,
        damping: 10.0,
        rest_length: 1.0,
    };
}

pub fn time_step_with_rk4<T>(
    particles: &[Particle],
    extra_state: &T,
    accelerations: impl Fn(&[Particle], &T) -> Vec<Vector3<f32>>,
) -> Vec<Particle> {
    let mut new_particles = vec![Particle::default(); particles.len()];

    let a0s = accelerations(particles, extra_state);
    assert_eq!(particles.len(), a0s.len());
    for ((new, old), a0) in new_particles.iter_mut().zip(particles).zip(a0s.iter()) {
        new.pos = old.pos + old.vel * (DT / 2.0);
        new.vel = old.vel + a0 * (DT / 2.0);
    }

    let a1s = accelerations(&new_particles, extra_state);
    assert_eq!(particles.len(), a1s.len());
    for ((new, old), a1) in new_particles.iter_mut().zip(particles).zip(a1s.iter()) {
        // Data dependency (current) or recompute?
        new.pos = old.pos + new.vel * (DT / 2.0);
        new.vel = old.vel + a1 * (DT / 2.0);
    }

    let a2s = accelerations(&new_particles, extra_state);
    assert_eq!(particles.len(), a2s.len());
    for ((new, old), a2) in new_particles.iter_mut().zip(particles).zip(&a2s) {
        // Data dependency (current) or recompute?
        new.pos = old.pos + new.vel * DT;
        new.vel = old.vel + a2 * DT;
    }
    let a3s = accelerations(&new_particles, extra_state);
    assert_eq!(particles.len(), a3s.len());

    for ((new, old), (((a0, a1), a2), a3)) in new_particles
        .iter_mut()
        .zip(particles)
        .zip(a0s.iter().zip(a1s).zip(a2s).zip(a3s))
    {
        let a012 = a0 + a1 + a2;
        let a123 = a1 + a2 + a3;
        new.pos = old.pos + old.vel * DT + a012 * (DT * DT / 4.0);
        new.vel = old.vel + (a012 + a123) * (DT / 6.0);
    }
    new_particles
}
