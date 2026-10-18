//! Validates the header this crate ships with naga, by including the
//! build-time validator from `src/build.rs` by path (a build script cannot
//! depend on the crate it builds).

#[path = "src/build.rs"]
mod validate;

fn main() {
    println!("cargo:rerun-if-changed=src/instance.wgsl");
    validate::validate(&["src/instance.wgsl"], &[]);
}
