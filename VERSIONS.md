# VERSIONS

Each change of the crate's version and the public API changes it carries
(AGENTS.md, Rule 5). Internal changes are not listed, and there is no entry
without a version change. Entries are appended, oldest first, and never
edited.

## 0.1.0 — 2026-10-18 (5236d2c, tag v0.1.0)

The first version: the crate standing alone.

- Added `instance`:
  - `InstanceHeader`, the record every instanced renderer's instance
    begins with (position, uniform scale, rotation, time offset and
    scale, id, flags), with `IDENTITY`, `with_time`, `with_id` and
    `with_flags`;
  - `FrameHeader` and `FrameHeader::new`;
  - the flags `HEADER_HIDDEN`, `HEADER_SELECTED`, `HEADER_HIGHLIGHT` and
    the reserved high bits `HEADER_RESERVED`;
  - `HEADER_WGSL`, the same layouts in WGSL;
  - `uniform_scale` and its error `NonUniformScale`.
- Added `build`: `validate`, for a renderer's `build.rs` (joins a prelude
  and its WGSL files, validates them with naga, reports errors against
  the original file and line), and `check`.
- Added `layout`: `wgsl_offsets` and `wgsl_size`, the member offsets and
  size naga computes for a WGSL struct.
