// An outdoor light rig: the sun through a day into night, the sky it hangs
// in, the lamps a host adds, and the tone map and encoding that put the
// result on a target. Linear light throughout, until `dl_encode`.
//
// A host joins this file ahead of its own source, as it joins the shared
// header. Nothing here declares a binding, a uniform, an entry point or a
// vertex format: every input arrives in the three private globals below,
// which a host sets at the start of its fragment stage, or as an argument.
// So one copy of this serves a renderer's own passes and the hooks it gives
// someone else's renderer, and the two are lit the same.
//
// **A host must define one function**, the height field the sun's shadow is
// marched through:
//
//     fn dl_height(c: vec2<i32>) -> f32
//
// the height of whatever casts shadows over cell `c`. A host with no such
// field joins `DAYLIGHT_FLAT_WGSL` instead of writing one; `dl_shadow` is
// then a constant 1 and costs nothing, every other function being
// unaffected.
//
// Names are prefixed `dl_` because this text is concatenated into a host's
// own module, where an unprefixed name could collide now or later.

// The sky's time, and the one number the whole rig is keyed on: 0 noon,
// 0.6 dusk, 1 night. A host sets it from its own clock at the start of its
// fragment stage. At 0 every value below is exactly its daylight column.
var<private> dl_t: f32 = 0.0;
// The hour of day as hours before 17:00, 0 to 10. Only the day follows it
// (`dl_t == 0`); from twilight on, the sun's arc is left at its 17:00
// position and the dusk key takes over.
var<private> dl_early: f32 = 0.0;
// The sun's declination for the date, radians: 0 at an equinox, +-0.409066
// at a solstice.
var<private> dl_decl: f32 = 0.0;

// The latitude the sun's arc is written for, radians north. The noon
// elevation is `DL_ZENITH_ANGLE + dl_decl`, so a world at another latitude
// changes this one constant.
const DL_ZENITH_ANGLE: f32 = 0.671952; // 90 - 51.5 degrees

// The three keys of a value: day, dusk (amber-rose) and night (blue-black
// with a pale moon), blended by the sky's time.
fn dl_key(day: vec3<f32>, dusk: vec3<f32>, night: vec3<f32>) -> vec3<f32> {
    if (dl_t <= 0.0) {
        return day;
    }
    if (dl_t < 0.6) {
        // sqrt: early twilight (0.3) is already most of the way to dusk's
        // low amber sun and long shadows.
        return mix(day, dusk, sqrt(dl_t / 0.6));
    }
    return mix(dusk, night, (dl_t - 0.6) / 0.4);
}

// The day's sun at `early` hours before 17:00 for the declination
// `dl_decl`: along an arc, linear in azimuth (east of south) and elevation,
// through three keys. Noon: due south, `DL_ZENITH_ANGLE` plus the
// declination up. 07:00: east-south-east (67.5 degrees east of south) at
// 0.3 of the noon height; 17:00: west-south-west (57.8 degrees west of
// south) at 0.75 of it. The ends swing north of that in summer and south in
// winter in proportion to the declination (25 degrees at 07:00, 30 at 17:00
// at the solstice).
fn dl_arc_sun(early: f32) -> vec3<f32> {
    let s = clamp(dl_decl / 0.409066, -1.0, 1.0);
    let noon = DL_ZENITH_ANGLE + dl_decl;
    let hour = 17.0 - clamp(early, 0.0, 10.0);
    var az = 0.0;
    var el = 0.0;
    if (hour < 12.0) {
        let u = (hour - 7.0) / 5.0;
        az = mix(1.178097 + 0.436332 * s, 0.0, u);
        el = mix(0.3 * noon, noon, u);
    } else {
        let u = (hour - 12.0) / 5.0;
        az = mix(0.0, -(1.008281 + 0.523599 * s), u);
        el = mix(noon, 0.75 * noon, u);
    }
    return vec3<f32>(sin(az) * cos(el), sin(el), cos(az) * cos(el));
}

