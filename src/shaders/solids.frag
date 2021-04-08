#version 450
#include "common.glsl"
#include "geometry.glsl"
#include "background.glsl"

// Internal structs
struct hit_report {
    vec3 pos;
    vec3 normal;
    uint index;
};

// Constants ===
const uint STACK_SIZE = 100;
const uint ERROR_NONE = 0;
const uint ERROR_OTHER = 1;
const uint ERROR_INVALID_KIND = 2;
const uint ERROR_STACK_OVERFLOW = 3;
const uint ERROR_INVALID_BUFFER_SIZES = 4;
const uint ERROR_INVALID_PUSH_CONSTANTS = 5;

const vec4 TRANSPARENT = vec4(0);

const float INITIAL_HIT_TIME_BOUND = 1e9;
const vec3 NO_HIT_NORMAL = vec3(0);
const float EPSILON = 0.01;

const vec3 AMBIENT = vec3(0.08);
const vec3 SUN_COLOR = vec3(1);
const float SUN_SIZE = 1e-2;
const vec3 SUN_DIRECTION = vec3(0, 0, 1);
const float SUN_CORONA = 1e-3;

// Global variables ===
uint fatal_error = ERROR_NONE;

// IO ===
in vec4 gl_FragCoord;
layout(location=0) out vec4 f_color;

// Buffers & Push constants ===
layout(push_constant) uniform PushConstants {
    mat4 camera_to_world;
    vec2 window_size;
};
layout(set=0, binding=0) readonly buffer AABBTree {
    AABBNode aabb_tree[];
};
layout(set=0, binding=1) readonly buffer Solids {
    mat4 solids[];
};

// Forward function declarations ===
vec3 simple_ray(const vec3, const vec3);
float background(const vec3);
hit_report cast_ray(const vec3, const vec3);
float hit_time_node(const vec3, const vec3, const uint);
float hit_time_solid(const vec3, const vec3, const mat4);
vec3 solid_normal(const vec3, const mat4);
hit_report no_hit_report();

void main() {
    const vec2 frag_pos = gl_FragCoord.xy / window_size.y;
    const vec2 mid_frag_pos = vec2(0.5 * window_size.x / window_size.y, 0.5);
    const vec3 camera_space_ray = normalize(vec3(frag_pos - mid_frag_pos, 1));

    const vec4 camera_pos = camera_to_world * vec4(vec3(0), 1);
    // w = 0 => not translated by transformation
    const vec4 camera_ray = camera_to_world * vec4(camera_space_ray, 0);

    const bool invalid_push_constants = any(isinf(window_size))
        || any(isnan(window_size))
        || any(lessThanEqual(window_size, vec2(0)));

    if (aabb_tree.length() != 2 * solids.length() - 1) {
        fatal_error = ERROR_INVALID_BUFFER_SIZES;
    } else if (invalid_push_constants) {
        fatal_error = ERROR_INVALID_PUSH_CONSTANTS;
    } else {
        vec3 color = simple_ray(camera_pos.xyz / camera_pos.w, camera_ray.xyz);
        f_color = vec4(color, 1);
    }

    switch (fatal_error) {
        case ERROR_NONE:
            break;
        case ERROR_INVALID_KIND:
            f_color = vec4(1, 1, 0, 1); // YELLOW
            break;
        case ERROR_STACK_OVERFLOW:
            f_color = vec4(0, 1, 1, 1); // CYAN
            break;
        case ERROR_INVALID_BUFFER_SIZES:
            f_color = vec4(1, 0, 1, 1); // MAGENTA
            break;
        case ERROR_INVALID_PUSH_CONSTANTS:
            f_color = vec4(0, 1, 0, 1); // GREEN
            break;
        case ERROR_OTHER:
        default:
            f_color = vec4(1, 0, 0, 1); // RED
            break;
    }
}

// Cast a ray using Blinn-Phong illumination
vec3 simple_ray(const vec3 from, const vec3 ray) {
    hit_report hit = cast_ray(from, ray);
    // No hit normal => no hit
    if (hit.normal == NO_HIT_NORMAL) {
        return vec3(smooth_noise(ray));
    }
    const vec3 hit_point = hit.pos;
    const vec3 normal = hit.normal;
    vec3 color = solid_get_color(solids[hit.index]);

    // Ambient
    vec3 light = AMBIENT * color;
    if (cast_ray(hit_point + EPSILON * (SUN_DIRECTION + normal), SUN_DIRECTION).normal == NO_HIT_NORMAL) {
        const float alignment = max(dot(normal, normalize(SUN_DIRECTION - ray)), 0);
        // Diffuse
        light += color * SUN_COLOR * alignment;
        // Specular
        light += SUN_COLOR * pow(alignment, inversesqrt(SUN_CORONA));
    }
    return light;
}

