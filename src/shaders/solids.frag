#version 450

// Spheres, cylinders, rectangular cuboids

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

// Internal structs
struct hit_report {
    vec3 pos;
    vec3 normal;
    uint kind;
    uint index;
};
struct rays {
    vec3 reflected_pos;
    vec3 reflected_ray;
    vec3 refracted_pos;
    vec3 refracted_ray;
};


// Constants ===
const int STACK_SIZE = 100;
const uint ERROR_NONE = 0;
const uint ERROR_OTHER = 1;
const uint ERROR_INVALID_KIND = 2;
const uint ERROR_STACK_OVERFLOW = 3;
const uint ERROR_INVALID_BUFFER_SIZES = 4;
const uint ERROR_INVALID_PUSH_CONSTANTS = 5;

const vec4 TRANSPARENT = vec4(0);

const uint NO_HIT_KIND = 0;
const uint SPHERE_KIND = 1;
const uint CYLINDER_KIND = 2;
const uint CUBOID_KIND = 3;

const float EPSILON = 0.01;

//const vec3 AMBIENT = vec3(0.08);
const vec3 AMBIENT = vec3(1);
const vec3 SUN_COLOR = vec3(1);
const float SUN_SIZE = 1e-2;
const vec3 SUN_DIRECTION = vec3(0, 1, 0);
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
layout(set=0, binding=0) readonly buffer BoundingBallTree {
    BoundingBallNode solids_tree[];
};
layout(set=0, binding=1) readonly buffer Spheres {
    Sphere spheres[];
};
layout(set=0, binding=2) readonly buffer Cylinders {
    Cylinder cylinders[];
};
layout(set=0, binding=3) readonly buffer Cuboids {
    Cuboid cuboids[];
};

// Forward function declarations ===
vec4 simple_ray(const vec3, const vec3);
vec3 background_light(const vec3);
hit_report cast_ray(const vec3, const vec3);
hit_report no_hit_report();
float hit_time_node(const vec3, const vec3, const uint);
float hit_time_sphere(const vec3, const vec3, const vec3, const float);

