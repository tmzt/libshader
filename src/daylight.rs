//! An outdoor light rig a renderer imports: the sun through a day into
//! night, the sky it hangs in, and the tone map and encoding that put the
//! result on a target.
//!
//! [`DAYLIGHT_WGSL`] is the whole of it, and is joined ahead of a
//! renderer's own source as [`HEADER_WGSL`](crate::HEADER_WGSL) is. It
//! declares no binding, no uniform and no entry point: everything it needs
//! arrives in three private globals a host sets at the start of its
//! fragment stage, or as an argument. One copy of it therefore serves a
//! renderer's own passes *and* the hooks that renderer gives someone
//! else's - the lighting hooks of a hosted prop renderer, say - and the
//! two come out the same, which is the reason this is shared rather than
//! written twice.
//!
//! A host must define `dl_height`, the height field the sun's shadow is
//! marched through, or join [`DAYLIGHT_FLAT_WGSL`] to say it has none.
//!
//! This module is the CPU's half: the same keys and the same sun, for a
//! host that has to know where the light is coming from without asking the
//! GPU - placing a drawn shadow, tinting a sprite, telling a second
//! renderer. The two halves are held together by
//! `the_keys_are_the_ones_the_wgsl_uses`, which reads the WGSL this crate
//! ships and fails if a number here is not in it.

use glam::Vec3;

/// The WGSL: the keys, the sun's arc, the sky, the lamps, the tone map and
/// the encoding. Validated by naga when this crate builds.
pub const DAYLIGHT_WGSL: &str = include_str!("daylight.wgsl");

/// `dl_height` for a host with nothing that casts a shadow: everything
/// stands on the ground plane and `dl_shadow` is a constant 1.
///
/// Joined after [`DAYLIGHT_WGSL`] instead of writing the function. WGSL has
/// no default definitions, so this is a file rather than a flag.
pub const DAYLIGHT_FLAT_WGSL: &str = include_str!("daylight_flat.wgsl");

/// The sky's time at noon.
pub const NOON: f32 = 0.0;
/// The sky's time at dusk. Below it the day blends into dusk, above it dusk
/// blends into night, so this is the one value at which every key is
/// exactly its dusk column.
pub const DUSK: f32 = 0.6;
/// The sky's time at night.
pub const NIGHT: f32 = 1.0;

/// The latitude the sun's arc is written for, as the zenith angle in
/// radians: the noon elevation is this plus the declination.
///
/// `DL_ZENITH_ANGLE` in the WGSL. 0.671952 is 90 degrees less 51.5.
pub const ZENITH_ANGLE: f32 = 0.671_952;

/// The declination at a solstice, radians. An equinox is 0.
pub const SOLSTICE_DECLINATION: f32 = 0.409_066;

/// A value of the light at the day's three keys, blended by the sky's time.
///
/// Every colour and direction in the rig is one of these. Written as a
/// struct rather than three arguments so a host can name one, pass it
/// about, and interpolate it with [`Keys::at`] exactly as the shader does.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Keys {
    /// Noon.
    pub day: [f32; 3],
    /// Dusk: amber-rose.
    pub dusk: [f32; 3],
    /// Night: blue-black, with a pale moon.
    pub night: [f32; 3],
}

impl Keys {
    /// This value at the sky's time `t`, 0 noon, 0.6 dusk, 1 night.
    ///
    /// `sqrt` from day to dusk, because early twilight is already most of
    /// the way to dusk's low amber sun and long shadows; linear from dusk
    /// to night. `dl_key` in the WGSL, and the same arithmetic.
    #[must_use]
    pub fn at(self, t: f32) -> Vec3 {
        let day = Vec3::from_array(self.day);
        let dusk = Vec3::from_array(self.dusk);
        let night = Vec3::from_array(self.night);
        if t <= NOON {
            return day;
        }
        if t < DUSK {
            return day.lerp(dusk, (t / DUSK).sqrt());
        }
        dusk.lerp(night, (t - DUSK) / (NIGHT - DUSK))
    }
}

/// The sun's light, linear. At night it is the moon's.
pub const SUN_RGB: Keys = Keys {
    day: [2.6, 2.3, 1.85],
    dusk: [1.9, 0.95, 0.5],
    night: [0.2, 0.24, 0.34],
};

