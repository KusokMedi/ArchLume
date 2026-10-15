//! Window-management policy.
//!
//! Currently this covers the snap zones required by the ArchLume desktop
//! (AGENTS section 6.3): dragging a floating window to the top edge maximizes
//! it, to the left/right edge snaps it to half the screen, and to a corner
//! snaps it to that quarter. Floating is the default; a drop away from any edge
//! leaves the window where it is.
//!
//! The geometry is expressed with plain `f64` rectangles so the policy can be
//! unit-tested independently of Smithay. The compositor converts to and from
//! Smithay's logical coordinates at the call site.

/// Fraction of an output's width/height that counts as "at the edge".
pub const DEFAULT_BAND_FRACTION: f64 = 0.05;

/// A rectangle in logical space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// The snap zones a dragged window can be dropped into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapZone {
    Maximize,
    LeftHalf,
    RightHalf,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl SnapZone {
    pub fn is_maximized(self) -> bool {
        matches!(self, SnapZone::Maximize)
    }
}

/// Decide which snap zone (if any) a cursor position falls into.
///
/// `pos` and `output` are in the same logical coordinate space. A position
/// within `band` (a fraction of the output dimension) of the top edge means the
/// top zone; left/right edges and corners combine accordingly.
pub fn zone_for(pos: (f64, f64), output: Rect, band: f64) -> Option<SnapZone> {
    let (px, py) = pos;
    let band_x = output.w * band;
    let band_y = output.h * band;

    let left = px - output.x <= band_x;
    let right = (output.x + output.w) - px <= band_x;
    let top = py - output.y <= band_y;
    let bottom = (output.y + output.h) - py <= band_y;

    if top && left {
        Some(SnapZone::TopLeft)
    } else if top && right {
        Some(SnapZone::TopRight)
    } else if bottom && left {
        Some(SnapZone::BottomLeft)
    } else if bottom && right {
        Some(SnapZone::BottomRight)
    } else if top {
        Some(SnapZone::Maximize)
    } else if left {
        Some(SnapZone::LeftHalf)
    } else if right {
        Some(SnapZone::RightHalf)
    } else {
        None
    }
}

/// Compute the target geometry for a snap zone within `output`.
pub fn geometry_for(zone: SnapZone, output: Rect) -> Rect {
    let (x, y, w, h) = (output.x, output.y, output.w, output.h);
    let half_w = w / 2.0;
    let half_h = h / 2.0;
    let (rx, ry, rw, rh) = match zone {
        SnapZone::Maximize => (x, y, w, h),
        SnapZone::LeftHalf => (x, y, half_w, h),
        SnapZone::RightHalf => (x + half_w, y, half_w, h),
        SnapZone::TopLeft => (x, y, half_w, half_h),
        SnapZone::TopRight => (x + half_w, y, half_w, half_h),
        SnapZone::BottomLeft => (x, y + half_h, half_w, half_h),
        SnapZone::BottomRight => (x + half_w, y + half_h, half_w, half_h),
    };
    Rect {
        x: rx,
        y: ry,
        w: rw,
        h: rh,
    }
}

/// Convenience helper: decide a zone for `pos` and return its geometry.
pub fn resolve(pos: (f64, f64), output: Rect, band: f64) -> Option<(SnapZone, Rect)> {
    zone_for(pos, output, band).map(|zone| (zone, geometry_for(zone, output)))
}

/// Whether a window rectangle's centre lies inside a workspace region on the
/// horizontal axis. Workspaces are laid out side by side, so only `x` matters.
pub fn in_region(rect: Rect, region_x: f64, region_w: f64) -> bool {
    let centre_x = rect.x + rect.w / 2.0;
    centre_x >= region_x && centre_x < region_x + region_w
}

/// Remembers the floating size of each snapped window so the compositor can
/// restore it when the window is dragged back to the centre (un-snap, AGENTS
/// section 6.3). Keyed by an arbitrary window identity supplied by the caller.
///
/// The first size recorded for a window is kept: repeatedly re-snapping an
/// already-snapped window must not overwrite the original floating size.
#[derive(Debug, Clone)]
pub struct SnapMemory<K> {
    floating: std::collections::HashMap<K, (i32, i32)>,
}

impl<K> Default for SnapMemory<K> {
    fn default() -> Self {
        Self {
            floating: std::collections::HashMap::new(),
        }
    }
}

impl<K> SnapMemory<K> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<K: Eq + std::hash::Hash> SnapMemory<K> {
    /// Record `size` as the window's floating size, unless one is already known.
    pub fn remember(&mut self, key: K, size: (i32, i32)) {
        self.floating.entry(key).or_insert(size);
    }

    /// Whether the window is currently snapped (has a remembered floating size).
    pub fn is_snapped(&self, key: &K) -> bool {
        self.floating.contains_key(key)
    }

    /// Take and forget the remembered floating size, if any.
    pub fn restore(&mut self, key: &K) -> Option<(i32, i32)> {
        self.floating.remove(key)
    }

