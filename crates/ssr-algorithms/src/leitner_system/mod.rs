use std::time::SystemTime;

use level::Level;
use s_text_input_f::BlocksWithAnswer;
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
}

impl Task<'_> for WriteAnswer {
    type SharedState = ();

    fn next_repetition(&self, shared: &(), _: f64) -> SystemTime {
        self.level.next_repetition(shared, 0.)
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
                let mut feedback = s_text_input_f::to_answered(
                    self.input_blocks.clone(),
                    user_answer,
                    self.correct_answer.clone(),
                )
                .into_iter()
                .map(s_text_input_f::Block::Answered)
                .collect::<Vec<_>>();
                feedback.push(s_text_input_f::Block::Paragraph(vec![]));
                feedback.push(s_text_input_f::Block::OneOf(vec!["OK".to_string()]));
                interaction(feedback)?;
                self.level.update(&mut (), (SystemTime::now(), false));
            }
            true => {
                let feedback = vec![
                    s_text_input_f::Block::Paragraph(vec![s_text_input_f::ParagraphItem::Text(
                        "All answers correct!".to_string(),
                    )]),
                    s_text_input_f::Block::OneOf(vec!["OK".to_string()]),
                ];
                interaction(feedback)?;
                self.level.update(&mut (), (SystemTime::now(), true));
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
