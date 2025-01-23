use std::time::SystemTime;

use level::{Level, Quality};
use s_text_input_f::{BlocksWithAnswer, ParagraphItem};
use serde::{Deserialize, Serialize};
use ssr_core::task::{level::TaskLevel, Task};

mod level;

#[derive(Serialize, Deserialize)]
pub struct WriteAnswer {
    level: Level,
    input_blocks: s_text_input_f::Blocks,
    correct_answer: s_text_input_f::Response,
}

impl WriteAnswer {
    pub fn new(
        input_blocks: s_text_input_f::Blocks,
        correct_answer: s_text_input_f::Response,
    ) -> Self {
        Self {
            level: Default::default(),
            input_blocks,
            correct_answer,
        }
    }

    fn gen_feedback_form(
        &mut self,
        user_answer: Vec<Vec<String>>,
        directive: String,
        qualities_strings: Vec<String>,
    ) -> Vec<s_text_input_f::Block> {
        let mut feedback = s_text_input_f::to_answered(
            self.input_blocks.clone(),
            user_answer,
            self.correct_answer.clone(),
        )
        .into_iter()
        .map(s_text_input_f::Block::Answered)
        .collect::<Vec<_>>();
        feedback.push(s_text_input_f::Block::Paragraph(vec![]));
        feedback.push(s_text_input_f::Block::Paragraph(vec![ParagraphItem::Text(
            directive,
        )]));
        feedback.push(s_text_input_f::Block::OneOf(qualities_strings));
        feedback
    }

    fn get_feedback(
        &mut self,
        user_answer: Vec<Vec<String>>,
        directive: String,
        qualities_strings: Vec<String>,
        interaction: &mut impl FnMut(
            Vec<s_text_input_f::Block>,
        ) -> Result<Vec<Vec<String>>, std::io::Error>,
        qualities: [Quality; 3],
    ) -> Result<Quality, std::io::Error> {
        let feedback = self.gen_feedback_form(user_answer, directive, qualities_strings);
        let user_feedback = interaction(feedback)?;
        let i = s_text_input_f::response_as_one_of(user_feedback.last().unwrap().to_owned())
            .unwrap()
            .unwrap();
        let quality = qualities[i];
        Ok(quality)
    }
}

impl Task<'_> for WriteAnswer {
    type SharedState = ();

    fn next_repetition(&self, shared: &(), retrievability_goal: f64) -> SystemTime {
        self.level.next_repetition(shared, retrievability_goal)
    }

    fn complete(
        &mut self,
        _: &mut (),
        _desired_retention: f64,
        interaction: &mut impl FnMut(
            s_text_input_f::Blocks,
        ) -> std::io::Result<s_text_input_f::Response>,
    ) -> std::io::Result<()> {
        let user_answer = interaction(self.input_blocks.clone())?;
        match s_text_input_f::eq_response(&user_answer, &self.correct_answer, true, false) {
            false => {
                const QUALITIES: [Quality; 3] = [
                    Quality::CompleteBlackout,
                    Quality::IncorrectResponseButCorrectRemembered,
                    Quality::IncorrectResponseAndSeemedEasyToRecall,
                ];
                let qualities_strings = vec![
                    "complete blackout".to_string(),
                    "incorrect response, but correct remembered".to_string(),
                    "incorrect response, but seemed easy to recall".to_string(),
                ];
                let directive = "Choose difficulty:".to_string();

                let quality = self.get_feedback(
                    user_answer,
                    directive,
                    qualities_strings,
                    interaction,
                    QUALITIES,
                )?;

                self.level.update(&mut (), (SystemTime::now(), quality));
            }
            true => {
                const QUALITIES: [Quality; 3] = [
                    Quality::CorrectResponseRecalledWithSeriousDifficulty,
                    Quality::CorrectResponseAfterHesitation,
                    Quality::PerfectResponse,
                ];
                let qualities_strings = vec![
                    "recalled with serious difficulty".to_string(),
                    "correct, but after hesitation".to_string(),
                    "perfect response".to_string(),
                ];
                let directive = "All answers correct! Choose difficulty:".to_string();

                let quality = self.get_feedback(
                    user_answer,
                    directive,
                    qualities_strings,
                    interaction,
                    QUALITIES,
                )?;

                self.level.update(&mut (), (SystemTime::now(), quality));
            }
        }
        Ok(())
    }

    fn new(input: s_text_input_f::BlocksWithAnswer) -> Self {
        Self {
            level: Default::default(),
            input_blocks: input.blocks,
            correct_answer: input.answer,
        }
    }

    fn get_blocks(&self) -> s_text_input_f::BlocksWithAnswer {
        BlocksWithAnswer {
            blocks: self.input_blocks.clone(),
            answer: self.correct_answer.clone(),
        }
    }
}
