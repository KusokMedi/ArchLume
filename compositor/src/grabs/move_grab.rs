use crate::{wm, ArchLume};
use smithay::{
    desktop::Window,
    input::pointer::{
        AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent,
        GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent,
        GestureSwipeEndEvent, GestureSwipeUpdateEvent, GrabStartData as PointerGrabStartData,
        MotionEvent, PointerGrab, PointerInnerHandle, RelativeMotionEvent,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
};

pub struct MoveSurfaceGrab {
    pub start_data: PointerGrabStartData<ArchLume>,
    pub window: Window,
    pub initial_window_location: Point<i32, Logical>,
    /// Latest pointer location in space coordinates, used to decide the snap
    /// zone when the drag ends.
    pub current_location: Point<f64, Logical>,
}

impl PointerGrab<ArchLume> for MoveSurfaceGrab {
    fn motion(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        _focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &MotionEvent,
    ) {
        // While the grab is active, no client has pointer focus.
        handle.motion(data, None, event);

        self.current_location = event.location;

        let delta = event.location - self.start_data.location;
        let new_location = self.initial_window_location.to_f64() + delta;
        data.space
            .map_element(self.window.clone(), new_location.to_i32_round(), true);

        // Show the snap preview for the zone the pointer is currently over.
        data.set_snap_preview(data.snap_zone_at((event.location.x, event.location.y)));
    }

    fn relative_motion(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &RelativeMotionEvent,
    ) {
        handle.relative_motion(data, focus, event);
    }

    fn button(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        event: &ButtonEvent,
    ) {
        handle.button(data, event);

        // BTN_LEFT as defined in linux/input-event-codes.h.
        const BTN_LEFT: u32 = 0x110;

        if !handle.current_pressed().contains(&BTN_LEFT) {
            // No more buttons pressed: snap the window if the drop landed in an
            // edge zone, then release the grab.
            let output = data
                .space
                .output_under(self.current_location)
                .next()
                .cloned();
            if let Some(output) = output {
                if let Some(geo) = data.space.output_geometry(&output) {
                    let output_rect = wm::Rect {
                        x: geo.loc.x as f64,
                        y: geo.loc.y as f64,
                        w: geo.size.w as f64,
                        h: geo.size.h as f64,
                    };
                    let target = wm::resolve(
                        (self.current_location.x, self.current_location.y),
                        output_rect,
                        wm::DEFAULT_BAND_FRACTION,
                    );
                    match target {
                        Some((zone, geometry)) => data.snap_window(&self.window, zone, geometry),
                        // Dropped in the centre: restore the floating size if the
                        // window was snapped (un-snap).
                        None => data.un_snap_window(&self.window),
                    }
                }
            }
            // The preview only reflects an in-progress drag.
            data.set_snap_preview(None);
            handle.unset_grab(self, data, event.serial, event.time, true);
        }
    }

    fn axis(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        details: AxisFrame,
    ) {
        handle.axis(data, details)
    }

    fn frame(&mut self, data: &mut ArchLume, handle: &mut PointerInnerHandle<'_, ArchLume>) {
        handle.frame(data);
    }

    fn gesture_swipe_begin(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        event: &GestureSwipeBeginEvent,
    ) {
        handle.gesture_swipe_begin(data, event)
    }

    fn gesture_swipe_update(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        event: &GestureSwipeUpdateEvent,
    ) {
        handle.gesture_swipe_update(data, event)
    }

    fn gesture_swipe_end(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        event: &GestureSwipeEndEvent,
    ) {
        handle.gesture_swipe_end(data, event)
    }

    fn gesture_pinch_begin(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        event: &GesturePinchBeginEvent,
    ) {
        handle.gesture_pinch_begin(data, event)
    }

    fn gesture_pinch_update(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        event: &GesturePinchUpdateEvent,
    ) {
        handle.gesture_pinch_update(data, event)
    }

    fn gesture_pinch_end(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        event: &GesturePinchEndEvent,
    ) {
        handle.gesture_pinch_end(data, event)
    }

    fn gesture_hold_begin(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        event: &GestureHoldBeginEvent,
    ) {
        handle.gesture_hold_begin(data, event)
    }

    fn gesture_hold_end(
        &mut self,
        data: &mut ArchLume,
        handle: &mut PointerInnerHandle<'_, ArchLume>,
        event: &GestureHoldEndEvent,
    ) {
        handle.gesture_hold_end(data, event)
    }

    fn start_data(&self) -> &PointerGrabStartData<ArchLume> {
        &self.start_data
    }

    fn unset(&mut self, data: &mut ArchLume) {
        // Never leave a preview behind if the grab ends outside the button path.
        data.set_snap_preview(None);
    }
}
