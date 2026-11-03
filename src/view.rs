//! The orthographic view a renderer draws through, as the two angles and the
//! framing both sides have to agree on.
//!
//! A renderer turns these six numbers into a basis on the CPU and the same
//! basis in its shader, so the view belongs beside the headers rather than in
//! any one renderer: two renderers drawing one frame have to be told the same
//! camera, and a host that frames a scene has to be able to state it once.

use std::f32::consts::FRAC_PI_4;

use glam::{EulerRot, Quat, Vec3};

/// The isometric pitch: the angle at which a cube's three visible faces are
/// foreshortened equally, `atan(1 / sqrt 2)`.
///
/// The same angle is `asin(1 / sqrt 3)`, the elevation of the view direction
/// `(1, 1, 1) / sqrt 3`, which is how a scene built on a unit grid usually
/// states it. [`View::isometric`] is this pitch at yaw 45 degrees.
pub const ISOMETRIC_PITCH: f32 = 0.615_479_7;

/// Furthest the pitch may tilt either way, radians, as an orbit control holds
/// it: short of straight up or down, where yaw stops meaning anything.
pub const PITCH_LIMIT: f32 = 1.45;

/// An orthographic view, described the way a turntable is: how far it has
/// turned about `+y` and tipped down toward the ground, how much of the world
/// it shows and where it is centred.
///
/// It is six `f32` and no behaviour of its own beyond reading them, so a host
/// may store it, interpolate it or send it to a shader as it likes. The
/// framing is in world units: [`scale`](Self::scale) is a length, not a zoom,
/// so the same view frames the same amount of world at any resolution.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct View {
    /// Radians about `+y`. Zero looks along `-z`.
    pub yaw: f32,
    /// Radians the camera is raised above the horizon, looking down.
    pub pitch: f32,
    /// World half-height of the frame, in the scene's own units.
    pub scale: f32,
    /// Screen-right offset of the frame's centre, in the scene's own units.
    pub center_x: f32,
    /// Screen-up offset of the frame's centre, in the scene's own units.
    pub center_y: f32,
    /// Width over height of the target.
    pub aspect: f32,
}

impl Default for View {
    /// The front view of a square frame two units across: no turn, no tilt,
    /// centred on the origin.
    ///
    /// Neutral rather than a framing of anything, so what a renderer or a
    /// host frames is that caller's to state. Only `scale` and `aspect` are
    /// non-zero, both at 1.
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            scale: 1.0,
            center_x: 0.0,
            center_y: 0.0,
            aspect: 1.0,
        }
    }
}

impl View {
    /// The isometric view: yaw 45 degrees, pitch [`ISOMETRIC_PITCH`], with
    /// [`Default`]'s framing.
    #[must_use]
    pub fn isometric() -> Self {
        Self {
            yaw: FRAC_PI_4,
            pitch: ISOMETRIC_PITCH,
            ..Self::default()
        }
    }

