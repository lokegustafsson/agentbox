#ifndef BACKGROUND_GLSL
#define BACKGROUND_GLSL

// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.
uint hash(uint x) {
    x += (x << 10u);
    x ^= (x >>  6u);
    x += (x <<  3u);
    x ^= (x >> 11u);
    x += (x << 15u);
    return x;
}
uint hash(uvec3 v) { return hash( v.x ^ hash(v.y) ^ hash(v.z)); }

// Construct a float with half-open range [0:1] using low 23 bits.
// All zeroes yields 0.0, all ones yields the next smallest representable value below 1.0.
float random(float x, float y, float z) {
    const uint ieeeMantissa = 0x007FFFFFu;
    const uint ieeeOne      = 0x3F800000u;
    uint res = hash(floatBitsToUint(vec3(x,y,z)));
    res &= ieeeMantissa;                     // Keep only mantissa bits (fractional part)
    res |= ieeeOne;                          // Add fractional part to 1.0
    return uintBitsToFloat(res) - 1.0;                        // Range [0:1]
}

float random_tile(vec3 pos, float zoom, float strength) {
    pos *= zoom;
    vec3 low = floor(pos);
    vec3 offset = pos - low;
    float c00 = mix(random(low.x, low.y, low.z), random(low.x, low.y, low.z+1), offset.z);
    float c01 = mix(random(low.x, low.y+1, low.z), random(low.x, low.y+1, low.z+1), offset.z);
    float c10 = mix(random(low.x+1, low.y, low.z), random(low.x+1, low.y, low.z+1), offset.z);
    float c11 = mix(random(low.x+1, low.y+1, low.z), random(low.x+1, low.y+1, low.z+1), offset.z);
    return mix(mix(c00, c01, offset.y), mix(c10, c11, offset.y), offset.x) * strength;
}

float square(float x) {
    return x * x;
}

float smooth_noise(const vec3 dir) {
    return square(random_tile(dir, 0.5, 0.5))
    + square(random_tile(dir, 2, 0.2))
    + square(random_tile(dir, 8, 0.2))
    + random_tile(dir, 30, 0.1)
    + random_tile(dir, 100, 0.05)
    + random_tile(dir, 320, 0.05);
}

#endif
