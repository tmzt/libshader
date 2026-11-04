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

## 2026-11-04 10:52 EST — A daylight rig, imported rather than written twice

`daylight`: the sun through a day into night, the sky it hangs in, the
lamps a host adds, and the tone map and encoding that put the result on a
target. Released as 0.1.2.

**Why it is here at all, against this crate's own line.** The crate has
said since 0.1.0 that it is "a contract both sides must agree on, not a
library of shader code: SDF primitives, lighting and noise stay in each
renderer". That line is now narrowed rather than kept, and the doc says
so: two renderers drawing one frame have to agree on where the light comes
from exactly as they have to agree on where the camera is, which is the
same argument that put `view` here in 0.1.1. A rig written twice is two
rigs, and the second one drifts. SDF primitives stay in each renderer.

It is imported, never required. Nothing else in the crate names it, so a
renderer that does not want it compiles exactly what it compiled before.

**It declares nothing.** No binding, no uniform, no entry point, no vertex
format. Everything arrives in three private globals a host sets at the
start of its fragment stage - the sky's time, the hour, the declination -
or as an argument. That is what lets one copy serve a renderer's own
passes and the lighting hooks it hands to somebody else's renderer, and
come out the same in both; a rig that declared its own uniform could not
be joined into a module whose bindings are already spoken for.

**What a host must supply.** One function, `dl_height(c) -> f32`: the
height of whatever casts shadows over cell `c`, which the sun's shadow
march reads. A host with nothing to cast one joins `DAYLIGHT_FLAT_WGSL`
instead of writing it. `daylight.wgsl` therefore does not validate on its
own and is not meant to - `build.rs` validates the pair, which is how a
host with no height field will join them.

**The lamp's falloff takes its radius rather than naming one.** A flame's
reach is `1 / (1 + r^2 / r0^2)` cut at a distance, and both numbers are
arguments with no default. The rig this was measured from had them baked
for a grid whose cell is 0.025 world units; a renderer working in metres
that took those constants would have every lamp die a few centimetres from
itself, with nothing to see but a dark room. A shared crate cannot carry
one caller's scale.

**The Rust half is a mirror, and a test holds it there.** `Keys`, the
palette and `arc_sun`, `sun_dir`, `ambient` and `tonemap` state the same
numbers for a host that has to know where the light is without asking the
GPU - placing a drawn shadow, tinting a sprite, telling a second renderer.
Two statements of one number drift silently, so
`the_keys_are_the_ones_the_wgsl_uses` reads the WGSL this crate ships and
fails if a key here is not spelt there.

`arc_sun` carries an `allow(clippy::approx_constant)`: its literals are
the shader's digit for digit, and `FRAC_PI_6` is a different number in the
last places and so a different sun.
