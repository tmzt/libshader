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

It depends on no GPU API. Licensed under MIT OR Apache-2.0
([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)).

## Architectural decisions

| Document | Date | Status | Decision |
|---|---|---|---|
