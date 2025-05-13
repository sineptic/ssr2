use std::time::SystemTime;

use s_text_input_f as stif;
use serde::{Deserialize, Serialize};
use ssr_core::BlocksDatabaseId;

use super::{
    Shared,
    level::{self, Level, Quality, RepetitionContext},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatelessTask {
    level: Level,
    task_id: BlocksDatabaseId,
}

impl ssr_core::task::StatelessTask for StatelessTask {
    type SharedState = Shared;

    fn new(id: BlocksDatabaseId) -> Self {
        Self {
            level: Level::default(),
            task_id: id,
        }
    }

    fn next_repetition(
        &self,
        shared_state: &Self::SharedState,
        retrievability_goal: f64,
    ) -> SystemTime {
        self.level
            .get_next_repetition(&level::fsrs(shared_state), retrievability_goal)
    }

    fn complete(
        &mut self,
        is_correct: bool,
        shared_state: &mut Self::SharedState,
        desired_retention: f64,
        interaction: &mut impl FnMut(stif::Blocks) -> std::io::Result<stif::Response>,
    ) -> std::io::Result<()> {
        let review_time = chrono::Local::now();

        let next_states = self.next_states(shared_state, desired_retention as f32);
        let quality = if is_correct {
            self.feedback_correct(&next_states, interaction)?
        } else {
            self.feedback_wrong(&next_states, interaction)?
        };

        self.level.add_repetition(RepetitionContext {
            quality,
            review_time,
        });
        Ok(())
    }

    fn get_id(&self) -> BlocksDatabaseId {
        self.task_id
    }
}

impl StatelessTask {
    fn feedback_correct(
        &self,
        next_states: &fsrs::NextStates,
        interaction: &mut impl FnMut(stif::Blocks) -> std::io::Result<stif::Response>,
    ) -> std::io::Result<Quality> {
        let qualities = [Quality::Hard, Quality::Good, Quality::Easy];
        let user_feedback = interaction(vec![
            stif::Block::Paragraph(vec!["Correct! Choose difficulty:".into()]),
            stif::Block::one_of([
                format!("Hard {}d", next_states.hard.interval),
                format!("Good {}d", next_states.good.interval),
                format!("Easy {}d", next_states.easy.interval),
            ]),
        ])?;
        let i = stif::response_as_one_of(user_feedback.last().unwrap().to_owned())
            .unwrap()
            .unwrap();

        Ok(qualities[i])
    }

    fn feedback_wrong(
        &self,
        next_states: &fsrs::NextStates,
        interaction: &mut impl FnMut(stif::Blocks) -> std::io::Result<stif::Response>,
    ) -> std::io::Result<Quality> {
        interaction(vec![stif::Block::Paragraph(vec![
            format!(
                "Wrong. Next review in {}h",
                next_states.again.interval * 24.
            )
            .into(),
        ])])?;
        Ok(Quality::Again)
    }

    fn next_states(&self, shared: &Shared, desired_retention: f32) -> fsrs::NextStates {
        let fsrs = level::fsrs(shared);
        let now = chrono::Local::now();
        self.level.next_states(&fsrs, desired_retention, now)
    }
}
