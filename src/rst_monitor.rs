#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RstLevel {
    Low,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RstTransition {
    pub level: RstLevel,
}

pub struct RstMonitor {
    last_level: Option<RstLevel>,
    last_change_us: Option<u64>,
}

impl RstMonitor {
    const DEBOUNCE_US: u64 = 20;

    pub const fn new() -> Self {
        Self {
            last_level: None,
            last_change_us: None,
        }
    }

    pub fn update(&mut self, now_us: u64, level_high: bool) -> Option<RstTransition> {
        let current = if level_high {
            RstLevel::High
        } else {
            RstLevel::Low
        };

        if self.last_level != Some(current) {
            if let Some(last_change) = self.last_change_us {
                if now_us.saturating_sub(last_change) < Self::DEBOUNCE_US {
                    return None;
                }
            }
            self.last_level = Some(current);
            self.last_change_us = Some(now_us);
            return Some(RstTransition { level: current });
        }

        None
    }
}
