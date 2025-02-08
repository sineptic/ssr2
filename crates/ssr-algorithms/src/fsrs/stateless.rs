use std::time::SystemTime;

use s_text_input_f as stif;
use serde::{Deserialize, Serialize};
use ssr_core::{BlocksDatabaseId, TaskDatabase, task::level::TaskLevel as _};

use super::{
    Shared,
    level::{self, Level, Quality, RepetitionContext},
};

#[derive(Clone)]
struct FeedbackParams {
    user_answer: Vec<Vec<String>>,
    blocks: stif::Blocks,
    correct_answer: stif::Response,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatelessTask {
    level: Option<Level>,
    db_task_id: BlocksDatabaseId,
}

impl ssr_core::task::StatelessTask for StatelessTask {
    type SharedState = Shared;

    fn new(id: BlocksDatabaseId) -> Self {
        Self {
            level: None,
            db_task_id: id,
        }
    }

    fn next_repetition(
        &self,
        shared_state: &Self::SharedState,
        retrievability_goal: f64,
    ) -> SystemTime {
        if let Some(ref level) = self.level {
            level.next_repetition(shared_state, retrievability_goal)
        } else {
            SystemTime::UNIX_EPOCH
        }
    }

    fn complete(
        &mut self,
        shared_state: &mut Self::SharedState,
        db: &impl TaskDatabase,
        desired_retention: f64,
        interaction: &mut impl FnMut(stif::Blocks) -> std::io::Result<stif::Response>,
    ) -> std::io::Result<()> {
        let review_time = chrono::Local::now();
        let (blocks, correct_answer, other_answers) = db
            .get_blocks(self.db_task_id)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Task not found"))?;

        let user_answer = interaction(blocks.clone())?;

        let feedback_params = FeedbackParams {
            user_answer,
            blocks,
            correct_answer,
        };

        let quality = self.complete_inner(
            feedback_params,
            &other_answers,
            shared_state,
            desired_retention,
            interaction,
        )?;

        if let Some(ref mut level) = self.level {
            level.update(shared_state, RepetitionContext {
                quality,
                review_time,
            });
        } else {
            self.level = Some(Level::new(quality, review_time));
        }
        Ok(())
    }
}

impl StatelessTask {
    fn complete_inner(
        &mut self,
        feedback_params: FeedbackParams,
        other_answers: &[stif::Response],
        shared_state: &Shared,
        retrievability_goal: f64,
        interaction: &mut impl FnMut(stif::Blocks) -> std::io::Result<Vec<Vec<String>>>,
    ) -> std::io::Result<Quality> {
        let next_states = self.next_states(shared_state, retrievability_goal);
        Ok(
            match self.check_correctness(
                &feedback_params.user_answer,
                &feedback_params.correct_answer,
                other_answers,
            ) {
                true => self.feedback_correct(feedback_params, next_states, interaction)?,
                false => self.feedback_wrong(feedback_params, next_states, interaction)?,
            },
        )
    }

    fn check_correctness(
        &self,
        user_answer: &Vec<Vec<String>>,
        correct_answer: &stif::Response,
        other_answers: &[stif::Response],
    ) -> bool {
        if stif::eq_response(correct_answer, user_answer, true, false) {
            return true;
        }
        other_answers
            .iter()
            .any(|ans| stif::eq_response(ans, user_answer, true, false))
    }

    fn feedback_correct(
        &self,
        feedback_params: FeedbackParams,
        next_states: fsrs::NextStates,
        interaction: &mut impl FnMut(stif::Blocks) -> std::io::Result<Vec<Vec<String>>>,
    ) -> std::io::Result<Quality> {
        let qualities = vec![Quality::Hard, Quality::Good, Quality::Easy];
        let qualities_strings = vec![
            format!("Hard {}d", next_states.hard.interval),
            format!("Good {}d", next_states.good.interval),
            format!("Easy {}d", next_states.easy.interval),
        ];
        let directive = "Correct! Choose difficulty:".to_string();
        self.get_feedback(
            feedback_params,
            directive,
            qualities_strings,
            interaction,
            qualities,
        )
    }

    fn feedback_wrong(
        &self,
        feedback_params: FeedbackParams,
        next_states: fsrs::NextStates,
        interaction: &mut impl FnMut(stif::Blocks) -> std::io::Result<Vec<Vec<String>>>,
    ) -> std::io::Result<Quality> {
        let directive = format!(
            "Wrong. Next review in {}h",
            next_states.again.interval * 24.
        );
        self.show_feedback(feedback_params, directive, interaction)?;
        Ok(Quality::Again)
    }

    fn get_feedback<T: Copy>(
        &self,
        feedback_params: FeedbackParams,
        directive: String,
        qualities_strings: Vec<String>,
        interaction: &mut impl FnMut(Vec<stif::Block>) -> Result<Vec<Vec<String>>, std::io::Error>,
        qualities: Vec<T>,
    ) -> Result<T, std::io::Error> {
        let feedback = self.gen_feedback_form(feedback_params, directive, Some(qualities_strings));
        let user_feedback = interaction(feedback)?;
        let i = stif::response_as_one_of(user_feedback.last().unwrap().to_owned())
            .expect("user feedback should be a OneOf response")
            .expect("user feedback index should be a valid integer since feedback form contains only numbers");
        let quality = qualities[i];
        Ok(quality)
    }

    fn show_feedback(
        &self,
        feedback_params: FeedbackParams,
        directive: String,
        interaction: &mut impl FnMut(Vec<stif::Block>) -> Result<Vec<Vec<String>>, std::io::Error>,
    ) -> std::io::Result<()> {
        let feedback = self.gen_feedback_form(feedback_params, directive, None);
        interaction(feedback)?;
        Ok(())
    }

    fn gen_feedback_form(
        &self,
        feedback_params: FeedbackParams,
        directive: String,
        qualities_strings: Option<Vec<String>>,
    ) -> Vec<stif::Block> {
        let mut feedback = stif::to_answered(
            feedback_params.blocks.clone(),
            feedback_params.user_answer.clone(),
            feedback_params.correct_answer.clone(),
        )
        .into_iter()
        .map(stif::Block::Answered)
        .collect::<Vec<_>>();

        feedback.push(stif::Block::Paragraph(vec![]));
        feedback.push(stif::Block::Paragraph(vec![stif::ParagraphItem::Text(
            directive,
        )]));

        if let Some(qualities) = qualities_strings {
            feedback.push(stif::Block::OneOf(qualities));
        }

        feedback
    }

    fn next_states(&self, shared: &Shared, retrievability_goal: f64) -> fsrs::NextStates {
        let fsrs = level::fsrs(shared);
        let now = chrono::Local::now();
        fsrs.next_states(
            self.level.as_ref().map(|l| l.memory_state(&fsrs)),
            retrievability_goal as f32,
            level::sleeps_between(self.level.as_ref().map_or(now, |l| l.last_review), now)
                .try_into()
                .unwrap(),
        )
        .unwrap()
    }
}
