use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local};
use fsrs::{FSRS, FSRSItem, FSRSReview};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[repr(u32)]
pub enum Quality {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}
pub struct RepetitionContext {
    pub quality: Quality,
    pub review_time: chrono::DateTime<chrono::Local>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub enum Level {
    #[allow(private_interfaces)]
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

    pub fn next_repetition(&self, fsrs: &FSRS, retrievability_goal: f64) -> SystemTime {
        match self {
            Level::Started(level) => level.next_repetition(fsrs, retrievability_goal),
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

    pub fn history(&self) -> Option<FSRSItem> {
        match self {
            Level::Started(level) => {
                if level.history.reviews.iter().any(|r| r.delta_t != 0) {
                    Some(level.history.clone())
                } else {
                    None
                }
            }
            Level::NotStarted => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StartedLevel {
    last_quality: Quality,
    last_review: chrono::DateTime<chrono::Local>,
    history: FSRSItem,
}
impl StartedLevel {
    fn new(quality: Quality, review_time: chrono::DateTime<chrono::Local>) -> Self {
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
    fn memory_state(&self, fsrs: &FSRS) -> fsrs::MemoryState {
        fsrs.memory_state(self.history.clone(), None).unwrap()
    }
    fn add_repetition(&mut self, repetition: RepetitionContext) {
        self.history.reviews.push(FSRSReview {
            rating: repetition.quality as u32,
            delta_t: sleeps_between(&self.last_review, &repetition.review_time)
                .try_into()
                .unwrap(),
        });
        self.last_quality = repetition.quality;
        self.last_review = repetition.review_time;
    }
    fn next_repetition(&self, fsrs: &FSRS, retrievability_goal: f64) -> SystemTime {
        let interval_in_days = fsrs.next_interval(
            Some(self.memory_state(fsrs).stability),
            retrievability_goal as f32,
            self.last_quality as u32,
        );
        const SECS_IN_DAY: f32 = 24. * 60. * 60.;
        let interval = Duration::from_secs_f32(interval_in_days * SECS_IN_DAY);

        SystemTime::from(self.last_review) + interval
    }
    fn next_states(
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

fn sleeps_between(first: &impl chrono::Datelike, second: &impl chrono::Datelike) -> i32 {
    second.num_days_from_ce() - first.num_days_from_ce()
}
