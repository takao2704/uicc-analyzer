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
}

impl RstMonitor {
    pub const fn new() -> Self {
        Self { last_level: None }
    }

    pub fn update(&mut self, level_high: bool) -> Option<RstTransition> {
        let current = if level_high {
            RstLevel::High
        } else {
            RstLevel::Low
        };

        if self.last_level != Some(current) {
            self.last_level = Some(current);
            return Some(RstTransition { level: current });
        }

        None
    }
}
