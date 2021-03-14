use crate::models::Model;
use cgmath::{prelude::*, Matrix3, Matrix4, Rad, Vector3, Vector4};
use std::sync::{atomic::AtomicUsize, Arc, Mutex};

// Communication between the event loop and the simulation thread

pub struct WorldChannel<M: Model> {
    pub world: Mutex<Arc<M::World>>,
    pub version: AtomicUsize,
}

impl<M: Model> WorldChannel<M> {
    pub fn new() -> Self {
        Self {
            world: Mutex::new(Arc::new(M::new_world())),
            version: AtomicUsize::new(0),
        }
    }
}

#[derive(Debug)]
pub enum SimulationEvent {
    RequestExit,
    RequestHide,
    RequestShow,
    SimulationPanic,
}

// Structs that will be sent to the GPU

pub enum SolidKind {
    Sphere,
    Cylinder,
    Cube,
}

impl SolidKind {
    const SPHERE: f32 = 1.0;
    const CYLINDER: f32 = 2.0;
    const CUBE: f32 = 4.0;
    pub(super) const fn to_f32(self) -> f32 {
        match self {
            SolidKind::Sphere => Self::SPHERE,
            SolidKind::Cylinder => Self::CYLINDER,
            SolidKind::Cube => Self::CUBE,
        }
    }
    pub(super) fn from_f32(f: f32) -> Self {
        use SolidKind::*;
        if f == Self::SPHERE {
            Sphere
        } else if f == Self::CYLINDER {
            Cylinder
        } else if f == Self::CUBE {
            Cube
        } else {
            panic!("Bad float input")
        }
    }
}

// [--- --- --- ---
//  --- matrix  ---
//  --- --- --- ---
//  -- color - kind]
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Solid(Matrix4<f32>);

unsafe impl bytemuck::Pod for Solid {}
unsafe impl bytemuck::Zeroable for Solid {}

impl Solid {
    fn new(world_to_local: Matrix4<f32>, color: Vector3<f32>, kind: SolidKind) -> Self {
        cgmath::assert_relative_eq!(world_to_local.row(3), Vector4::unit_w(),);
        assert!(
            world_to_local.is_invertible(),
            "world_to_local must be invertible: {:?}",
            world_to_local
        );

        let mut matrix = world_to_local;
        matrix.x.w = color.x;
        matrix.y.w = color.y;
        matrix.z.w = color.z;
        matrix.w.w = kind.to_f32();
        Self(matrix)
    }

    pub fn new_sphere(pos: Vector3<f32>, radius: f32, color: Vector3<f32>) -> Self {
        let world_to_local = Matrix4::from_scale(1.0 / radius) * Matrix4::from_translation(-pos);
        Self::new(world_to_local, color, SolidKind::Sphere)
    }
    pub fn new_cylinder(
        first: Vector3<f32>,
        second: Vector3<f32>,
        radius: f32,
        color: Vector3<f32>,
    ) -> Self {
        let midpoint: Vector3<f32> = (first + second) / 2.0;
        let axis = first - midpoint;
        let length_scale = axis.magnitude();
        let world_to_local =
            Matrix4::from_nonuniform_scale(1.0 / radius, 1.0 / radius, 1.0 / length_scale)
                * Matrix4::from_axis_angle(
                    axis.cross(Vector3::unit_z()).normalize(),
                    Rad::acos(Vector3::unit_z().dot(axis / length_scale)),
                )
                * Matrix4::from_translation(-midpoint);
        Self::new(world_to_local, color, SolidKind::Cylinder)
    }

    pub fn kind(&self) -> SolidKind {
        SolidKind::from_f32(self.0.w.w)
    }
    pub fn world_to_local(self) -> Matrix4<f32> {
        let mut matrix = self.0;
        matrix.x.w = 0.0;
        matrix.y.w = 0.0;
        matrix.z.w = 0.0;
        matrix.w.w = 1.0;
        matrix
    }

    pub(crate) fn bounding_sphere(&self) -> (Vector3<f32>, f32) {
        let local_to_world = self.world_to_local().invert().unwrap();
        // local_to_world can be decomposed as a 3d linear transformation [linear], then a translation [pos]
        let pos = local_to_world.w.truncate();
        let linear = Matrix3::from_cols(
            local_to_world.x.truncate(),
            local_to_world.y.truncate(),
            local_to_world.z.truncate(),
        );
        // The object must be bounded by the cube with side length 2. Let's use a radius that is
        // sufficient to enclose the parallelepiped image of that cube under [linear]
        let radius_squared = &[
            Vector3::new(1.0, 1.0, 1.0),
            Vector3::new(1.0, 1.0, -1.0),
            Vector3::new(1.0, -1.0, 1.0),
            Vector3::new(1.0, -1.0, -1.0),
        ]
        .iter()
        .map(|v| (linear * v).magnitude2())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

        (pos, radius_squared.sqrt())
    }

    pub(crate) fn assert_valid(&self) {
        let _ = SolidKind::from_f32(self.0.w.w);
        assert!(self.0.is_finite());
        assert!(self.0.is_invertible());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn properly_bound_sphere(pos: Vector3<f32>, radius: f32) {
        let inner = Solid::new_sphere(pos, radius, Vector3::unit_x());
        let (bound_pos, bound_radius) = inner.bounding_sphere();
        cgmath::assert_relative_eq!(bound_pos, pos);
        assert!(bound_radius >= radius);
    }

    #[test]
    fn properly_bound_sphere_origin_1() {
        properly_bound_sphere(Vector3::zero(), 1.0);
    }
    #[test]
    fn properly_bound_sphere_origin_large() {
        properly_bound_sphere(Vector3::zero(), 123.0);
    }
    #[test]
    fn properly_bound_sphere_origin_small() {
        properly_bound_sphere(Vector3::zero(), 1.0 / 123.0);
    }
    #[test]
    fn properly_bound_sphere_elsewhere_small() {
        properly_bound_sphere(Vector3::unit_x(), 1.0 / 123.0);
    }
    #[test]
    fn properly_bound_sphere_elsewhere_1() {
        properly_bound_sphere(2.3f32 * Vector3::unit_x(), 1.0);
    }
    #[test]
    fn properly_bound_sphere_elsewhere_large() {
        properly_bound_sphere(100.0f32 * Vector3::unit_x(), 123.0);
    }
}
