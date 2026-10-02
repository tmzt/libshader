//! Shared GPU data layouts, and the build-time tooling that keeps a renderer's
//! shaders agreeing with them.
//!
//! - [`instance`]: the instance and frame headers every instanced renderer's
//!   records begin with, as `#[repr(C)]` bytemuck structs and as WGSL, with
//!   the WGSL functions that read those layouts and nothing else.
//! - [`build`]: `validate`, for a renderer's `build.rs`: joins a prelude (such
//!   as [`HEADER_WGSL`]) and the renderer's own files and validates them with
//!   naga, reporting errors against the original file and line.
//! - [`layout`]: `wgsl_offsets`, the member offsets naga computes for a WGSL
//!   struct, for tests that compare them with `offset_of!` on the Rust side.
//!
//! This is a contract both sides must agree on, not a library of shader code:
//! SDF primitives, lighting and noise stay in each renderer. It depends on no
//! GPU API.

pub mod build;
pub mod instance;
pub mod layout;

pub use instance::{
    FrameHeader, HEADER_HIDDEN, HEADER_HIGHLIGHT, HEADER_RESERVED, HEADER_SELECTED, HEADER_WGSL,
    InstanceHeader, NonUniformScale, uniform_scale,
};
