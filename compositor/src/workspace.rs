//! Fixed workspaces (AGENTS section 6.4).
//!
//! ArchLume has exactly ten workspaces. Rather than hiding and re-mapping
//! windows on every switch, all windows stay mapped in one `Space`; each
//! workspace is a horizontal region of that space, and switching moves the
//! output's view to the target region's offset. This keeps window bookkeeping
//! (frame callbacks, commit handling, popups) unchanged and makes switching
//! cheap.
//!
//! The arithmetic and key mapping are pure and unit-tested here; the compositor
//! wires them to the output and keyboard.

/// Number of workspaces (AGENTS section 6.4: exactly ten).
pub const COUNT: usize = 10;

/// X11 keysyms for the top-row digits.
const XK_0: u32 = 0x30;
const XK_1: u32 = 0x31;
const XK_9: u32 = 0x39;

/// Map a digit keysym to a zero-based workspace index following the ArchLume
/// convention: `1`..`9` select workspaces 1..9 and `0` selects workspace 10.
pub fn index_for_digit(keysym: u32) -> Option<usize> {
    match keysym {
        XK_1..=XK_9 => Some((keysym - XK_1) as usize),
        XK_0 => Some(COUNT - 1),
        _ => None,
    }
}

/// A workspace action triggered by a key binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// `Super + digit`: switch to the workspace.
    Switch(usize),
    /// `Shift + Super + digit`: move the focused window there and follow it.
    MoveTo(usize),
    /// `Super + d`: toggle the application launcher.
    ToggleLauncher,
}

/// Resolve a key event into a workspace action (AGENTS section 6.4).
///
/// Only `Super` (logo) combinations on a digit keysym, while the key is being
/// pressed, produce an action; everything else is left for the client.
pub fn action_for(keysym: u32, pressed: bool, logo: bool, shift: bool) -> Option<Action> {
    if !pressed || !logo {
        return None;
    }
    // `Super + d` toggles the launcher (XKB_KEY_d == 0x64).
    if keysym == 0x64 && !shift {
        return Some(Action::ToggleLauncher);
    }
    let index = index_for_digit(keysym)?;
    Some(if shift {
        Action::MoveTo(index)
    } else {
        Action::Switch(index)
    })
}

/// Current workspace and the horizontal stride between workspaces (the output
/// width).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Workspaces {
    current: usize,
    stride: i32,
}

impl Workspaces {
    pub fn new(stride: i32) -> Self {
        Self { current: 0, stride }
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn stride(&self) -> i32 {
        self.stride
    }

    /// Update the stride (e.g. after the output size changes).
    pub fn set_stride(&mut self, stride: i32) {
        self.stride = stride;
    }

    /// Left offset of a workspace region in the space.
    pub fn offset_of(&self, index: usize) -> i32 {
        index as i32 * self.stride
    }

    pub fn current_offset(&self) -> i32 {
        self.offset_of(self.current)
    }

    /// Switch to `index` (clamped to the valid range). Returns whether the
    /// active workspace changed.
    pub fn switch_to(&mut self, index: usize) -> bool {
        let index = index.min(COUNT - 1);
        if index == self.current {
            false
        } else {
            self.current = index;
            true
        }
    }

    /// Switch by a relative delta, wrapping around the workspace list.
    pub fn switch_by(&mut self, delta: i32) -> bool {
        let n = COUNT as i32;
        let index = ((self.current as i32 + delta).rem_euclid(n)) as usize;
        self.switch_to(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_map_to_expected_workspaces() {
        assert_eq!(index_for_digit(XK_1), Some(0));
        assert_eq!(index_for_digit(0x35), Some(4)); // '5'
        assert_eq!(index_for_digit(XK_9), Some(8));
        assert_eq!(index_for_digit(XK_0), Some(9));
        assert_eq!(index_for_digit(0x61), None); // 'a'
    }

    #[test]
    fn offsets_follow_stride() {
        let ws = Workspaces::new(1366);
        assert_eq!(ws.offset_of(0), 0);
        assert_eq!(ws.offset_of(1), 1366);
        assert_eq!(ws.offset_of(9), 9 * 1366);
        assert_eq!(ws.current_offset(), 0);
    }

    #[test]
    fn switch_to_clamps_and_reports_change() {
        let mut ws = Workspaces::new(800);
        assert!(ws.switch_to(3));
        assert_eq!(ws.current(), 3);
        assert_eq!(ws.current_offset(), 2400);
        assert!(!ws.switch_to(3));
        assert!(ws.switch_to(999));
        assert_eq!(ws.current(), COUNT - 1);
    }

    #[test]
    fn switch_by_wraps() {
        let mut ws = Workspaces::new(100);
        ws.switch_to(0);
        assert!(ws.switch_by(-1));
        assert_eq!(ws.current(), COUNT - 1);
        assert!(ws.switch_by(1));
        assert_eq!(ws.current(), 0);
        assert!(!ws.switch_by(0));
    }

    #[test]
    fn action_requires_super_and_press() {
        assert_eq!(action_for(XK_1, true, true, false), Some(Action::Switch(0)));
        assert_eq!(action_for(XK_0, true, true, true), Some(Action::MoveTo(9)));
        assert_eq!(action_for(0x35, true, false, false), None); // no Super
        assert_eq!(action_for(0x35, false, true, false), None); // released
        assert_eq!(action_for(0x61, true, true, false), None); // not a digit
    }

    #[test]
    fn super_d_toggles_launcher() {
        assert_eq!(
            action_for(0x64, true, true, false),
            Some(Action::ToggleLauncher)
        );
        assert_eq!(action_for(0x64, true, true, true), None); // Shift+D is not bound
        assert_eq!(action_for(0x64, true, false, false), None); // no Super
    }

    #[test]
    fn stride_update_changes_offsets() {
        let mut ws = Workspaces::new(100);
        ws.switch_to(2);
        assert_eq!(ws.offset_of(2), 200);
        ws.set_stride(1920);
        assert_eq!(ws.offset_of(2), 3840);
    }
}