void main() {
    const vec2 frag_pos = gl_FragCoord.xy / window_size.y;
    const vec2 mid_frag_pos = vec2(0.5 * window_size.x / window_size.y, 0.5);
    const vec3 camera_space_ray = normalize(vec3(frag_pos - mid_frag_pos, 1));

    const vec4 camera_pos = camera_to_world * vec4(vec3(0), 1);
    const vec4 camera_ray = camera_to_world * vec4(camera_space_ray, 0); // 0 => not translated

    const uint num_bodies = spheres.length() + cylinders.length() + cuboids.length();
    const bool invalid_buffers = solids_tree.length() != 2 * num_bodies - 1;
    const bool invalid_push_constants = any(isinf(window_size))
        || any(isnan(window_size))
        || any(lessThanEqual(window_size, vec2(0)));

    if (invalid_buffers) {
        fatal_error = ERROR_INVALID_BUFFER_SIZES;
    } else if (invalid_push_constants) {
        fatal_error = ERROR_INVALID_PUSH_CONSTANTS;
    } else {
        f_color = simple_ray(camera_pos.xyz / camera_pos.w, camera_ray.xyz);
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

// Casts a ray using Blinn-Phong illumination
vec4 simple_ray(const vec3 from, const vec3 ray) {
    hit_report hit = cast_ray(from, ray);
    if (hit.kind == NO_HIT_KIND) {
        return TRANSPARENT;
    }
    const vec3 normal = hit.normal;
    const vec3 hit_point = hit.pos;
    vec3 color;
    switch (hit.kind) {
        case SPHERE_KIND:
            color = spheres[hit.index].color;
            break;
        case CYLINDER_KIND:
        case CUBOID_KIND:
        default:
            fatal_error = ERROR_INVALID_KIND;
            return vec4(0);
    }

    // Ambient
    vec3 light = AMBIENT * color;
    if (cast_ray(hit_point + EPSILON * SUN_DIRECTION, SUN_DIRECTION).kind == NO_HIT_KIND) {
        const float alignment = dot(normal, normalize(SUN_DIRECTION - ray));
        // Diffuse
        light += color * SUN_COLOR * alignment;
        // Specular
        light += SUN_COLOR * pow(alignment, inversesqrt(SUN_CORONA));
    }
    return vec4(light, 1);
}

// Cast a ray by traversing [solids_tree]. Will set [fatal_error] on overflow
hit_report cast_ray(const vec3 from, const vec3 ray) {
    uint stack[STACK_SIZE];
    int stack_ptr = -1;

    const int root = 0; // See bounding_ball_tree.rs
    if (hit_time_node(from, ray, root) > 0) {
        stack[++stack_ptr] = root;

        // TRACE (maybe hit something)
        //fatal_error = ERROR_OTHER;
        //return no_hit_report();
        // TRACE
    }
    float first_hit_time = 1e9;
    uint first_hit_kind = NO_HIT_KIND;
    uint first_hit_index = 0;
    while (stack_ptr >= 0) {
        const uint hit = stack[stack_ptr--];
        if (solids_tree[hit].left < 0) {
            // Hit leaf
            const int kind = -solids_tree[hit].left;
            const int index = solids_tree[hit].right;
            float time;
            switch (kind) {
                case SPHERE_KIND:
                    time = hit_time_sphere(from, ray, spheres[index].pos, spheres[index].radius);
                    break;
                case CYLINDER_KIND: // TODO
                case CUBOID_KIND: // TODO
                default:
                    fatal_error = ERROR_INVALID_KIND;
                    return no_hit_report();
            }
            if (time > 0 && time < first_hit_time) {
                first_hit_time = time;
                first_hit_kind = kind;
                first_hit_index = index;
            }
        } else {
            // Continue traversal down
            int left = solids_tree[hit].left;
            int right = solids_tree[hit].right;
            float l_hit = hit_time_node(from, ray, left);
            float r_hit = hit_time_node(from, ray, right);
            if (r_hit < l_hit) {
                float tmpf = l_hit;
                l_hit = r_hit;
                r_hit = tmpf;

                int tmpi = left;
                left = right;
                right = tmpi;
            }
            if (r_hit > 0) {
                if (stack_ptr + 1 == STACK_SIZE) {
                    fatal_error = ERROR_STACK_OVERFLOW;
                    return no_hit_report();
                }
                stack[++stack_ptr] = right;
            }
            if (l_hit > 0) {
                if (stack_ptr + 1 == STACK_SIZE) {
                    fatal_error = ERROR_STACK_OVERFLOW;
                    return no_hit_report();
                }
                stack[++stack_ptr] = left;
            }
        }
    }
    const vec3 hit_pos = from + ray * first_hit_time;
    vec3 normal;
    switch (first_hit_kind) {
        case NO_HIT_KIND:
            return no_hit_report();
        case SPHERE_KIND:
            normal = normalize(hit_pos - spheres[first_hit_index].pos);
            break;
        case CYLINDER_KIND: // TODO
        case CUBOID_KIND: // TODO
        default:
            fatal_error = ERROR_INVALID_KIND;
            return no_hit_report();
    }
    return hit_report(
        hit_pos,
        normal,
        first_hit_kind,
        first_hit_index
    );
}

hit_report no_hit_report() {
    return hit_report(vec3(0), vec3(0), NO_HIT_KIND, 0);
}

float hit_time_node(const vec3 from, const vec3 ray, const uint node) {
    return hit_time_sphere(from, ray, solids_tree[node].pos, solids_tree[node].radius);
}

// When will the ray from [from] along [ray] hit the sphere at [pos] with radius [radius]?
// If never, we return -1
float hit_time_sphere(const vec3 from, const vec3 ray, const vec3 pos, const float radius) {
    const vec3 rel_pos = pos - from;

    const float A = dot(ray, ray);
    const float B = dot(ray, rel_pos);
    const float C = dot(rel_pos, rel_pos) - radius * radius;

    const float det = B*B - A*C;
    if (det < 0) {
        return -1;
    }

    const float sqrtd = sqrt(det);
    const float t1 = (B + sqrtd)/A;
    const float t2 = (B - sqrtd)/A;
    if (t1 > 0 && t2 > 0) {
        return min(t1, t2);
    } else if (t1 > 0) {
        return t1;
    } else {
        return t2;
    }
}
