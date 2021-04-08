#ifndef GEOMETRY_GLSL
#define GEOMETRY_GLSL

#include "common.glsl"

// HIT TIMES:
// When will a particle at [from] with (constant) velocity [ray] hit the unit shape?
// If never, return -1

// x^2 + y^2 + z^2 = 1
float hit_time_unit_sphere(const vec3 from, const vec3 ray) {
    const float A = dot(ray, ray);
    const float B = dot(ray, from);
    const float C = dot(from, from) - 1;
    // Solve At^2 + 2Bt + C = 0. Return smallest nonnegative root, else -1.
    return interact_time(solve_quadratic(A, B, C));
}

// |z| = 1 and x^2 + y^2 = 1
float hit_time_unit_cylinder(const vec3 from, const vec3 ray) {
    float plane_t1 = (-1 - from.z) / ray.z;
    float plane_t2 = ( 1 - from.z) / ray.z;

    vec2 plane = vec2(min(plane_t1, plane_t2), max(plane_t1, plane_t2));

    const float A = dot(ray.xy, ray.xy);
    const float B = dot(ray.xy, from.xy);
    const float C = dot(from.xy, from.xy) - 1;

    vec2 tube = solve_quadratic(A, B, C);

    float enter = max(plane[0], tube[0]);
    float exit = min(plane[1], tube[1]);

    return interact_time(vec2(enter, exit));
}

// max(|x|, |y|, |z|) = 1
float hit_time_unit_cube(const vec3 from, const vec3 ray) {
    return hit_time_aabb(from, ray, vec3(-1), vec3(1));
}

// NORMALS

// x^2 + y^2 + z^2 = 1
vec3 normal_unit_sphere(const vec3 pos) {
    return normalize(pos); // FIXME Is already be normalized?
}

// |z| = 1 and x^2 + y^2 = 1
vec3 normal_unit_cylinder(const vec3 pos) {
    const float EPSILON = 0.01;

    if (are_equal(pos.z, 1, EPSILON)) {
        return vec3(0, 0, 1);
    } else if (are_equal(pos.z, -1, EPSILON)) {
        return vec3(0, 0, -1);
    } else {
        return vec3(normalize(pos.xy), 0); // FIXME Is already be normalized?
    }
}

// max(|x|, |y|, |z|) = 1
vec3 normal_unit_cube(const vec3 pos) {
    const float EPSILON = 0.001;
    return normalize(trunc(pos * (1 + EPSILON)));
}

#endif