/// The sky's light on a surface facing up, linear.
pub const SKY_RGB: Keys = Keys {
    day: [0.34, 0.45, 0.62],
    dusk: [0.24, 0.2, 0.28],
    night: [0.03, 0.04, 0.07],
};

/// The ground's bounce on a surface facing down, linear.
pub const BOUNCE_RGB: Keys = Keys {
    day: [0.11, 0.12, 0.06],
    dusk: [0.08, 0.06, 0.04],
    night: [0.006, 0.007, 0.008],
};

/// The sky overhead, linear.
pub const ZENITH_RGB: Keys = Keys {
    day: [0.075, 0.2, 0.52],
    dusk: [0.06, 0.07, 0.2],
    night: [0.004, 0.007, 0.02],
};

/// The sky at the horizon, linear.
pub const HORIZON_RGB: Keys = Keys {
    day: [0.42, 0.56, 0.74],
    dusk: [0.9, 0.42, 0.3],
    night: [0.016, 0.022, 0.045],
};

/// The haze the distant ground fades into, linear, so ground and sky meet
/// without a seam.
pub const HAZE_RGB: Keys = Keys {
    day: [0.62, 0.68, 0.74],
    dusk: [0.62, 0.4, 0.36],
    night: [0.02, 0.026, 0.045],
};

/// Toward the sun at `early` hours before 17:00, for declination `decl`.
///
/// Along an arc, linear in azimuth (east of south) and elevation, through
/// three keys: east-south-east and low at 07:00, due south and highest at
/// noon, west-south-west and lower at 17:00. The ends swing north of that
/// in summer and south in winter in proportion to the declination.
///
/// `dl_arc_sun` in the WGSL.
// The literals are the WGSL's, digit for digit, because the point of this
// function is to answer what the shader answers: 0.523_599 is 30 degrees
// and clippy would rather have `FRAC_PI_6`, which is a different number in
// the last places and a different sun.
#[allow(clippy::approx_constant)]
#[must_use]
pub fn arc_sun(early: f32, decl: f32) -> Vec3 {
    let s = (decl / SOLSTICE_DECLINATION).clamp(-1.0, 1.0);
    let noon = ZENITH_ANGLE + decl;
    let hour = 17.0 - early.clamp(0.0, 10.0);
    let (az, el) = if hour < 12.0 {
        let u = (hour - 7.0) / 5.0;
        (
            lerp(1.178_097 + 0.436_332 * s, 0.0, u),
            lerp(0.3 * noon, noon, u),
        )
    } else {
        let u = (hour - 12.0) / 5.0;
        (
            lerp(0.0, -(1.008_281 + 0.523_599 * s), u),
            lerp(noon, 0.75 * noon, u),
        )
    };
    Vec3::new(az.sin() * el.cos(), el.sin(), az.cos() * el.cos())
}

/// The sun's direction at the day's three keys: the arc's 17:00 sun, then
/// 6 degrees up in the west at dusk, then the moon high in the south-east.
///
/// The dusk blend starts from the 17:00 sun whatever the hour was, which is
/// why the day key here is not the arc at the hour.
pub const SUN_DIR: Keys = Keys {
    // Filled from `arc_sun(0.0, 0.0)` by `sun_dir`; the literal is the
    // dusk and night pair the WGSL writes.
    day: [0.0, 0.0, 0.0],
    dusk: [-0.92, 0.105, 0.38],
    night: [0.45, 0.7, 0.55],
};

/// Toward the sun at the sky's time `t`, for the hour `early` and the
/// declination `decl`.
///
/// By day it follows the hour's arc; from twilight on, the arc is left at
/// its 17:00 position and the dusk and night keys take over.
///
/// `dl_sun` in the WGSL.
#[must_use]
pub fn sun_dir(t: f32, early: f32, decl: f32) -> Vec3 {
    if t <= NOON {
        return arc_sun(early, decl);
    }
    let keys = Keys {
        day: arc_sun(0.0, decl).to_array(),
        ..SUN_DIR
    };
    keys.at(t).normalize()
}