    /// Forget the window without restoring (e.g. it was closed).
    pub fn forget(&mut self, key: &K) {
        self.floating.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn out() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            w: 1280.0,
            h: 720.0,
        }
    }

    #[test]
    fn center_is_floating() {
        assert_eq!(zone_for((640.0, 360.0), out(), DEFAULT_BAND_FRACTION), None);
        assert_eq!(resolve((640.0, 360.0), out(), DEFAULT_BAND_FRACTION), None);
    }

    #[test]
    fn top_edge_maximizes() {
        assert_eq!(
            zone_for((640.0, 10.0), out(), DEFAULT_BAND_FRACTION),
            Some(SnapZone::Maximize)
        );
    }

    #[test]
    fn side_edges_snap_to_halves() {
        assert_eq!(
            zone_for((5.0, 360.0), out(), DEFAULT_BAND_FRACTION),
            Some(SnapZone::LeftHalf)
        );
        assert_eq!(
            zone_for((1275.0, 360.0), out(), DEFAULT_BAND_FRACTION),
            Some(SnapZone::RightHalf)
        );
    }

    #[test]
    fn corners_snap_to_quarters() {
        assert_eq!(
            zone_for((5.0, 5.0), out(), DEFAULT_BAND_FRACTION),
            Some(SnapZone::TopLeft)
        );
        assert_eq!(
            zone_for((1275.0, 5.0), out(), DEFAULT_BAND_FRACTION),
            Some(SnapZone::TopRight)
        );
        assert_eq!(
            zone_for((5.0, 715.0), out(), DEFAULT_BAND_FRACTION),
            Some(SnapZone::BottomLeft)
        );
        assert_eq!(
            zone_for((1275.0, 715.0), out(), DEFAULT_BAND_FRACTION),
            Some(SnapZone::BottomRight)
        );
    }

    #[test]
    fn bottom_center_does_not_snap() {
        assert_eq!(zone_for((640.0, 715.0), out(), DEFAULT_BAND_FRACTION), None);
    }

    #[test]
    fn band_boundary_is_inclusive() {
        // Exactly `band * width` from the left edge still counts as the edge.
        let x = 1280.0 * DEFAULT_BAND_FRACTION;
        assert_eq!(
            zone_for((x, 360.0), out(), DEFAULT_BAND_FRACTION),
            Some(SnapZone::LeftHalf)
        );
        // Just outside it does not.
        assert_eq!(
            zone_for((x + 1.0, 360.0), out(), DEFAULT_BAND_FRACTION),
            None
        );
    }

    #[test]
    fn geometry_is_correct() {
        assert_eq!(
            geometry_for(SnapZone::Maximize, out()),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 1280.0,
                h: 720.0
            }
        );
        assert_eq!(
            geometry_for(SnapZone::LeftHalf, out()),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 640.0,
                h: 720.0
            }
        );
        assert_eq!(
            geometry_for(SnapZone::RightHalf, out()),
            Rect {
                x: 640.0,
                y: 0.0,
                w: 640.0,
                h: 720.0
            }
        );
        assert_eq!(
            geometry_for(SnapZone::TopLeft, out()),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 640.0,
                h: 360.0
            }
        );
        assert_eq!(
            geometry_for(SnapZone::BottomRight, out()),
            Rect {
                x: 640.0,
                y: 360.0,
                w: 640.0,
                h: 360.0
            }
        );
    }

    #[test]
    fn geometry_respects_output_offset() {
        let shifted = Rect {
            x: 100.0,
            y: 50.0,
            w: 1280.0,
            h: 720.0,
        };
        assert_eq!(
            geometry_for(SnapZone::LeftHalf, shifted),
            Rect {
                x: 100.0,
                y: 50.0,
                w: 640.0,
                h: 720.0
            }
        );
        assert_eq!(
            zone_for((105.0, 400.0), shifted, DEFAULT_BAND_FRACTION),
            Some(SnapZone::LeftHalf)
        );
    }

    #[test]
    fn snap_memory_remembers_first_size() {
        let mut mem = wm_snap_memory();
        assert!(!mem.is_snapped(&1));
        mem.remember(1, (800, 600));
        mem.remember(1, (400, 300)); // re-snap must not overwrite
        assert!(mem.is_snapped(&1));
        assert_eq!(mem.restore(&1), Some((800, 600)));
        assert!(!mem.is_snapped(&1));
        assert_eq!(mem.restore(&1), None);
    }

    #[test]
    fn snap_memory_forget_keeps_nothing() {
        let mut mem = wm_snap_memory();
        mem.remember(2, (640, 480));
        mem.forget(&2);
        assert!(!mem.is_snapped(&2));
    }

    fn wm_snap_memory() -> SnapMemory<u32> {
        SnapMemory::<u32>::new()
    }

    #[test]
    fn region_membership_uses_centre() {
        let r = Rect {
            x: 100.0,
            y: 0.0,
            w: 200.0,
            h: 100.0,
        }; // centre x = 200
        assert!(in_region(r, 0.0, 500.0));
        assert!(in_region(r, 150.0, 500.0));
        assert!(!in_region(r, 500.0, 500.0));
        // A window straddling the boundary belongs to the region holding its
        // centre.
        let straddle = Rect {
            x: 450.0,
            y: 0.0,
            w: 200.0,
            h: 100.0,
        }; // centre = 550
        assert!(in_region(straddle, 500.0, 500.0));
        assert!(!in_region(straddle, 0.0, 500.0));
    }
}
