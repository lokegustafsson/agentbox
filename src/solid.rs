use cgmath::{prelude::*, Matrix3, Matrix4, Quaternion, Rad, Vector3, Vector4};

/// The graphical primitive. A solid can represent any affine transformation of
/// a sphere, cylinder or cube.
// Memory representation:
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
    /// Create a sphere centered on `pos`, with radius `radius` and rgb-color `color`.
    pub fn new_sphere(pos: Vector3<f32>, radius: f32, color: Vector3<f32>) -> Self {
        let world_to_local = Matrix4::from_scale(1.0 / radius) * Matrix4::from_translation(-pos);
        Self::new(world_to_local, color, SolidKind::Sphere)
    }
    /// Create a cylinder with radius `radius` and rgb-color `color`, where `first`
    /// and `second` are the centers of the base disk and top disk respectively.
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
    /// Create a rectangular cuboid with width-depth-height given by `dimensions`
    pub fn new_rectangular_cuboid(
        dimensions: Vector3<f32>,
        center: Vector3<f32>,
        orientation: Quaternion<f32>,
        color: Vector3<f32>,
    ) -> Self {
        let world_to_local = Matrix4::from_nonuniform_scale(
            2.0 / dimensions.x,
            2.0 / dimensions.y,
            2.0 / dimensions.z,
        ) * Matrix4::from(orientation.conjugate())
            * Matrix4::from_translation(-center);
        Self::new(world_to_local, color, SolidKind::Cube)
    }

    fn world_to_local(self) -> Matrix4<f32> {
        let mut matrix = self.0;
        matrix.x.w = 0.0;
        matrix.y.w = 0.0;
        matrix.z.w = 0.0;
        matrix.w.w = 1.0;
        matrix
    }

    /// Returns (min, max) for the AABB
    pub(crate) fn bounding_aabb(&self) -> (Vector3<f32>, Vector3<f32>) {
        let local_to_world = self.world_to_local().invert().unwrap();
        // local_to_world can be decomposed as a 3d linear transformation [linear],
        // then a translation [center]
        let center = local_to_world.w.truncate();
        let linear = Matrix3::from_cols(
            local_to_world.x.truncate(),
            local_to_world.y.truncate(),
            local_to_world.z.truncate(),
        );
        // The object must (pre-linear transformation) be bounded by the cube with side
        // length 2, centered on the origin. Let's use the AABB encapsulating it,
        // irrespective of the SolidKind.
        let extent = &[
            Vector3::new(1.0, 1.0, 1.0),
            Vector3::new(1.0, 1.0, -1.0),
            Vector3::new(1.0, -1.0, 1.0),
            Vector3::new(1.0, -1.0, -1.0),
        ]
        .iter()
        .map(|v| (linear * v).map(f32::abs))
        .reduce(|a, b| Vector3::new(f32::max(a.x, b.x), f32::max(a.y, b.y), f32::max(a.z, b.z)))
        .unwrap();

        (center - extent, center + extent)
    }

    pub(crate) fn assert_valid(&self) {
        let _ = SolidKind::from_f32(self.0.w.w);
        assert!(self.0.is_finite());
        assert!(self.0.is_invertible());
    }
}

enum SolidKind {
    Sphere,
    Cylinder,
    Cube,
}

impl SolidKind {
    const SPHERE: f32 = 1.0;
    const CYLINDER: f32 = 2.0;
    const CUBE: f32 = 4.0;
    pub fn to_f32(self) -> f32 {
        match self {
            SolidKind::Sphere => Self::SPHERE,
            SolidKind::Cylinder => Self::CYLINDER,
            SolidKind::Cube => Self::CUBE,
        }
    }
    pub fn from_f32(f: f32) -> Self {
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
