
const uint LEAF_NODE = -1;
const uint SPHERE_KIND = 1;
const uint CYLINDER_KIND = 2;
const uint CUBE_KIND = 3;

// Buffer items need their size to be a multiple of 16 bytes. This struct is 32 bytes.
struct BoundingBallNode {
    vec3 pos;
    float radius;
    // Doubles as solid index if right == LEAF_NODE
    uint left;
    // We are a leaf if right == LEAF_NODE
    uint right;

    int _padding1;
    int _padding2;
};

// Solids are sent to the GPU as mat4. Since every mat4 expressing a 3d transformation in homogenous
// coordinates leaves the last row as (0, 0, 0, 1), we have 4 spare floats of space for other data.
// There we fit color as a vec3 and a float-enum indicating sphere, cylinder or cube.
// [--- --- --- ---
//  --- matrix  ---
//  --- --- --- ---
//  -- color - kind]

uint solid_get_kind(mat4 solid) {
    const float kind = solid[3][3];
    if (kind == 1.0) {
        return SPHERE_KIND;
    } else if (kind == 2.0) {
        return CYLINDER_KIND;
    } else if (kind == 4.0) {
        return CUBE_KIND;
    } else {
        return 1234567;
    }
}

vec3 solid_get_color(mat4 solid) {
    return vec3(solid[0][3], solid[1][3], solid[2][3]);
}

mat4 solid_get_world_to_local(mat4 solid) {
    solid[0][3] = 0;
    solid[1][3] = 0;
    solid[2][3] = 0;
    solid[3][3] = 1;
    return solid;
}
