use std::time::Duration;

use fsrs::{FSRSItem, FSRSReview, FSRS};
use serde::{Deserialize, Serialize};

use super::Shared;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[repr(u32)]
pub enum Quality {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Level {
    pub last_quality: Quality,
    pub last_review: chrono::DateTime<chrono::Local>,
    pub history: FSRSItem,
}
pub struct RepetitionContext {
    pub quality: Quality,
    pub review_time: chrono::DateTime<chrono::Local>,
}

pub fn fsrs(shared: &Shared) -> FSRS {
    FSRS::new(Some(&shared.weights)).unwrap()
}

impl Level {
    pub fn new(quality: Quality, review_time: chrono::DateTime<chrono::Local>) -> Self {
        Self {
            last_quality: quality,
            last_review: review_time,
            history: FSRSItem {
                reviews: vec![FSRSReview {
                    rating: quality as u32,
                    delta_t: 0,
                }],
            },
        }
    }
    pub fn memory_state(&self, fsrs: &FSRS) -> fsrs::MemoryState {
        fsrs.memory_state(self.history.clone(), None).unwrap()
    }
}
impl ssr_core::task::level::TaskLevel<'_> for Level {
    type Context = RepetitionContext;

    type SharedState = super::Shared;

    fn update(&mut self, _shared_state: &mut Self::SharedState, context: Self::Context) {
        self.history.reviews.push(FSRSReview {
            rating: context.quality as u32,
            delta_t: sleeps_between(self.last_review, context.review_time)
                .try_into()
                .unwrap(),
        });
        self.last_quality = context.quality;
        self.last_review = context.review_time;
    }

    fn next_repetition(
        &self,
        shared_state: &Self::SharedState,
        retrievability_goal: f64,
    ) -> std::time::SystemTime {
        let fsrs = fsrs(shared_state);

        let interval_in_days = fsrs.next_interval(
            Some(self.memory_state(&fsrs).stability),
            retrievability_goal as f32,
            self.last_quality as u32,
        );
        const SECS_IN_DAY: f32 = 24. * 60. * 60.;
        let interval = Duration::from_secs_f32(interval_in_days * SECS_IN_DAY);

        std::time::SystemTime::from(self.last_review) + interval
    }
}

pub fn sleeps_between(first: impl chrono::Datelike, second: impl chrono::Datelike) -> i32 {
    second.num_days_from_ce() - first.num_days_from_ce()
}
