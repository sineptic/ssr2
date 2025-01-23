use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use ssr_core::task::level::TaskLevel;

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub(crate) group: u32,
    pub(crate) last_repetition_time: SystemTime,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            group: 1,
            last_repetition_time: SystemTime::UNIX_EPOCH,
        }
    }
}

impl TaskLevel<'_> for Level {
    type Context = (SystemTime, bool);
    type SharedState = ();
    fn update(&mut self, _: &mut (), (now, is_correct): Self::Context) {
        self.last_repetition_time = now;
        if is_correct {
            self.group = (self.group + 1).clamp(1, 4);
        } else {
            self.group = (self.group - 1).clamp(1, 4);
        }
    }

    fn next_repetition(&self, _: &(), _: f64) -> SystemTime {
        const DAY: Duration = Duration::new(60 * 60 * 24, 0);
        self.last_repetition_time + DAY * self.group
    }
}
