# CHANGES

Planned work, plans and architectural changes, appended oldest first
(AGENTS.md, Rule 0).

## 2026-10-18 16:30 EDT — The repository stands alone

Extraction plan step 2 for libshader (the plan lives with the repository's
owner; this repository was extracted from another game's repository with
its hair-rendering parts left out). Adds AGENTS.md (libcontrols' critical
rules 0 to 5, Rule 5.4 from 0.1.0), README.md, CHANGES.md, VERSIONS.md,
LICENSE-MIT and LICENSE-APACHE, a standalone manifest (edition 2024,
"MIT OR Apache-2.0", 0.1.0, naga 30.0.1 with `wgsl-in`, bytemuck 1.23.2
with `derive`, glam 0.33) and a `build.rs` that validates the shipped
header with naga.

Departures from the extraction plan:

- The extraction kept no `build.rs` for this crate (the manifest named one
  that was not carried over), so a new one is written: it includes
  `src/build.rs` by path and validates `src/instance.wgsl`.
- There are no `docs/design` documents yet, so the README's decisions table
  is empty.
- Dependency versions are written as the plan's minimum caret requirements
  (`1.23.2`, `0.33`, `30.0.1`), not `=` pins; `Cargo.lock` is ignored by git,
  so a build may resolve a newer compatible patch (bytemuck resolves 1.25.2).

## 2026-10-18 16:36 EDT — A timestamp corrected

The entry "The repository stands alone" above was timed 16:30 by hand, not
from the clock; its text stands and this note records that.

## 2026-10-22 02:57 EDT — 0.1.0 recorded and tagged after the fact

The version was set to 0.1.0 in 5236d2c without a `VERSIONS.md` entry or
a tag (Rule 5.1 and 5.3). The entry is appended now, naming 5236d2c, and
`v0.1.0` tags 5236d2c, the commit that set the version.

## 2026-11-03 03:57 EST — An orthographic view beside the headers

Planned work for 0.1.1: a `view` module carrying `View`, the orthographic
camera a renderer draws through, with `ISOMETRIC_PITCH` and `PITCH_LIMIT`.
Strictly additive; nothing already here changes.

Why it belongs beside the headers rather than in a renderer. Two renderers
drawing one frame have to be told the same camera, and today a host that
draws a scene with more than one of them has no type to say it in: each
renderer declares its own six fields and converts between them, which is the
same number written several times with nothing holding the copies in step.
Measured across two renderers that already do this, the file is identical
byte for byte in both, so there is nothing to reconcile - which is the
argument for the module and not merely for a convenience.

It keeps the crate's own rule: six `f32`, `bytemuck`/`glam`/`naga` only, no
GPU API. The two angles are a basis on the CPU and the same basis in a
shader, which is exactly the kind of agreement this crate is for.

Departures and notes:

- `Default` is neutral - the front view of a square frame two units across -
  rather than framing any particular object. The renderers this was measured
  against both default to a framing of their own subject, which is a caller's
  business; a shared default that framed one of them would be wrong for every
  other. A renderer wanting its old framing states it, and `isometric()`
  takes the neutral framing.
- The six methods are lifted unchanged: compared with their doc comments and
  the type's name removed, `isometric`, `clamped_pitch`, `rotation`, `basis`,
  `project` and `to_pixel` and the struct's fields are character-identical to
  the measured source. Only the doc comments and `Default` are new.
- `PITCH_LIMIT` keeps its value and its reason, generalised: an orbit control
  holds the pitch short of the poles, where yaw stops meaning anything.
- This addendum is written with the change rather than before it (Rule 0.1).
