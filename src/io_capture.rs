#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IoSample {
    High,
    Low,
}

pub struct IoCapture {
    enabled: bool,
}

impl IoCapture {
    pub const fn new() -> Self {
        Self { enabled: false }
    }

    pub fn start_wait_for_atr(&mut self) {
        self.enabled = true;
    }

    pub fn stop(&mut self) {
        self.enabled = false;
    }

    pub fn feed_sample(&mut self, _timestamp_us: u64, _sample: IoSample) {
        if !self.enabled {
            return;
        }

        // TODO(stage 5): Replace this with PIO + DMA backed bit capture.
        // TODO(stage 5): Add byte reconstruction with ETU-aware timing.
    }
}
