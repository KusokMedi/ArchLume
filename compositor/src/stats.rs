//! Lightweight frame-time accounting for the render loop.
//!
//! The compositor records how long each rendered frame took so the shell and
//! diagnostics tools can observe performance (see AGENTS.md section 7). Only a
//! rolling window of recent frames is kept to bound memory use.

use std::time::Duration;

/// Number of recent frame durations retained.
const WINDOW: usize = 120;

/// Rolling frame-time statistics.
#[derive(Debug)]
pub struct FrameStats {
    durations_us: [u32; WINDOW],
    index: usize,
    len: usize,
    frames: u64,
    max_us: u32,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameStats {
    pub const fn new() -> Self {
        Self {
            durations_us: [0; WINDOW],
            index: 0,
            len: 0,
            frames: 0,
            max_us: 0,
        }
    }

    /// Record one rendered frame's duration.
    pub fn record(&mut self, elapsed: Duration) {
        let us = elapsed.as_micros().min(u32::MAX as u128) as u32;
        self.durations_us[self.index] = us;
        self.index = (self.index + 1) % WINDOW;
        if self.len < WINDOW {
            self.len += 1;
        }
        self.frames += 1;
        if us > self.max_us {
            self.max_us = us;
        }
    }

    /// Total number of frames recorded since start or the last reset.
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Mean duration of the frames in the rolling window, in microseconds.
    pub fn mean_us(&self) -> u64 {
        if self.len == 0 {
            return 0;
        }
        let sum: u64 = self.durations_us[..self.len]
            .iter()
            .map(|&d| d as u64)
            .sum();
        sum / self.len as u64
    }

    /// Worst frame duration observed so far, in microseconds.
    pub fn max_us(&self) -> u64 {
        self.max_us as u64
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::FrameStats;
    use std::time::Duration;

    #[test]
    fn empty_stats_are_zero() {
        let stats = FrameStats::new();
        assert_eq!(stats.frames(), 0);
        assert_eq!(stats.mean_us(), 0);
        assert_eq!(stats.max_us(), 0);
    }

    #[test]
    fn records_mean_and_max() {
        let mut stats = FrameStats::new();
        stats.record(Duration::from_micros(1_000));
        stats.record(Duration::from_micros(3_000));
        stats.record(Duration::from_micros(2_000));

        assert_eq!(stats.frames(), 3);
        assert_eq!(stats.mean_us(), 2_000);
        assert_eq!(stats.max_us(), 3_000);
    }

    #[test]
    fn max_is_lifetime_not_window() {
        let mut stats = FrameStats::new();
        stats.record(Duration::from_micros(9_000));
        for _ in 0..(super::WINDOW * 2) {
            stats.record(Duration::from_micros(100));
        }
        // The slow frame has left the rolling window, so the mean is low, but
        // the worst-case value is still reported.
        assert_eq!(stats.mean_us(), 100);
        assert_eq!(stats.max_us(), 9_000);
        assert_eq!(stats.frames() as usize, super::WINDOW * 2 + 1);
    }
}
