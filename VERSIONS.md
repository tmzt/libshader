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

## 0.1.2 — 2026-11-04 (tag v0.1.2)

Strictly additive (Rule 5.4), so the patch version.

- Added `daylight`, an outdoor light rig a renderer imports:
  - `DAYLIGHT_WGSL`, the rig as WGSL, joined ahead of a renderer's own
    source. The sun's arc by the hour and the date, its dusk and night
    keys, the sky's gradient with clouds and a disc, the ambient from the
    sky and the ground's bounce, a lamp's falloff and whether it is seen,
    aerial haze, a soft sun shadow marched through the host's height
    field, the ACES tone map, and sRGB encoding both ways. It declares no
    binding, no uniform and no entry point: a host sets `dl_t`, `dl_early`
    and `dl_decl` and passes the rest as arguments.
  - `DAYLIGHT_FLAT_WGSL`, the `dl_height` a host with nothing to cast a
    shadow joins instead of writing one.
  - `Keys` and `Keys::at`, a value at the day's three keys and the blend
    between them, with `SUN_RGB`, `SKY_RGB`, `BOUNCE_RGB`, `ZENITH_RGB`,
    `HORIZON_RGB`, `HAZE_RGB` and `SUN_DIR`.
  - `arc_sun`, `sun_dir`, `ambient` and `tonemap`, the CPU's half of the
    same numbers.
  - `NOON`, `DUSK`, `NIGHT`, `ZENITH_ANGLE` and `SOLSTICE_DECLINATION`.
- Re-exported `DAYLIGHT_WGSL` and `DAYLIGHT_FLAT_WGSL` at the crate root.
- The crate's description names the rig.

Nothing existing changed, and nothing else in the crate depends on the new
module: a renderer that does not import it is unaffected, including in what
its shaders compile.

Rule 5.2 asks the heading to name the commit, which a commit cannot carry
for itself; the tag is that pointer, and it is on the commit that sets the
version, as Rule 5.1 requires.
