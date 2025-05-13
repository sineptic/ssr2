use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local};
use fsrs::{FSRS, FSRSItem, FSRSReview};
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

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub enum Level {
    Started(StartedLevel),
    #[default]
    NotStarted,
}
impl Level {
    pub fn next_states(
        &self,
        fsrs: &FSRS,
        retrievability_goal: f32,
        now: DateTime<Local>,
    ) -> fsrs::NextStates {
        match self {
            Level::Started(level) => level.next_states(fsrs, retrievability_goal, now),
            Level::NotStarted => fsrs.next_states(None, retrievability_goal, 0).unwrap(),
        }
    }

    pub fn get_next_repetition(&self, fsrs: &FSRS, retrievability_goal: f64) -> SystemTime {
        match self {
            Level::Started(level) => level.get_next_repetition(fsrs, retrievability_goal),
            Level::NotStarted => SystemTime::UNIX_EPOCH,
        }
    }

    pub fn add_repetition(&mut self, repetition: RepetitionContext) {
        match self {
            Level::Started(level) => {
                level.add_repetition(repetition);
            }
            Level::NotStarted => {
                *self = Level::Started(StartedLevel::new(
                    repetition.quality,
                    repetition.review_time,
                ));
            }
        }
    }

    pub fn as_started(&self) -> Option<&StartedLevel> {
        if let Self::Started(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StartedLevel {
    pub(crate) last_quality: Quality,
    pub(crate) last_review: chrono::DateTime<chrono::Local>,
    pub(crate) history: FSRSItem,
}
pub struct RepetitionContext {
    pub quality: Quality,
    pub review_time: chrono::DateTime<chrono::Local>,
}

pub(crate) fn fsrs(shared: &Shared) -> FSRS {
    FSRS::new(Some(&shared.weights)).unwrap()
}

impl StartedLevel {
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
    pub fn add_repetition(&mut self, repetition: RepetitionContext) {
        self.history.reviews.push(FSRSReview {
            rating: repetition.quality as u32,
            delta_t: sleeps_between(&self.last_review, &repetition.review_time)
                .try_into()
                .unwrap(),
        });
        self.last_quality = repetition.quality;
        self.last_review = repetition.review_time;
    }
    pub fn get_next_repetition(&self, fsrs: &FSRS, retrievability_goal: f64) -> SystemTime {
        let interval_in_days = fsrs.next_interval(
            Some(self.memory_state(fsrs).stability),
            retrievability_goal as f32,
            self.last_quality as u32,
        );
        const SECS_IN_DAY: f32 = 24. * 60. * 60.;
        let interval = Duration::from_secs_f32(interval_in_days * SECS_IN_DAY);

        SystemTime::from(self.last_review) + interval
    }
    pub fn next_states(
        &self,
        fsrs: &FSRS,
        retrievability_goal: f32,
        now: DateTime<Local>,
    ) -> fsrs::NextStates {
        fsrs.next_states(
            Some(self.memory_state(fsrs)),
            retrievability_goal,
            sleeps_between(&self.last_review, &now).try_into().unwrap(),
        )
        .unwrap()
    }
}
impl ssr_core::task::level::TaskLevel<'_> for StartedLevel {
    type Context = RepetitionContext;

    type SharedState = super::Shared;

    fn update(&mut self, _shared_state: &mut Self::SharedState, repetition_context: Self::Context) {
        self.add_repetition(repetition_context);
    }

    fn next_repetition(
        &self,
        shared_state: &Self::SharedState,
        retrievability_goal: f64,
    ) -> std::time::SystemTime {
        let fsrs = fsrs(shared_state);
        self.get_next_repetition(&fsrs, retrievability_goal)
    }
}

fn sleeps_between(first: &impl chrono::Datelike, second: &impl chrono::Datelike) -> i32 {
    second.num_days_from_ce() - first.num_days_from_ce()
}
