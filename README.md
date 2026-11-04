# libshader

Shared GPU data layouts and build-time WGSL tooling for instanced renderers.

- `instance`: the `InstanceHeader` and `FrameHeader` every instanced
  renderer's records begin with, as `#[repr(C)]` bytemuck structs and as
  WGSL (`HEADER_WGSL`), with their flags.
- `build`: `validate`, for a renderer's `build.rs`: joins a prelude and the
  renderer's own WGSL files and validates them with naga, reporting errors
  against the original file and line.
- `layout`: `wgsl_offsets` and `wgsl_size`, the member offsets naga computes
  for a WGSL struct, for tests that compare them with `offset_of!`.
- `view`: `View`, the orthographic view a renderer draws through - yaw and
  pitch, the frame's world half-height, its centre and the target's aspect -
  whose basis is built the same way on the CPU and in a shader.
- `daylight`: an outdoor light rig a renderer imports - the sun through a day
  into night, the sky it hangs in, the lamps a host adds, and the ACES tone
  map and encoding that put the result on a target. `DAYLIGHT_WGSL` is joined
  ahead of a renderer's own source and declares no binding or entry point, so
  one copy serves a renderer's own passes and the lighting hooks it gives
  another renderer; the Rust half states the same keys for a host that has to
  know where the light comes from without asking the GPU.

It depends on no GPU API. Licensed under MIT OR Apache-2.0
([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)).

## Architectural decisions

| Document | Date | Status | Decision |
|---|---|---|---|