// Cast a ray by traversing [aabb_tree].
hit_report cast_ray(const vec3 from, const vec3 ray) {
    uint stack[STACK_SIZE];
    int stack_ptr = -1;

    const int root = 0; // See aabb_tree.rs
    if (hit_time_node(from, ray, root) > 0) {
        stack[++stack_ptr] = root;
    }
    float first_hit_time = INITIAL_HIT_TIME_BOUND;
    uint first_hit_index = 0;
    while (stack_ptr >= 0) {
        const uint hit = stack[stack_ptr--];
        if (aabb_tree[hit].right == LEAF_NODE) {
            // Hit leaf
            const uint index = aabb_tree[hit].left;
            float time = hit_time_solid(from, ray, solids[index]);
            if (time >= 0 && time < first_hit_time) {
                first_hit_time = time;
                first_hit_index = index;
            }
        } else {
            // Continue traversal down
            uint left = aabb_tree[hit].left;
            uint right = aabb_tree[hit].right;
            float l_hit = hit_time_node(from, ray, left);
            float r_hit = hit_time_node(from, ray, right);
            if (r_hit < l_hit) {
                float tmpf = l_hit;
                l_hit = r_hit;
                r_hit = tmpf;

                uint tmpi = left;
                left = right;
                right = tmpi;
            }
            if (r_hit >= 0) {
                if (stack_ptr + 1 == STACK_SIZE) {
                    fatal_error = ERROR_STACK_OVERFLOW;
                    return no_hit_report();
                }
                stack[++stack_ptr] = right;
            }
            if (l_hit >= 0) {
                if (stack_ptr + 1 == STACK_SIZE) {
                    fatal_error = ERROR_STACK_OVERFLOW;
                    return no_hit_report();
                }
                stack[++stack_ptr] = left;
            }
        }
    }
    if (first_hit_time == INITIAL_HIT_TIME_BOUND) {
        return no_hit_report();
    } else {
        const vec3 hit_pos = from + ray * first_hit_time;
        return hit_report(
            hit_pos,
            solid_normal(hit_pos, solids[first_hit_index]),
            first_hit_index
        );
    }
}

float hit_time_node(const vec3 from, const vec3 ray, const uint index) {
    return hit_time_aabb(from, ray, aabb_tree[index].mini, aabb_tree[index].maxi);
}

float hit_time_solid(const vec3 from, const vec3 ray, const mat4 solid) {
    const mat4 to_local = solid_get_world_to_local(solid);

    const vec3 local_from = (to_local * vec4(from, 1)).xyz;
    // w = 0 => not translated by transformation
    const vec3 local_ray = (to_local * vec4(ray, 0)).xyz;

    switch (solid_get_kind(solid)) {
        case SPHERE_KIND:
            return hit_time_unit_sphere(local_from, local_ray);
        case CYLINDER_KIND:
            return hit_time_unit_cylinder(local_from, local_ray);
        case CUBE_KIND:
            return hit_time_unit_cube(local_from, local_ray);
        default:
            fatal_error = ERROR_INVALID_KIND;
            return -1;
    }
}

vec3 solid_normal(const vec3 hit_pos, const mat4 solid) {
    const mat4 to_local = solid_get_world_to_local(solid);
    const vec3 pos = (to_local * vec4(hit_pos, 1)).xyz;

    vec3 normal;
    switch (solid_get_kind(solid)) {
        case SPHERE_KIND:
            normal = normal_unit_sphere(pos);
            break;
        case CYLINDER_KIND:
            normal = normal_unit_cylinder(pos);
            break;
        case CUBE_KIND:
            normal = normal_unit_cube(pos);
            break;
        default:
            fatal_error = ERROR_INVALID_KIND;
            return NO_HIT_NORMAL;
    }
    // FIXME Is inverting here a bottleneck?
    return normalize((inverse(to_local) * vec4(normal, 0)).xyz);
}

hit_report no_hit_report() {
    return hit_report(vec3(0), NO_HIT_NORMAL, 0);
}
