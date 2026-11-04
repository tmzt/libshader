// The height field for a host that has none: everything stands on the
// ground plane, so nothing casts a shadow and `dl_shadow` is a constant 1.
//
// Joined after `daylight.wgsl` by a host that does not define `dl_height`
// itself. WGSL has no default definitions, so this is one line in a file
// rather than a flag.
fn dl_height(c: vec2<i32>) -> f32 {
    return 0.0;
}
