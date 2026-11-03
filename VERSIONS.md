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

## 0.1.1 — 2026-11-03 (tag v0.1.1)

Strictly additive (Rule 5.4), so the patch version.

- Added `view`:
  - `View`, the orthographic view a renderer draws through - yaw and
    pitch, the frame's world half-height, its centre and the target's
    aspect - with `Default` (the front view of a square frame two units
    across), `isometric`, `clamped_pitch`, `rotation`, `basis`, `project`
    and `to_pixel`;
  - `ISOMETRIC_PITCH`, `atan(1 / sqrt 2)`, and `PITCH_LIMIT`.
- Re-exported `View`, `ISOMETRIC_PITCH` and `PITCH_LIMIT` at the crate
  root.

Rule 5.2 asks the heading to name the commit, which a commit cannot carry
for itself; the tag is that pointer, and it is on the commit that sets the
version, as Rule 5.1 requires.