    /// The pitch, held inside [`PITCH_LIMIT`]. Everything that reads the view
    /// - the basis, the projection and a frame uniform - uses this.
    #[must_use]
    pub fn clamped_pitch(self) -> f32 {
        self.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT)
    }

    /// The camera's orientation: turned by the yaw about `+y`, then tipped
    /// down by the pitch. It takes `+x` to screen right, `+y` to screen up and
    /// `+z` to the direction toward the viewer. Raising the camera tips its
    /// view axis down, which is a negative turn about its own `x`.
    #[must_use]
    pub fn rotation(self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, -self.clamped_pitch(), 0.0)
    }

    /// The camera's three axes in world space: screen right, screen up, and
    /// toward the viewer. A shader builds the same basis from the same two
    /// angles, which is the agreement this type exists for.
    #[must_use]
    pub fn basis(self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let rotation = self.rotation();
        (
            (rotation * Vec3::X).to_array(),
            (rotation * Vec3::Y).to_array(),
            (rotation * Vec3::Z).to_array(),
        )
    }

    /// Where a world point lands in normalised device coordinates, `-1..1`
    /// on both axes with `+y` up.
    #[must_use]
    pub fn project(self, world: [f32; 3]) -> [f32; 2] {
        let local = self.rotation().inverse() * Vec3::from_array(world);
        [
            (local.x - self.center_x) / (self.scale * self.aspect),
            (local.y - self.center_y) / self.scale,
        ]
    }

    /// Where a world point lands in a `width` by `height` frame, pixels from
    /// the top-left corner.
    #[must_use]
    pub fn to_pixel(self, world: [f32; 3], width: u32, height: u32) -> [f32; 2] {
        let [x, y] = self.project(world);
        [
            (x * 0.5 + 0.5) * width as f32,
            (0.5 - y * 0.5) * height as f32,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{ISOMETRIC_PITCH, PITCH_LIMIT, View};
    use glam::Vec3;
    use std::f32::consts::FRAC_PI_4;

    #[test]
    fn the_front_view_looks_down_minus_z() {
        let view = View {
            yaw: 0.0,
            pitch: 0.0,
            ..View::default()
        };
        let (right, up, toward) = view.basis();
        assert_eq!(right, [1.0, 0.0, 0.0]);
        assert_eq!(up, [0.0, 1.0, 0.0]);
        assert_eq!(toward, [0.0, 0.0, 1.0]);
    }

    /// The default frames two units by two and nothing else: a point at its
    /// own half-extent lands on the edge, and the origin in the middle.
    #[test]
    fn the_default_frames_two_units_square() {
        let view = View::default();
        assert_eq!(view.project([0.0, 0.0, -5.0]), [0.0, 0.0]);
        assert_eq!(view.project([1.0, 1.0, 0.0]), [1.0, 1.0]);
        assert_eq!(view.project([-1.0, -1.0, 0.0]), [-1.0, -1.0]);
    }

    #[test]
    fn the_frame_centre_projects_to_the_middle() {
        let view = View {
            center_x: 0.3,
            center_y: 0.4,
            ..View::default()
        };
        assert_eq!(view.project([0.3, 0.4, -2.0]), [0.0, 0.0]);
    }

    #[test]
    fn the_basis_matches_the_shaders_angle_formula() {
        for (yaw, pitch) in [(0.0f32, 0.0f32), (0.6, 0.35), (2.1, -0.9), (-1.3, 1.2)] {
            let view = View {
                yaw,
                pitch,
                ..View::default()
            };
            let (sy, cy) = yaw.sin_cos();
            let (sp, cp) = pitch.sin_cos();
            let toward = Vec3::new(sy * cp, sp, cy * cp);
            let right = Vec3::new(cy, 0.0, -sy);
            let up = toward.cross(right);
            let (r, u, t) = view.basis();
            for (got, want) in [(r, right), (u, up), (t, toward)] {
                assert!(
                    Vec3::from_array(got).abs_diff_eq(want, 1e-6),
                    "{got:?} vs {want}"
                );
            }
        }
    }

    #[test]
    fn pitch_is_held_short_of_the_poles() {
        let view = View {
            pitch: 3.0,
            ..View::default()
        };
        assert_eq!(view.clamped_pitch(), PITCH_LIMIT);
        let (_, _, toward) = view.basis();
        assert!((toward[1] - PITCH_LIMIT.sin()).abs() < 1e-6);
    }

    /// The isometric pitch stated two ways. `atan(1 / sqrt 2)` is the angle at
    /// which a cube's three faces foreshorten equally; `asin(1 / sqrt 3)` is
    /// the elevation of `(1, 1, 1) / sqrt 3`. They are the same number, so a
    /// scene stating either gets this view.
    #[test]
    fn the_isometric_pitch_is_the_same_angle_either_way() {
        let from_faces = (1.0f32 / 2.0f32.sqrt()).atan();
        let from_direction = (1.0f32 / 3.0f32.sqrt()).asin();
        assert!((from_faces - from_direction).abs() < 1e-6);
        assert!((ISOMETRIC_PITCH - from_faces).abs() < 1e-6);

        let view = View::isometric();
        assert!((view.yaw - FRAC_PI_4).abs() < 1e-6);
        assert!((view.clamped_pitch() - ISOMETRIC_PITCH).abs() < 1e-6);
    }

    /// Under the isometric view a world-vertical foreshortens by `sqrt(2/3)`,
    /// which is what a renderer scales a drawn height by. The number is a
    /// consequence of the pitch rather than a constant of its own.
    #[test]
    fn a_world_vertical_foreshortens_by_sqrt_two_thirds() {
        let (_, up, _) = View::isometric().basis();
        let want = (2.0f32 / 3.0).sqrt();
        assert!((up[1] - want).abs() < 1e-6, "{} vs {want}", up[1]);
    }
}
