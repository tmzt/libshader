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
//! - [`view`]: the orthographic view a renderer draws through, as the two
//!   angles and the framing both sides have to agree on.
//! - [`daylight`]: an outdoor light rig a renderer imports - the sun through
//!   a day into night, the sky, the lamps, the tone map - as WGSL joined
//!   ahead of a renderer's own source, and as the CPU half of the same
//!   numbers.
//!
//! The first four are a contract both sides must agree on rather than a
//! library of shader code; a renderer's own SDF primitives stay in it.
//! [`daylight`] is the exception and is deliberately one: two renderers
//! drawing one frame have to agree on where the light comes from exactly as
//! they have to agree on where the camera is, and a rig written twice is two
//! rigs. It is imported, never required - nothing else here depends on it.
//! Nothing in this crate depends on a GPU API.

pub mod build;
pub mod daylight;
pub mod instance;
pub mod layout;
pub mod view;

pub use daylight::{DAYLIGHT_FLAT_WGSL, DAYLIGHT_WGSL};
pub use instance::{
    FrameHeader, HEADER_HIDDEN, HEADER_HIGHLIGHT, HEADER_RESERVED, HEADER_SELECTED, HEADER_WGSL,
    InstanceHeader, NonUniformScale, uniform_scale,
};
pub use view::{ISOMETRIC_PITCH, PITCH_LIMIT, View};