/// The light from the sky and the ground's bounce on a surface facing `n`,
/// at the sky's time `t`, linear. `dl_ambient` in the WGSL.
#[must_use]
pub fn ambient(n: Vec3, t: f32) -> Vec3 {
    let up = 0.5 + 0.5 * n.y;
    SKY_RGB.at(t) * up + BOUNCE_RGB.at(t) * (1.0 - up)
}

/// The ACES filmic curve (Narkowicz's fit), per channel. `dl_tonemap`.
#[must_use]
pub fn tonemap(c: Vec3) -> Vec3 {
    let x = c.max(Vec3::ZERO);
    ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(Vec3::ZERO, Vec3::ONE)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::{
        BOUNCE_RGB, DAYLIGHT_FLAT_WGSL, DAYLIGHT_WGSL, DUSK, HAZE_RGB, HORIZON_RGB, Keys, NIGHT,
        NOON, SKY_RGB, SUN_DIR, SUN_RGB, ZENITH_ANGLE, ZENITH_RGB, ambient, arc_sun, sun_dir,
        tonemap,
    };
    use glam::Vec3;

    /// Every key this module states is the one the WGSL states.
    ///
    /// The two halves are written out separately - a `const` here, a
    /// `vec3<f32>` there - so nothing but a test stops them drifting, and
    /// a host that places a drawn shadow from the Rust sun while the GPU
    /// lights from another one has a fault with no error message. The
    /// numbers are matched as the WGSL spells them.
    #[test]
    fn the_keys_are_the_ones_the_wgsl_uses() {
        let named = [
            ("SUN_RGB", SUN_RGB),
            ("SKY_RGB", SKY_RGB),
            ("BOUNCE_RGB", BOUNCE_RGB),
            ("ZENITH_RGB", ZENITH_RGB),
            ("HORIZON_RGB", HORIZON_RGB),
            ("HAZE_RGB", HAZE_RGB),
        ];
        for (name, keys) in named {
            for (part, rgb) in [
                ("day", keys.day),
                ("dusk", keys.dusk),
                ("night", keys.night),
            ] {
                let spelt = format!("vec3<f32>({}, {}, {})", rgb[0], rgb[1], rgb[2]);
                assert!(
                    DAYLIGHT_WGSL.contains(&spelt),
                    "{name}'s {part} key is {spelt} here and is not in the WGSL",
                );
            }
        }
        for (part, rgb) in [("dusk", SUN_DIR.dusk), ("night", SUN_DIR.night)] {
            let spelt = format!("vec3<f32>({}, {}, {})", rgb[0], rgb[1], rgb[2]);
            assert!(
                DAYLIGHT_WGSL.contains(&spelt),
                "the sun's {part} direction is {spelt} here and is not in the WGSL",
            );
        }
        assert!(
            DAYLIGHT_WGSL.contains(&format!("const DL_ZENITH_ANGLE: f32 = {ZENITH_ANGLE}")),
            "the zenith angle differs from the WGSL's",
        );
    }

    /// The flat height field defines what `daylight.wgsl` requires and
    /// nothing else, so a host joining it gains no name it did not ask for.
    #[test]
    fn the_flat_height_field_is_the_one_function() {
        assert!(DAYLIGHT_FLAT_WGSL.contains("fn dl_height(c: vec2<i32>) -> f32"));
        assert_eq!(DAYLIGHT_FLAT_WGSL.matches("fn ").count(), 1);
        // A definition, not the mention of one: the file's own header
        // names the function it requires, which is the point.
        assert!(
            !DAYLIGHT_WGSL
                .lines()
                .any(|l| l.starts_with("fn dl_height(")),
            "daylight.wgsl must leave dl_height to its host",
        );
        assert!(
            DAYLIGHT_WGSL.contains("dl_height("),
            "daylight.wgsl is meant to call dl_height",
        );
    }

    /// The three keys are reached exactly, and the blend between them is
    /// the shader's: `sqrt` below dusk, linear above.
    #[test]
    fn a_key_is_reached_at_its_own_time() {
        let k = Keys {
            day: [1.0, 0.0, 0.0],
            dusk: [0.0, 1.0, 0.0],
            night: [0.0, 0.0, 1.0],
        };
        assert_eq!(k.at(NOON), Vec3::X);
        assert_eq!(k.at(DUSK), Vec3::Y);
        assert_eq!(k.at(NIGHT), Vec3::Z);
        // Half way to dusk in time is `sqrt(0.5)` of the way in colour, so
        // early twilight is already most of the way there.
        let half = k.at(DUSK / 2.0);
        assert!((half.y - 0.5_f32.sqrt()).abs() < 1e-6, "{half}");
        // Below noon is clamped rather than extrapolated.
        assert_eq!(k.at(-1.0), Vec3::X);
    }

    /// Dusk is a low sun in the west, which is the whole of its look.
    #[test]
    fn the_dusk_sun_is_six_degrees_up_in_the_west() {
        let sun = sun_dir(DUSK, 0.0, 0.0);
        assert!((sun.length() - 1.0).abs() < 1e-6, "not a unit vector");
        let elevation = sun.y.asin().to_degrees();
        assert!(
            (elevation - 6.0).abs() < 0.5,
            "the dusk sun is {elevation} degrees up",
        );
        assert!(sun.x < -0.9, "the dusk sun is in the west: {sun}");
    }

    /// At noon the sun is due south and highest, and the declination moves
    /// it by exactly itself.
    #[test]
    fn the_arc_is_highest_at_noon_and_follows_the_date() {
        // 17:00 less five hours is noon.
        let noon = arc_sun(5.0, 0.0);
        assert!(noon.x.abs() < 1e-6, "due south: {noon}");
        assert!(noon.z > 0.0, "south is +z");
        let flat = noon.y.asin();
        assert!((flat - ZENITH_ANGLE).abs() < 1e-5);
        let midsummer = arc_sun(5.0, super::SOLSTICE_DECLINATION);
        assert!(
            midsummer.y.asin() > flat,
            "a summer noon is higher than an equinox's",
        );
        // Morning is east, evening is west, and both are lower.
        assert!(arc_sun(10.0, 0.0).x > 0.5, "07:00 is east of south");
        assert!(arc_sun(0.0, 0.0).x < -0.5, "17:00 is west of south");
        assert!(arc_sun(10.0, 0.0).y < noon.y && arc_sun(0.0, 0.0).y < noon.y);
    }

    /// The ambient is the sky above and the bounce below, and it darkens
    /// through the day. Stated because a host that tints its own sprites
    /// reads this rather than the shader.
    #[test]
    fn the_ambient_turns_over_and_darkens() {
        for t in [NOON, DUSK, NIGHT] {
            let up = ambient(Vec3::Y, t);
            let down = ambient(Vec3::NEG_Y, t);
            assert_eq!(up, SKY_RGB.at(t));
            assert_eq!(down, BOUNCE_RGB.at(t));
            assert!(up.length() > down.length(), "the sky outshines the bounce");
        }
        assert!(ambient(Vec3::Y, NOON).length() > ambient(Vec3::Y, DUSK).length());
        assert!(ambient(Vec3::Y, DUSK).length() > ambient(Vec3::Y, NIGHT).length());
    }

    /// The tone map is bounded, never falls, leaves black alone and does
    /// not reach white early - the four things a caller relies on when it
    /// maps its own colours through it. The sun's own `(2.6, 2.3, 1.85)`
    /// at noon is in the range that matters, so a curve that saturated at
    /// 2 would flatten a lit surface to paper.
    #[test]
    fn the_tone_map_is_bounded_and_never_falls() {
        assert_eq!(tonemap(Vec3::ZERO), Vec3::ZERO);
        let mut last = -1.0;
        for i in 0..64u8 {
            let v = tonemap(Vec3::splat(f32::from(i) * 0.5)).x;
            assert!((0.0..=1.0).contains(&v), "{v} is outside the target");
            assert!(v >= last, "the curve falls at {i}");
            last = v;
        }
        assert!(tonemap(Vec3::splat(3.0)).x < 1.0, "white arrives too early");
        assert!(tonemap(Vec3::splat(1.0)).x > tonemap(Vec3::splat(0.5)).x);
    }
}
