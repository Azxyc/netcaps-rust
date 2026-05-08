use std::time::{Duration, Instant};

use crate::ffi;

const CAPS_LOCK_KEY_CODE: u16 = 57;
const CAPS_ON_INTERVAL: Duration = Duration::from_micros(10_500);
const CAPS_OFF_INTERVAL: Duration = Duration::from_micros(3_000);
const CAPS_STATE_REFRESH_INTERVAL: Duration = Duration::from_millis(500);

pub fn is_caps_lock_on() -> bool {
    unsafe {
        ffi::CGEventSourceKeyState(
            ffi::K_CG_EVENT_SOURCE_STATE_COMBINED_SESSION_STATE,
            CAPS_LOCK_KEY_CODE,
        )
    }
}

pub struct BlinkInterval {
    current: Duration,
    last_caps_state: bool,
    last_check: Option<Instant>,
}

impl Default for BlinkInterval {
    fn default() -> Self {
        Self {
            current: CAPS_OFF_INTERVAL,
            last_caps_state: false,
            last_check: None,
        }
    }
}

impl BlinkInterval {
    pub fn current(&self) -> Duration {
        self.current
    }

    pub fn refresh(&mut self) {
        if self
            .last_check
            .is_some_and(|last_check| last_check.elapsed() <= CAPS_STATE_REFRESH_INTERVAL)
        {
            return;
        }

        let current_caps_state = is_caps_lock_on();
        if current_caps_state != self.last_caps_state {
            self.current = if current_caps_state {
                CAPS_ON_INTERVAL
            } else {
                CAPS_OFF_INTERVAL
            };
            self.last_caps_state = current_caps_state;
        }

        self.last_check = Some(Instant::now());
    }
}
