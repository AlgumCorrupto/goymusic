use std::time::Duration;

pub const TICK: Duration = Duration::from_micros(16_667); // ~60 Hz
pub const MAX_CATCH_UP: Duration = Duration::from_millis(100); // ~6 ticks