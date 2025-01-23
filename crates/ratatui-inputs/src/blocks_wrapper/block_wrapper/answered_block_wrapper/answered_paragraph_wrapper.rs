use ratatui::{
    style::Stylize,
    text::{Line, Span},
};

#[derive(Debug)]
pub struct AnsweredParagraphWrapper(Vec<AnsweredParagraphItem>);
impl From<Vec<s_text_input_f::ParagraphItemAnswered>> for AnsweredParagraphWrapper {
    fn from(value: Vec<s_text_input_f::ParagraphItemAnswered>) -> Self {
        Self(value.into_iter().map(|x| x.into()).collect())
    }
}
impl AnsweredParagraphWrapper {
    pub fn as_line(&self) -> Line {
        self.0.iter().flat_map(|x| x.as_spans()).collect()
    }
}

#[derive(Debug)]
enum AnsweredParagraphItem {
    Text(String),
    Answer {
        user_answer: String,
        correct_answer: String,
    },
}
impl From<s_text_input_f::ParagraphItemAnswered> for AnsweredParagraphItem {
    fn from(value: s_text_input_f::ParagraphItemAnswered) -> Self {
        match value {
            s_text_input_f::ParagraphItemAnswered::Text(s) => AnsweredParagraphItem::Text(s),
            s_text_input_f::ParagraphItemAnswered::Answer {
                user_answer,
                correct_answer,
            } => AnsweredParagraphItem::Answer {
                user_answer,
                correct_answer,
            },
        }
    }
}
impl AnsweredParagraphItem {
    pub fn as_spans(&self) -> Vec<Span> {
        match self {
            AnsweredParagraphItem::Text(s) => {
                vec![Span::raw(s)]
            }
            AnsweredParagraphItem::Answer {
                user_answer,
                correct_answer,
            } => {
                let user_answer = if user_answer.trim().is_empty() {
                    "<empty>"
                } else {
                    user_answer
                };
                let correct_answer = if correct_answer.trim().is_empty() {
                    "<empty>"
                } else {
                    correct_answer
                };
                if user_answer.trim() == correct_answer.trim() {
                    vec![Span::raw(correct_answer).green()]
                } else {
                    vec![
                        Span::raw(user_answer).red().crossed_out(),
                        Span::raw(correct_answer).yellow(),
                    ]
                }
            }
        }
    }
}
