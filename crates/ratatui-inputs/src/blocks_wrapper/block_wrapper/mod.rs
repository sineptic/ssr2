use crate::ResultKind;
use ratatui::text::Line;

#[derive(Debug)]
pub enum BlockWrapper {
    Order,
    AnyOf,
    OneOf(one_of_wrapper::OneOfWrapper),
    Paragraph(paragraph_wrapper::ParagraphWrapper),
    Answered(answered_block_wrapper::AnsweredBlockWrapper),
}
impl From<s_text_input_f::Block> for BlockWrapper {
    fn from(value: s_text_input_f::Block) -> Self {
        match value {
            s_text_input_f::Block::Order(_) => todo!("`order` input not implemented"),
            s_text_input_f::Block::AnyOf(_) => todo!("`any_of` input not implemented"),
            s_text_input_f::Block::OneOf(items) => {
                Self::OneOf(one_of_wrapper::OneOfWrapper::from(items))
            }
            s_text_input_f::Block::Paragraph(p) => {
                Self::Paragraph(paragraph_wrapper::ParagraphWrapper::from(p))
            }
            s_text_input_f::Block::Answered(a) => {
                Self::Answered(answered_block_wrapper::AnsweredBlockWrapper::from(a))
            }
            _ => todo!(),
        }
    }
}
impl BlockWrapper {
    pub fn finalize(self) -> Vec<String> {
        match self {
            BlockWrapper::Order => todo!(),
            BlockWrapper::AnyOf => todo!(),
            BlockWrapper::OneOf(o) => o.finalize(),
            BlockWrapper::Paragraph(p) => p.finalize(),
            BlockWrapper::Answered(_) => vec![],
        }
    }
    pub fn get_input(
        &mut self,
        start_from_left: bool,
        render: &mut impl FnMut(Vec<Line>) -> std::io::Result<()>,
    ) -> Option<std::io::Result<ResultKind>> {
        match self {
            BlockWrapper::Order => todo!(),
            BlockWrapper::AnyOf => todo!(),
            BlockWrapper::OneOf(o) => o.get_input(start_from_left, render),
            BlockWrapper::Paragraph(p) => {
                p.get_input(start_from_left, &mut |line| render(vec![line]))
            }
            BlockWrapper::Answered(_) => None,
        }
    }
    pub fn as_lines(&self) -> Vec<Line> {
        match self {
            BlockWrapper::Order => todo!(),
            BlockWrapper::AnyOf => todo!(),
            BlockWrapper::OneOf(o) => o.as_lines(),
            BlockWrapper::Paragraph(p) => vec![p.as_line()],
            BlockWrapper::Answered(a) => a.as_lines(),
        }
    }
}

mod one_of_wrapper;
pub mod paragraph_wrapper;
mod answered_block_wrapper {
    use answered_one_of_wrapper::AnsweredOneOfWrapper;
    use answered_paragraph_wrapper::AnsweredParagraphWrapper;
    use ratatui::text::Line;

    #[derive(Debug)]
    pub enum AnsweredBlockWrapper {
        Order,
        AnyOf,
        OneOf(AnsweredOneOfWrapper),
        Paragraph(AnsweredParagraphWrapper),
    }
    impl From<s_text_input_f::BlockAnswered> for AnsweredBlockWrapper {
        fn from(value: s_text_input_f::BlockAnswered) -> Self {
            match value {
                s_text_input_f::BlockAnswered::Order {
                    items: _,
                    user_answer: _,
                    correct_answer: _,
                } => todo!(),
                s_text_input_f::BlockAnswered::AnyOf {
                    items: _,
                    user_answer: _,
                    correct_answer: _,
                } => todo!(),
                s_text_input_f::BlockAnswered::OneOf {
                    items,
                    user_answer,
                    correct_answer,
                } => Self::OneOf(AnsweredOneOfWrapper::new(
                    items,
                    user_answer,
                    correct_answer,
                )),
                s_text_input_f::BlockAnswered::Paragraph(p) => {
                    Self::Paragraph(AnsweredParagraphWrapper::from(p))
                }
                _ => todo!(),
            }
        }
    }
    impl AnsweredBlockWrapper {
        pub fn as_lines(&self) -> Vec<Line> {
            match self {
                AnsweredBlockWrapper::Order => todo!(),
                AnsweredBlockWrapper::AnyOf => todo!(),
                AnsweredBlockWrapper::OneOf(x) => x.as_lines(),
                AnsweredBlockWrapper::Paragraph(x) => {
                    vec![x.as_line()]
                }
            }
        }
    }

    mod answered_paragraph_wrapper;
    mod answered_one_of_wrapper {
        use ratatui::{
            style::{Style, Stylize},
            text::{Line, Span},
        };

        #[derive(Debug)]
        pub struct AnsweredOneOfWrapper {
            items: Vec<String>,
            user_answer: usize,
            correct_answer: usize,
        }
        impl AnsweredOneOfWrapper {
            pub fn new(items: Vec<String>, user_answer: usize, correct_answer: usize) -> Self {
                Self {
                    items,
                    user_answer,
                    correct_answer,
                }
            }
            pub fn as_lines(&self) -> Vec<Line> {
                let mut lines = self
                    .items
                    .iter()
                    .map(|x| Line::from(vec![Span::raw(" -  ").white(), Span::raw(x.as_str())]))
                    .collect::<Vec<_>>();
                if self.user_answer != self.correct_answer {
                    lines[self.user_answer] = lines[self.user_answer]
                        .to_owned()
                        .patch_style(Style::new().red());
                }
                lines[self.correct_answer] = lines[self.correct_answer]
                    .to_owned()
                    .patch_style(Style::new().green());
                lines
            }
        }
    }
}
