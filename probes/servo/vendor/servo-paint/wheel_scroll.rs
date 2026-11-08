/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::{Duration, Instant};

const DURATION: Duration = Duration::from_millis(120);

/// The remaining distance of a discrete wheel input, eased out over a short duration.
pub(crate) struct WheelScroll {
    remaining: [f32; 2],
    last_frame: Instant,
    end: Instant,
}

impl WheelScroll {
    pub(crate) fn new(delta: [f32; 2], now: Instant) -> Self {
        Self {
            remaining: delta,
            last_frame: now,
            end: now + DURATION,
        }
    }

    pub(crate) fn add(&mut self, delta: [f32; 2], now: Instant) {
        for (remaining, delta) in self.remaining.iter_mut().zip(delta) {
            // A reversal should respond immediately instead of finishing the old direction.
            if *remaining * delta < 0.0 {
                *remaining = 0.0;
            }
            *remaining += delta;
        }
        self.last_frame = now;
        self.end = now + DURATION;
    }

    pub(crate) fn advance(&mut self, now: Instant) -> [f32; 2] {
        let before = self
            .end
            .saturating_duration_since(self.last_frame)
            .as_secs_f32();
        let after = self.end.saturating_duration_since(now).as_secs_f32();
        self.last_frame = now;
        let fraction = if before == 0.0 {
            1.0
        } else {
            1.0 - (after / before).powi(3)
        };
        let delta = self.remaining.map(|remaining| remaining * fraction);
        for (remaining, step) in self.remaining.iter_mut().zip(delta) {
            *remaining -= step;
        }
        delta
    }

    pub(crate) fn finished(&self, now: Instant) -> bool {
        now >= self.end
    }

    pub(crate) fn finish(&mut self) -> [f32; 2] {
        std::mem::replace(&mut self.remaining, [0.0; 2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_preserve_distance_and_finish() {
        let start = Instant::now();
        let mut scroll = WheelScroll::new([40.0, -120.0], start);
        assert_eq!(scroll.advance(start), [0.0, 0.0]);
        let mut total = [0.0; 2];
        for ms in [16, 32, 48, 64, 80, 96, 112, 120] {
            let step = scroll.advance(start + Duration::from_millis(ms));
            assert!(step[0] > 0.0 && step[1] < 0.0);
            total[0] += step[0];
            total[1] += step[1];
        }
        assert!((total[0] - 40.0).abs() < 0.001);
        assert!((total[1] + 120.0).abs() < 0.001);
        assert!(scroll.finished(start + DURATION));
        assert_eq!(scroll.finish(), [0.0, 0.0]);
    }

    #[test]
    fn repeated_input_keeps_the_full_distance() {
        let start = Instant::now();
        let mut scroll = WheelScroll::new([0.0, 40.0], start);
        let first = scroll.advance(start + Duration::from_millis(16));
        scroll.add([0.0, 40.0], start + Duration::from_millis(20));
        let last = scroll.advance(start + Duration::from_millis(140));
        assert!((first[1] + last[1] - 80.0).abs() < 0.001);
        assert!(scroll.finished(start + Duration::from_millis(140)));
    }

    #[test]
    fn reversal_discards_only_the_opposite_axis_remainder() {
        let start = Instant::now();
        let mut scroll = WheelScroll::new([20.0, 40.0], start);
        let first = scroll.advance(start + Duration::from_millis(16));
        scroll.add([0.0, -40.0], start + Duration::from_millis(20));
        let last = scroll.advance(start + Duration::from_millis(140));
        assert!((last[1] + 40.0).abs() < 0.001);
        assert!((first[0] + last[0] - 20.0).abs() < 0.001);
    }

    #[test]
    fn delayed_frame_consumes_the_remainder_once() {
        let start = Instant::now();
        let mut scroll = WheelScroll::new([0.0, 40.0], start);
        assert_eq!(scroll.advance(start + Duration::from_secs(1)), [0.0, 40.0]);
        assert_eq!(scroll.advance(start + Duration::from_secs(2)), [0.0, 0.0]);
        assert!(scroll.finished(start + Duration::from_secs(2)));
    }
}
