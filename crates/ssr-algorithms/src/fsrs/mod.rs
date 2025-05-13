use std::time::SystemTime;

use s_text_input_f as stif;
use s_text_input_f::{BlocksWithAnswer, ParagraphItem};
use serde::{Deserialize, Serialize};

pub mod level;
use level::{Level, Quality, RepetitionContext};
use weights::Weights;

pub mod stateless;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    level: level::Level,
    input_blocks: s_text_input_f::Blocks,
    correct_answer: s_text_input_f::Response,
    #[serde(default)]
    other_answers: Vec<s_text_input_f::Response>,
}

pub mod weights;

impl ssr_core::task::Task<'_> for Task {
    type SharedState = Weights;

    fn next_repetition(&self, shared_state: &Weights, retrievability_goal: f64) -> SystemTime {
        self.level
            .next_repetition(&shared_state.fsrs(), retrievability_goal)
    }

    fn complete(
        &mut self,
        shared_state: &mut Self::SharedState,
        desired_retention: f64,
        interaction: &mut impl FnMut(
            s_text_input_f::Blocks,
        ) -> std::io::Result<s_text_input_f::Response>,
    ) -> std::io::Result<()> {
        let review_time = chrono::Local::now();
        let user_answer = interaction(self.input_blocks.clone())?;
        let quality =
            self.complete_inner(user_answer, shared_state, desired_retention, interaction)?;
        self.level.add_repetition(RepetitionContext {
            quality,
            review_time,
        });
        Ok(())
    }

    fn new(input: s_text_input_f::BlocksWithAnswer) -> Self {
        Self {
            level: Level::default(),
            input_blocks: input.blocks,
            correct_answer: input.answer,
            other_answers: Vec::new(),
        }
    }

    fn get_blocks(&self) -> s_text_input_f::BlocksWithAnswer {
        BlocksWithAnswer {
            blocks: self.input_blocks.clone(),
            answer: self.correct_answer.clone(),
        }
    }
}

pub enum Correctness {
    Wrong,
    DefaultCorrect,
    OtherCorrect { index: usize },
}
impl Correctness {
    #[must_use]
    pub fn is_correct(&self) -> bool {
        match self {
            Correctness::Wrong => false,
            Correctness::DefaultCorrect | Correctness::OtherCorrect { index: _ } => true,
        }
    }
}

impl Task {
    #[must_use]
    pub fn new(
        input_blocks: s_text_input_f::Blocks,
        correct_answer: s_text_input_f::Response,
        other_answers: Vec<s_text_input_f::Response>,
    ) -> Self {
        Self {
            level: Level::default(),
            input_blocks,
            correct_answer,
            other_answers,
        }
    }
    fn gen_feedback_form(
        &mut self,
        user_answer: Vec<Vec<String>>,
        directive: String,
        qualities_strings: Vec<String>,
    ) -> Vec<s_text_input_f::Block> {
        let correct_answer = match self.correctness(&user_answer) {
            Correctness::Wrong | Correctness::DefaultCorrect => self.correct_answer.clone(),
            Correctness::OtherCorrect { index } => self.other_answers[index].clone(),
        };
        let mut feedback =
            s_text_input_f::to_answered(self.input_blocks.clone(), user_answer, correct_answer)
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

    fn get_feedback<T: Copy>(
        &mut self,
        user_answer: Vec<Vec<String>>,
        directive: String,
        qualities_strings: Vec<String>,
        interaction: &mut impl FnMut(
            Vec<s_text_input_f::Block>,
        ) -> Result<Vec<Vec<String>>, std::io::Error>,
        qualities: &[T],
    ) -> Result<T, std::io::Error> {
        let feedback = self.gen_feedback_form(user_answer, directive, qualities_strings);
        let user_feedback = interaction(feedback)?;
        let i = s_text_input_f::response_as_one_of(user_feedback.last().unwrap().to_owned())
            .unwrap()
            .unwrap();
        let quality = qualities[i];
        Ok(quality)
    }

    fn complete_inner(
        &mut self,
        user_answer: Vec<Vec<String>>,
        shared_state: &Weights,
        retrievability_goal: f64,
        interaction: &mut impl FnMut(s_text_input_f::Blocks) -> std::io::Result<Vec<Vec<String>>>,
    ) -> std::io::Result<Quality> {
        let next_states = self.next_states(shared_state, retrievability_goal);
        Ok(if self.correctness(&user_answer).is_correct() {
            self.feedback_correct(user_answer, &next_states, interaction)?
        } else {
            self.feedback_wrong(user_answer, &next_states, interaction)?
        })
    }
    fn correctness(&mut self, user_answer: &Vec<Vec<String>>) -> Correctness {
        if stif::eq_response(&self.correct_answer, user_answer, true, false) {
            return Correctness::DefaultCorrect;
        }
        for (index, ans) in self.other_answers.iter().enumerate() {
            if stif::eq_response(ans, user_answer, true, false) {
                return Correctness::OtherCorrect { index };
            }
        }
        Correctness::Wrong
    }

    fn next_states(&self, weights: &Weights, retrievability_goal: f64) -> fsrs::NextStates {
        self.level.next_states(
            &weights.fsrs(),
            retrievability_goal as f32,
            chrono::Local::now(),
        )
    }

    fn feedback_correct(
        &mut self,
        user_answer: Vec<Vec<String>>,
        next_states: &fsrs::NextStates,
        interaction: &mut impl FnMut(s_text_input_f::Blocks) -> std::io::Result<Vec<Vec<String>>>,
    ) -> std::io::Result<Quality> {
        let qualities = vec![Quality::Hard, Quality::Good, Quality::Easy];
        let qualities_strings = vec![
            format!("Hard {}d", next_states.hard.interval),
            format!("Good {}d", next_states.good.interval),
            format!("Easy {}d", next_states.easy.interval),
        ];
        let directive = "All answers correct! Choose difficulty:".to_string();
        self.get_feedback(
            user_answer,
            directive,
            qualities_strings,
            interaction,
            &qualities,
        )
    }

    fn feedback_wrong(
        &mut self,
        user_answer: Vec<Vec<String>>,
        next_states: &fsrs::NextStates,
        interaction: &mut impl FnMut(s_text_input_f::Blocks) -> std::io::Result<Vec<Vec<String>>>,
    ) -> std::io::Result<Quality> {
        #[derive(Clone, Copy)]
        enum Feedback {
            Wrong,
            ActuallyCorrect,
        }
        let result = self.get_feedback(
            user_answer.clone(),
            "Your answer is wrong.".into(),
            vec![
                format!("OK {}h", next_states.again.interval * 24.),
                "It is actually correct".into(),
            ],
            interaction,
            &[Feedback::Wrong, Feedback::ActuallyCorrect],
        )?;
        match result {
            Feedback::Wrong => Ok(Quality::Again),
            Feedback::ActuallyCorrect => {
                self.other_answers.push(user_answer.clone());
                self.feedback_correct(user_answer, next_states, interaction)
            }
        }
    }
}
