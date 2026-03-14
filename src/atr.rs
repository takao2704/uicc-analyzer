use heapless::String;

use crate::rst_monitor::RstLevel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtrState {
    Idle,
    ResetAsserted,
    WaitForClock,
    WaitForAtr,
    // Planned extension states for full ISO 7816-3 ATR parsing:
    // AtrReceiving,
    // AtrComplete,
    // Timeout,
}

pub struct AtrMachine {
    state: AtrState,
}

impl AtrMachine {
    pub const fn new() -> Self {
        Self {
            state: AtrState::Idle,
        }
    }

    pub fn on_rst_transition(&mut self, level: RstLevel) -> Option<AtrState> {
        self.state = match level {
            RstLevel::Low => AtrState::ResetAsserted,
            RstLevel::High => AtrState::WaitForClock,
        };
        Some(self.state)
    }

    pub fn on_clk_activity(&mut self, detected: bool) -> Option<AtrState> {
        if self.state == AtrState::WaitForClock && detected {
            self.state = AtrState::WaitForAtr;
            return Some(self.state);
        }
        None
    }

    pub fn format_atr_prefix(bytes: &[u8]) -> String<128> {
        let mut out: String<128> = String::new();
        for (i, b) in bytes.iter().enumerate() {
            if i > 0 {
                out.push(' ').ok();
            }
            let _ = core::fmt::write(&mut out, format_args!("{:02X}", b));
        }
        out
    }
}
