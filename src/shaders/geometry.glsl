/*
*   Forward declarations
*/
float solve_quadratic(float, float, float);
bool on_unit_cube(const vec3);
bool on_unit_cylinder(const vec3);
bool are_equal(const float, const float);

/*
*   Implementations
*/

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
        if (t > 0) {
            time = min(time, t);
        }
    }
    {
        float t = (-1 - from.z) / ray.z;
        if (t > 0 && on_unit_cube(from + ray * t)) {
            time = min(time, t);
        }
    }
    {
        float t = ( 1 - from.z) / ray.z;
        if (t > 0 && on_unit_cube(from + ray * t)) {
            time = min(time, t);
        }
    }
    return (time == 1e9) ? -1 : time;
}

// max(|x|, |y|, |z|) = 1
float hit_time_unit_cube(const vec3 from, const vec3 ray) {
    float time = 1e9;

    { float t = (-1 - from.x) / ray.x;
        if (t > 0 && on_unit_cube(from + ray * t)) { time = min(time, t); } }
    { float t = (-1 - from.y) / ray.y;
        if (t > 0 && on_unit_cube(from + ray * t)) { time = min(time, t); } }
    { float t = (-1 - from.z) / ray.z;
        if (t > 0 && on_unit_cube(from + ray * t)) { time = min(time, t); } }
    { float t = ( 1 - from.x) / ray.x;
        if (t > 0 && on_unit_cube(from + ray * t)) { time = min(time, t); } }
    { float t = ( 1 - from.y) / ray.y;
        if (t > 0 && on_unit_cube(from + ray * t)) { time = min(time, t); } }
    { float t = ( 1 - from.z) / ray.z;
        if (t > 0 && on_unit_cube(from + ray * t)) { time = min(time, t); } }

    return (time == 1e9) ? -1 : time;
}

// NORMALS

// x^2 + y^2 + z^2 = 1
vec3 normal_unit_sphere(const vec3 pos) {
    return normalize(pos);
}

// |z| = 1 and x^2 + y^2 = 1
vec3 normal_unit_cylinder(const vec3 pos) {
    if (are_equal(pos.z, 1)) {
        return vec3(0, 0, 1);
    } else if (are_equal(pos.z, -1)) {
        return vec3(0, 0, -1);
    } else {
        return normalize(vec3(pos.x, pos.y, 0));
    }
}

// max(|x|, |y|, |z|) = 1
vec3 normal_unit_cube(const vec3 pos) {
    const float EPSILON = 0.001;
    return normalize(trunc(pos * (1 + EPSILON)));
}

/*
*   Internal
*/

// Solve At^2 + 2Bt + C = 0. Return smallest nonnegative or else -1
float solve_quadratic(float A, float B, float C) {
    const float det = B*B - A*C;
    if (det < 0) {
        return -1;
    }

    const float sqrtd = sqrt(det);
    const float t1 = (-B + sqrtd)/A;
    const float t2 = (-B - sqrtd)/A;
    if (t1 > 0 && t2 > 0) {
        return min(t1, t2);
    } else if (t1 > 0) {
        return t1;
    } else {
        return t2;
    }
}

// max(|x|, |y|, |z|) = 1
bool on_unit_cube(const vec3 pos) {
    vec3 abspos = abs(pos);
    float maxi = max(abspos.x, max(abspos.y, abspos.z));
    return are_equal(maxi, 1);
}

// |z| = 1 and x^2 + y^2 = 1
bool on_unit_cylinder(const vec3 pos) {
    const float EPSILON = 0.001;

    bool between_planes = abs(pos.z) < 1 + EPSILON;
    bool on_planes = are_equal(abs(pos.z), 1);

    float tube_radius2 = dot(pos.xy, pos.xy);
    bool on_tube = are_equal(tube_radius2, 1);
    bool in_tube = tube_radius2 < 1 + EPSILON;

    return (between_planes && on_tube) || (on_planes && in_tube);
}

bool are_equal(const float a, const float b) {
    const float EPSILON = 0.001;
    return abs(a - b) < EPSILON;
}