// Toward the sun: by day along the hour's arc (`dl_arc_sun`: low in the
// morning's east-south-east, high in the south at noon, lower in the
// west-south-west late, so south-facing surfaces are lit and shadows fall
// north); at dusk 6 degrees up in the west; at night toward the moon, high
// in the south-east. The dusk's blend starts from the arc's 17:00 sun for
// the date, whatever the hour was.
fn dl_sun() -> vec3<f32> {
    if (dl_t <= 0.0) {
        return dl_arc_sun(dl_early);
    }
    return normalize(dl_key(
        dl_arc_sun(0.0),
        vec3<f32>(-0.92, 0.105, 0.38),
        vec3<f32>(0.45, 0.7, 0.55),
    ));
}
// The sun's light, linear (the moon's at night), the sky's (the ambient
// from above) and the ground's bounce (from below).
fn dl_sun_rgb() -> vec3<f32> {
    return dl_key(vec3<f32>(2.6, 2.3, 1.85), vec3<f32>(1.9, 0.95, 0.5), vec3<f32>(0.2, 0.24, 0.34));
}
fn dl_sky_rgb() -> vec3<f32> {
    return dl_key(vec3<f32>(0.34, 0.45, 0.62), vec3<f32>(0.24, 0.2, 0.28), vec3<f32>(0.03, 0.04, 0.07));
}
fn dl_bounce_rgb() -> vec3<f32> {
    return dl_key(vec3<f32>(0.11, 0.12, 0.06), vec3<f32>(0.08, 0.06, 0.04), vec3<f32>(0.006, 0.007, 0.008));
}
// The sky, linear: the zenith, the horizon and the haze at the horizon (the
// distant ground fades into it, so ground and sky meet without a seam).
fn dl_zenith() -> vec3<f32> {
    return dl_key(vec3<f32>(0.075, 0.2, 0.52), vec3<f32>(0.06, 0.07, 0.2), vec3<f32>(0.004, 0.007, 0.02));
}
fn dl_horizon() -> vec3<f32> {
    return dl_key(vec3<f32>(0.42, 0.56, 0.74), vec3<f32>(0.9, 0.42, 0.3), vec3<f32>(0.016, 0.022, 0.045));
}
fn dl_haze() -> vec3<f32> {
    return dl_key(vec3<f32>(0.62, 0.68, 0.74), vec3<f32>(0.62, 0.4, 0.36), vec3<f32>(0.02, 0.026, 0.045));
}

