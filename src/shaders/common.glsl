
// Buffer items need their size to be a multiple of 16 bytes. This struct is 32 bytes.
struct BoundingBallNode {
    vec3 pos;
    float radius;
    // Doubles as kind indicator if negative: -1 => Sphere, -2 => Cylinder, -3 => Cuboid
    int left;
    // Doubles as index if [left < 0]
    int right;

    int _padding1;
    int _padding2;
};
// This struct is 32 bytes
struct Sphere {
    vec3 pos; // Sphere: center, Cylinder: a face center, A cuboid corner
    float radius;

    vec3 color;

    int _padding;
};
// This struct is 48 bytes
struct Cylinder {
    vec3 faceA;
    int _padding1;

    vec3 faceB;
    float radius;

    vec3 color;
    int _padding2;
};
// This struct is 64 bytes
struct Cuboid {
    vec3 corner;
    vec3 axisA;
    vec3 axisB;
    float width; // Size in last (axisA cross axisB) dimension

    vec3 color;
};
