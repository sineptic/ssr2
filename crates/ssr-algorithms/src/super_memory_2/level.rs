use std::{
    cmp::max_by,
    fmt::Display,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};
use ssr_core::task::level::TaskLevel;

#[derive(Serialize, Deserialize)]
pub struct Level {
    e_factor: f64,
    strike: u32,
    interval: Duration,
    last_repetition: SystemTime,
    repetition_required: bool,
}
impl Default for Level {
    fn default() -> Self {
        Self {
            e_factor: 2.5,
            strike: 1,
            interval: Default::default(),
            last_repetition: SystemTime::now(),
            repetition_required: false,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Quality {
    CompleteBlackout = 0,
    IncorrectResponseButCorrectRemembered = 1,
    IncorrectResponseAndSeemedEasyToRecall = 2,
    CorrectResponseRecalledWithSeriousDifficulty = 3,
    CorrectResponseAfterHesitation = 4,
    PerfectResponse = 5,
}
impl Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            match self {
                Quality::CompleteBlackout => "complete blackout",
                Quality::IncorrectResponseButCorrectRemembered => "correct remembered",
                Quality::IncorrectResponseAndSeemedEasyToRecall => "seem easy to recall",
                Quality::CorrectResponseRecalledWithSeriousDifficulty => "serious difficulty",
                Quality::CorrectResponseAfterHesitation => "after hesitation",
                Quality::PerfectResponse => "perfect",
            }
        )
    }
}

impl TaskLevel<'_> for Level {
    type SharedState = ();
    type Context = (SystemTime, Quality);
    fn update(&mut self, _: &mut (), (now, quality): Self::Context) {
        self.last_repetition = now;
        const SECS_IN_DAY: u64 = 60 * 60 * 24;

        let q = quality as u8;

        if q >= 3 {
            self.interval = match self.strike {
                0 => Duration::from_secs(SECS_IN_DAY),
                1 => Duration::from_secs(6 * SECS_IN_DAY),
                _ => Duration::from_secs_f64(self.interval.as_secs_f64() * self.e_factor),
            };
            self.strike += 1;
            self.repetition_required = false;
        } else {
            self.strike = 0;
            self.interval = Duration::from_secs(SECS_IN_DAY);
            self.repetition_required = true;
        }

        self.e_factor = max_by(
            self.e_factor + 0.1 - (5. - q as f64) * (0.08 + (5. - q as f64) * 0.02),
            1.3,
            |a, b| a.partial_cmp(b).unwrap(),
        );
    }

    fn next_repetition(&self, _: &(), _retrievability_goal: f64) -> SystemTime {
        if self.repetition_required {
            SystemTime::now()
        } else {
            self.last_repetition + self.interval
        }
    }
}
