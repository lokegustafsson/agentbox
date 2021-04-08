#ifndef COMMON_GLSL
#define COMMON_GLSL

const uint LEAF_NODE = -1;
const uint SPHERE_KIND = 1;
const uint CYLINDER_KIND = 2;
const uint CUBE_KIND = 3;

// Buffer items need their size to be a multiple of 16 bytes. This struct is 32 bytes.
struct AABBNode {
    vec3 mini;
    // Doubles as solid index if right == LEAF_NODE
    uint left;

    vec3 maxi;
    // We are a leaf if right == LEAF_NODE
    uint right;
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

float interact_time(vec2 enterexit) {
    float enter = enterexit[0];
    float exit = enterexit[1];

    if (exit < enter) {
        return -1;
    } else if (enter > 0) {
        return enter;
    } else {
        return exit;
    }
}

float hit_time_aabb(const vec3 from, const vec3 ray, vec3 mini, vec3 maxi) {
    vec3 times_mini = (mini - from) / ray;
    vec3 times_maxi = (maxi - from) / ray;

    vec3 times_enter = min(times_mini, times_maxi);
    vec3 times_exit = max(times_mini, times_maxi);

    float enter = max(times_enter.x, max(times_enter.y, times_enter.z));
    float exit = min(times_exit.x, min(times_exit.y, times_exit.z));

    return interact_time(vec2(enter, exit));
}

bool are_equal(const float a, const float b, const float epsilon) {
    return abs(a - b) < epsilon;
}

// Solve At^2 + 2Bt + C = 0. Return (smallest, largest) solutions
vec2 solve_quadratic(float A, float B, float C) {
    if (A < 0) {
        A = -A;
        B = -B;
        C = -C;
    }
    const float det = B*B - A*C;
    if (det < 0) {
        return vec2(-1, -1);
    }

    const float sqrtd = sqrt(det);
    const float enter = (-B - sqrtd)/A;
    const float exit = (-B + sqrtd)/A;

    return vec2(enter, exit);
}

#endif
