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
    // Solve At^2 + 2Bt + C = 0. Return smallest positive root, else -1.
    return solve_quadratic(A, B, C);
}

// |z| = 1 and x^2 + y^2 = 1
float hit_time_unit_cylinder(const vec3 from, const vec3 ray) {
    float time = 1e9;
    {
        const float A = dot(ray.xy, ray.xy);
        const float B = dot(ray.xy, from.xy);
        const float C = dot(from.xy, from.xy) - 1;
        // Solve At^2 + 2Bt + C = 0. Return smallest positive root, else -1.
        float t = solve_quadratic(A, B, C);
        if (t > 0 && on_unit_cylinder(from + ray * t)) {
            time = min(time, t);
        }
    }
    {
        float t = (-1 - from.z) / ray.z;
        if (t > 0 && on_unit_cylinder(from + ray * t)) {
            time = min(time, t);
        }
    }
    {
        float t = ( 1 - from.z) / ray.z;
        if (t > 0 && on_unit_cylinder(from + ray * t)) {
            time = min(time, t);
        }
    }
    return (time == 1e9) ? -1 : time;
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
    const float EPSILON = 0.001;

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
