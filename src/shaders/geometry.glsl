/*
*   Forward declarations
*/
float hit_time_sphere_impl(const vec3, const vec3, const vec3, const float);

/*
*   Implementations
*/
vec3 cylinder_normal(const vec3 pos, const Cylinder cylinder) {
    // TODO
    return vec3(1, 0, 0);
}

vec3 cuboid_normal(const vec3 pos, const Cuboid cuboid) {
    // TODO
    return vec3(1, 0, 0);
}

float hit_time_cylinder(const vec3 from, const vec3 ray, const Cylinder cylinder) {
    // TODO
    return -1.0;
}

float hit_time_cuboid(const vec3 from, const vec3 ray, const Cuboid cuboid) {
    // TODO
    return -1.0;
}

float hit_time_sphere(const vec3 from, const vec3 ray, const Sphere sphere) {
    return hit_time_sphere_impl(from, ray, sphere.pos, sphere.radius);
}

float hit_time_node(const vec3 from, const vec3 ray, const BoundingBallNode node) {
    return hit_time_sphere_impl(from, ray, node.pos, node.radius);
}

/*
*   Internal
*/

// When will the ray from [from] along [ray] hit the sphere at [pos] with radius [radius]?
// If never, we return -1
float hit_time_sphere_impl(const vec3 from, const vec3 ray, const vec3 pos, const float radius) {
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