fn dl_hashu(x: u32) -> u32 {
    var h = x * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

fn dl_hash2(p: vec2<i32>) -> f32 {
    let h = dl_hashu(bitcast<u32>(p.x) * 1597334677u ^ dl_hashu(bitcast<u32>(p.y)));
    return f32(h >> 8u) / 16777215.0;
}

// `dl_hash2(vec2(x, y))` given `hy = dl_hashu(bitcast<u32>(y))`.
fn dl_hash2_row(x: i32, hy: u32) -> f32 {
    let h = dl_hashu(bitcast<u32>(x) * 1597334677u ^ hy);
    return f32(h >> 8u) / 16777215.0;
}

// Value noise. The corners' rows are hashed once each, which is the same
// values for half the row hashes.
fn dl_noise(p: vec2<f32>) -> f32 {
    let i = vec2<i32>(floor(p));
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let h0 = dl_hashu(bitcast<u32>(i.y));
    let h1 = dl_hashu(bitcast<u32>(i.y + 1));
    let a = dl_hash2_row(i.x, h0);
    let b = dl_hash2_row(i.x + 1, h0);
    let c = dl_hash2_row(i.x, h1);
    let d = dl_hash2_row(i.x + 1, h1);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Four octaves, 0..1.
fn dl_fbm(p: vec2<f32>) -> f32 {
    var q = p;
    var a = 0.5;
    var s = 0.0;
    for (var k = 0; k < 4; k = k + 1) {
        s = s + a * dl_noise(q);
        q = vec2<f32>(q.x * 1.6 - q.y * 1.2, q.x * 1.2 + q.y * 1.6) + vec2<f32>(5.2, 1.3);
        a = a * 0.5;
    }
    return s / 0.9375;
}

// The ACES filmic curve (Narkowicz's fit).
fn dl_tonemap(c: vec3<f32>) -> vec3<f32> {
    let x = max(c, vec3<f32>(0.0));
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn dl_to_srgb(c: f32) -> f32 {
    if (c <= 0.0031308) {
        return 12.92 * c;
    }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

fn dl_to_linear(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

// Linear light to the target's space: an sRGB target encodes by itself.
fn dl_encode(c: vec3<f32>, srgb_target: bool) -> vec3<f32> {
    if (srgb_target) {
        return c;
    }
    return vec3<f32>(dl_to_srgb(c.r), dl_to_srgb(c.g), dl_to_srgb(c.b));
}

// A colour in the target's space to linear light.
fn dl_decode(c: vec3<f32>, srgb_target: bool) -> vec3<f32> {
    if (srgb_target) {
        return c;
    }
    return vec3<f32>(dl_to_linear(c.r), dl_to_linear(c.g), dl_to_linear(c.b));
}

// A flame - a wall sconce, a table lamp, a lamp on a stand - as a warm point
// light of about 2400 K on a surface at `p` facing `n`. `light` is its world
// position in `xyz` and its strength in `w`.
//
// It falls off as `1 / (1 + r^2 / r0^2)`, so `r0` is the distance at which
// it is half as strong, and it is cut at `cut`, fading out from five eighths
// of the way there. Both are in the host's own world units and neither has a
// default: a rig stated in metres and one stated in a grid's cells want
// different numbers, and a wrong guess is a lamp that lights nothing a few
// of its own radii away. A surface takes the light by its facing alone;
// whether the flame can see the surface at all is `dl_torch_seen`.
fn dl_torch(p: vec3<f32>, n: vec3<f32>, light: vec4<f32>, r0: f32, cut: f32) -> vec3<f32> {
    let d = light.xyz - p;
    let r2 = dot(d, d) + 1e-6;
    let l = d * inverseSqrt(r2);
    let reach = 1.0 - smoothstep(0.625 * cut, cut, sqrt(r2));
    let k = light.w * reach / (1.0 + r2 / max(r0 * r0, 1e-12));
    return vec3<f32>(1.0, 0.62, 0.35) * k * max(dot(n, l), 0.0);
}

// Whether the flame `light` sees the point `p`: 1 when nothing stands on the
// segment between them, 0 when something does. Eight samples from a tenth to
// four fifths of the way to the light, each against `dl_height` over its
// cell on a grid of `cell` measured from `-half`; the last fifth is left
// out, as a flame usually sits on the thing it is mounted to.
fn dl_torch_seen(p: vec3<f32>, light: vec4<f32>, cell: f32, half: f32) -> f32 {
    for (var k = 0; k < 8; k = k + 1) {
        let q = mix(p, light.xyz, 0.1 + 0.1 * f32(k));
        let c = vec2<i32>(floor((q.xz + half) / cell));
        if (dl_height(c) > q.y) {
            return 0.0;
        }
    }
    return 1.0;
}

// The light from the sky and the ground's bounce on a surface facing `n`.
fn dl_ambient(n: vec3<f32>) -> vec3<f32> {
    let up = 0.5 + 0.5 * n.y;
    return dl_sky_rgb() * up + dl_bounce_rgb() * (1.0 - up);
}

// The sky along the unit direction `d`: the gradient, the haze at the
// horizon, soft clouds on a high plane, the sun's disc and glow.
fn dl_sky(d: vec3<f32>) -> vec3<f32> {
    let elev = max(d.y, 0.0);
    var sky = mix(dl_horizon(), dl_zenith(), pow(elev, 0.5));
    sky = mix(sky, dl_haze(), exp(-elev * 14.0));
    let sun = dl_sun();
    let mu = max(dot(d, sun), 0.0);
    // The clouds and the glow follow the light: dark at night.
    let lit_k = dl_key(vec3<f32>(1.0), vec3<f32>(0.6, 0.45, 0.4), vec3<f32>(0.03, 0.035, 0.05));
    // The sun's warm glow, wide and narrow.
    sky = sky + vec3<f32>(1.0, 0.8, 0.55) * lit_k * (0.22 * pow(mu, 8.0) + 0.6 * pow(mu, 180.0));
    // Clouds: fbm on the plane a fixed height up, thinning to the horizon.
    if (d.y > 0.01) {
        let uv = d.xz / (d.y + 0.12) * 1.8;
        let c = dl_fbm(uv + vec2<f32>(3.1, 7.4));
        let cover = smoothstep(0.52, 0.78, c) * smoothstep(0.02, 0.2, d.y);
        // Lit from the sun's side, darker beneath.
        let lit = dl_fbm(uv + sun.xz * 0.08 + vec2<f32>(3.1, 7.4));
        let shade = clamp(0.5 + 2.0 * (c - lit), 0.0, 1.0);
        let cloud = (mix(vec3<f32>(1.15, 1.12, 1.08), vec3<f32>(0.5, 0.55, 0.62), shade)
            + vec3<f32>(0.6, 0.45, 0.3) * pow(mu, 12.0)) * lit_k;
        sky = mix(sky, cloud, cover * 0.9);
    }
    // The sun's disc, soft-edged: about a degree and a half across at the
    // core, fading to about three (the same for the moon).
    let disc = smoothstep(0.99965, 0.99990, mu);
    // At night the disc is the moon's, pale and much dimmer.
    let disc_rgb = dl_key(vec3<f32>(28.0, 25.0, 20.0), vec3<f32>(30.0, 14.0, 6.0), vec3<f32>(1.6, 1.7, 1.9));
    return mix(sky, disc_rgb, disc);
}

// A flat look's light at the sky's time `t`, for a pass that draws no sun
// and no sky and wants only the hour's colour: unchanged by day (0), warmer
// and dimmer toward dusk (0.6), as dim from dusk on.
fn dl_plain_light(c: vec3<f32>, t: f32) -> vec3<f32> {
    if (t <= 0.0) {
        return c;
    }
    let early = vec3<f32>(0.66, 0.5, 0.38);
    let dusk = vec3<f32>(0.55, 0.4, 0.33);
    var tint = mix(vec3<f32>(1.0), early, smoothstep(0.0, 0.3, t));
    tint = mix(tint, dusk, smoothstep(0.3, 0.6, t));
    return c * tint;
}

// The sky in a flat look: a warm glow low toward the horizon as the light
// goes (zero by day), strongest by early twilight. `low` is 0 at the horizon
// and 1 overhead.
fn dl_plain_sky(sky: vec3<f32>, t: f32, low: f32) -> vec3<f32> {
    let k = smoothstep(0.0, 0.3, t);
    return mix(sky, vec3<f32>(0.95, 0.52, 0.26), 0.55 * k * (1.0 - low));
}

// The haze along a ray of `dist` from the eye: full at `haze_distance`.
fn dl_aerial(c: vec3<f32>, dist: f32, haze_distance: f32) -> vec3<f32> {
    let k = clamp(dist / haze_distance, 0.0, 1.0);
    return mix(c, dl_haze(), 1.0 - exp(-4.0 * k * k) * (1.0 - k * k));
}

// The sun's soft shadow at `p` (world units), marched through the host's
// height field (`dl_height`) on a grid of `cell` measured from `-half`: 1
// lit, 0 in shadow, soft with the distance to the occluder. `top` is the
// tallest cell's height; nothing above it casts a shadow. The samples are at
// `t = (k + 0.5) dt`, `k < 64`, while below `top`.
//
// `rect` bounds the height field - world x, z, minimum then maximum - and
// only the samples over it read a height. A low sun otherwise marches all 64
// samples over empty ground for every fragment, which is the difference
// between a frame and a slideshow on a mobile part. Over bare ground the
// softness term `5 (p.y + sun.y t) / t` is monotone in `t`, so its least
// over each run of such samples is at one of the run's ends: the same
// shadow, without the loads.
fn dl_shadow(p: vec3<f32>, cell: f32, half: f32, top: f32, rect: vec4<f32>) -> f32 {
    if (top <= 0.0) {
        return 1.0;
    }
    let sun = dl_sun();
    let horiz = length(sun.xz);
    let dt = 0.3 * cell / horiz;
    // The samples below the top: k < n.
    var n = 64;
    if (sun.y > 1e-6) {
        n = clamp(i32(floor((top - p.y) / (sun.y * dt) - 0.5)) + 1, 0, 64);
    }
    if (n <= 0) {
        return 1.0;
    }
    // The run of samples over the field's bounds: k in [kin, kout].
    let o = p.xz;
    let d = select(sun.xz, vec2<f32>(1e-9), abs(sun.xz) < vec2<f32>(1e-9));
    let ta = (rect.xy - o) / d;
    let tb = (rect.zw - o) / d;
    let tin = max(min(ta.x, tb.x), min(ta.y, tb.y));
    let tout = min(max(ta.x, tb.x), max(ta.y, tb.y));
    var kin = n;
    var kout = n - 1;
    if (tin <= tout) {
        kin = clamp(i32(ceil(tin / dt - 0.5)), 0, n);
        kout = clamp(i32(floor(tout / dt - 0.5)), -1, n - 1);
        if (kin > kout) {
            kin = n;
            kout = n - 1;
        }
    }
    var res = 1.0;
    // The bare runs before and after: [0, kin - 1] and [kout + 1, n - 1].
    var runs = array<vec2<i32>, 2>(vec2<i32>(0, kin - 1), vec2<i32>(kout + 1, n - 1));
    for (var r = 0; r < 2; r = r + 1) {
        let run = runs[r];
        if (run.x <= run.y) {
            for (var e = 0; e < 2; e = e + 1) {
                let t = (f32(run[e]) + 0.5) * dt;
                let h = p.y + sun.y * t;
                if (h < 0.0) {
                    return 0.0;
                }
                res = min(res, 5.0 * h / t);
            }
        }
    }
    for (var k = kin; k <= kout; k = k + 1) {
        let t = (f32(k) + 0.5) * dt;
        let q = p + sun * t;
        let c = vec2<i32>(floor((q.xz + half) / cell));
        let dh = q.y - dl_height(c);
        if (dh < 0.0) {
            return 0.0;
        }
        res = min(res, 5.0 * dh / t);
    }
    return smoothstep(0.0, 1.0, res);
}
