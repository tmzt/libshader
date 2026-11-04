//! Validates the header this crate ships with naga, by including the
//! build-time validator from `src/build.rs` by path (a build script cannot
//! depend on the crate it builds).

#[path = "src/build.rs"]
mod validate;

fn main() {
    println!("cargo:rerun-if-changed=src/instance.wgsl");
    validate::validate(&["src/instance.wgsl"], &[]);
    // The light rig with the flat height field after it, which is how a
    // host with nothing to cast a shadow joins it. `daylight.wgsl` alone
    // does not validate and is not meant to: it calls `dl_height`, which
    // its host defines.
    println!("cargo:rerun-if-changed=src/daylight.wgsl");
    println!("cargo:rerun-if-changed=src/daylight_flat.wgsl");
    validate::validate(&["src/daylight.wgsl", "src/daylight_flat.wgsl"], &[]);
}
