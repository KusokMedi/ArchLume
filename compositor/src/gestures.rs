//! Touchpad gesture recognition (AGENTS section 6.6).
//!
//! Gesture events are delivered by the native libinput backend; the winit
//! development backend does not produce them. This module holds the backend-
//! independent, unit-tested part: turning a normalized swipe delta into a
//! direction and mapping (finger count, direction) to an action. The action is
//! then executed through the same workspace code paths as key bindings, so
//! gestures reuse the general action system.

/// Direction of a recognized swipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// An action triggered by a gesture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureAction {
    /// Nothing is bound to this gesture.
    None,
    /// Switch to the previous workspace (wraps around).
    PrevWorkspace,
    /// Switch to the next workspace (wraps around).
    NextWorkspace,
}

/// Fraction of the output the pointer must travel before a swipe counts. This
/// is the "sensitivity" knob referenced by AGENTS section 6.6.
pub const DEFAULT_THRESHOLD: f64 = 0.15;

/// Classify a normalized swipe delta into a direction, or `None` when the
/// movement is below the threshold. The dominant axis wins.
pub fn direction_for(dx: f64, dy: f64, threshold: f64) -> Option<Direction> {
    if dx.abs() < threshold && dy.abs() < threshold {
        return None;
    }
    if dx.abs() >= dy.abs() {
        Some(if dx < 0.0 {
            Direction::Left
        } else {
            Direction::Right
        })
    } else {
        Some(if dy < 0.0 {
            Direction::Up
        } else {
            Direction::Down
        })
    }
}

/// Default gesture bindings. Three-finger horizontal swipes switch workspaces;
/// other combinations are reserved (overview and app menu are not implemented
/// yet) and report [`GestureAction::None`].
pub fn action_for(fingers: u8, direction: Direction) -> GestureAction {
    match (fingers, direction) {
        (3, Direction::Left) => GestureAction::PrevWorkspace,
        (3, Direction::Right) => GestureAction::NextWorkspace,
        _ => GestureAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::{action_for, direction_for, Direction, GestureAction, DEFAULT_THRESHOLD};

    #[test]
    fn below_threshold_is_no_gesture() {
        assert_eq!(direction_for(0.05, 0.05, DEFAULT_THRESHOLD), None);
        assert_eq!(direction_for(-0.14, 0.02, DEFAULT_THRESHOLD), None);
    }

    #[test]
    fn dominant_axis_and_sign_decide_direction() {
        assert_eq!(
            direction_for(-0.5, 0.1, DEFAULT_THRESHOLD),
            Some(Direction::Left)
        );
        assert_eq!(
            direction_for(0.5, 0.1, DEFAULT_THRESHOLD),
            Some(Direction::Right)
        );
        assert_eq!(
            direction_for(0.1, -0.5, DEFAULT_THRESHOLD),
            Some(Direction::Up)
        );
        assert_eq!(
            direction_for(0.1, 0.5, DEFAULT_THRESHOLD),
            Some(Direction::Down)
        );
        // A tie goes to the horizontal axis.
        assert_eq!(
            direction_for(0.3, -0.3, DEFAULT_THRESHOLD),
            Some(Direction::Right)
        );
    }

    #[test]
    fn three_finger_swipes_switch_workspaces() {
        assert_eq!(action_for(3, Direction::Left), GestureAction::PrevWorkspace);
        assert_eq!(
            action_for(3, Direction::Right),
            GestureAction::NextWorkspace
        );
    }

    #[test]
    fn other_gestures_are_reserved() {
        assert_eq!(action_for(4, Direction::Up), GestureAction::None);
        assert_eq!(action_for(3, Direction::Up), GestureAction::None);
        assert_eq!(action_for(2, Direction::Left), GestureAction::None);
    }
}
