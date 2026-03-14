pub struct ClkMonitor {
    last_level: bool,
    edge_count: u32,
    window_start_us: u64,
    detected: bool,
}

impl ClkMonitor {
    pub const fn new() -> Self {
        Self {
            last_level: false,
            edge_count: 0,
            window_start_us: 0,
            detected: false,
        }
    }

    pub fn sample(&mut self, now_us: u64, level: bool) {
        if level != self.last_level {
            self.edge_count = self.edge_count.saturating_add(1);
            self.last_level = level;
        }

        if now_us.saturating_sub(self.window_start_us) >= 1_000 {
            self.detected = self.edge_count >= 4;
            self.edge_count = 0;
            self.window_start_us = now_us;
        }
    }

    pub fn clock_detected(&self) -> bool {
        self.detected
    }
}
