pub struct ClkMonitor {
    last_level: bool,
    edge_count: u32,
    window_start_us: u64,
    detected_windows: u8,
    detected: bool,
}

impl ClkMonitor {
    pub const fn new() -> Self {
        Self {
            last_level: false,
            edge_count: 0,
            window_start_us: 0,
            detected_windows: 0,
            detected: false,
        }
    }

    pub fn sample(&mut self, now_us: u64, level: bool) {
        if level != self.last_level {
            self.edge_count = self.edge_count.saturating_add(1);
            self.last_level = level;
        }

        if now_us.saturating_sub(self.window_start_us) >= 1_000 {
            // Require sustained edge activity across multiple 1ms windows
            // to avoid false positives from floating/noisy inputs.
            if self.edge_count >= 8 {
                self.detected_windows = self.detected_windows.saturating_add(1);
            } else {
                self.detected_windows = 0;
            }
            self.detected = self.detected_windows >= 3;
            self.edge_count = 0;
            self.window_start_us = now_us;
        }
    }

    pub fn clock_detected(&self) -> bool {
        self.detected
    }
}
