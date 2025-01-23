#![warn(clippy::doc_markdown)]

use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, num::ParseIntError};

pub type Blocks = Vec<Block>;
#[derive(Debug)]
pub struct BlocksWithAnswer {
    pub blocks: Blocks,
    pub answer: Response,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub enum Block {
    /// # Response
    /// Vec of starting positions(stating from 0; stringified) of each elem in the same order as it's displayed for user on
    /// screan
    Order(Vec<String>),
    /// # Response
    /// Vec of {`answer_positions`(starting from 0)}.`to_string()`
    AnyOf(Vec<String>),
    /// # Response
    /// vec!\[{`answer_position`(starting from 0)}.`to_string()`]
    OneOf(Vec<String>),
    /// # Response
    /// Vec of all placeholder answers in same order.
    Paragraph(Paragraph),
    Answered(BlockAnswered),
}

pub type Paragraph = Vec<ParagraphItem>;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ParagraphItem {
    Text(String),
    Placeholder,
}

pub type Response = Vec<ResponseItem>;
pub type ResponseItem = Vec<String>;
pub fn eq_response(a: &Response, b: &Response, trim: bool, case_insensitive: bool) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b)
        .all(|(a, b)| eq_response_item(a, b, trim, case_insensitive))
}
fn eq_response_item(
    a: &ResponseItem,
    b: &ResponseItem,
    trim: bool,
    case_insensitive: bool,
) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).all(|(a, b)| {
        if trim {
            let (a, b) = (a.trim(), b.trim());
            if case_insensitive {
                let (a, b) = (a.to_lowercase(), b.to_lowercase());
                a == b
            } else {
                a == b
            }
        } else {
            #[allow(clippy::collapsible_if)]
            if case_insensitive {
                let (a, b) = (a.to_lowercase(), b.to_lowercase());
                a == b
            } else {
                a == b
            }
        }
    })
}

/// # Returns
/// Vec of (`initial_pos`, `current_pos`)
pub fn response_as_order(response: ResponseItem) -> Result<Vec<(usize, usize)>, ParseIntError> {
    response
        .into_iter()
        .enumerate()
        .map(|(i, a)| -> Result<_, ParseIntError> { Ok((a.parse()?, i)) })
        .collect::<Result<Vec<_>, _>>()
        .map(|mut vec| {
            vec.sort_unstable();
            vec
        })
}
/// # Returns
/// `BTreeSet` of selected items
pub fn response_as_any_of(response: ResponseItem) -> Result<BTreeSet<usize>, ParseIntError> {
    response.into_iter().map(|s| s.parse()).collect()
}
/// # Errors
/// - `None` if there not 1 element
/// - `Some(ParseIntError)` if there not a number
pub fn response_as_one_of(response: ResponseItem) -> Option<Result<usize, ParseIntError>> {
    let [val]: [String; 1] = response.try_into().ok()?;
    Some(val.parse())
}
pub fn response_as_placeholders(response: ResponseItem) -> Vec<String> {
    response
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BlockAnswered {
    Order {
        items: Vec<String>,
        user_answer: Vec<usize>,
        correct_answer: Vec<usize>,
    },
    AnyOf {
        items: Vec<String>,
        user_answer: Vec<usize>,
        correct_answer: Vec<usize>,
    },
    OneOf {
        items: Vec<String>,
        user_answer: usize,
        correct_answer: usize,
    },
    Paragraph(Vec<ParagraphItemAnswered>),
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ParagraphItemAnswered {
    Text(String),
    Answer {
        user_answer: String,
        correct_answer: String,
    },
}
impl From<(Block, Vec<String>, Vec<String>)> for BlockAnswered {
    fn from((block, user_answer, correct_answer): (Block, Vec<String>, Vec<String>)) -> Self {
        match block {
            Block::Order(_) => todo!(),
            Block::AnyOf(_) => todo!(),
            Block::OneOf(items) => {
                let [user_answer] = user_answer.try_into().unwrap();
                let [correct_answer] = correct_answer.try_into().unwrap();
                let (user_answer, correct_answer): (usize, usize) = (
                    user_answer.parse().unwrap(),
                    correct_answer.parse().unwrap(),
                );
                Self::OneOf {
                    items,
                    user_answer,
                    correct_answer,
                }
            }
            Block::Paragraph(paragraph_items) => {
                let mut paragraph_items = paragraph_items
                    .into_iter()
                    .map(|x| match x {
                        ParagraphItem::Text(x) => ParagraphItemAnswered::Text(x),
                        ParagraphItem::Placeholder => ParagraphItemAnswered::Answer {
                            user_answer: String::new(),
                            correct_answer: String::new(),
                        },
                    })
                    .collect::<Vec<_>>();
                paragraph_items
                    .iter_mut()
                    .filter(|x| {
                        matches!(
                            x,
                            ParagraphItemAnswered::Answer {
                                user_answer: _,
                                correct_answer: _
                            }
                        )
                    })
                    .zip(user_answer.into_iter().zip(correct_answer))
                    .for_each(|(answered, (user_answer, correct_answer))| {
                        *answered = ParagraphItemAnswered::Answer {
                            user_answer,
                            correct_answer,
                        };
                    });
                Self::Paragraph(paragraph_items)
            }
            Block::Answered(_) => panic!("You can't answer already answered block"),
        }
    }
}
pub fn to_answered(
    blocks: Blocks,
    user_answers: Response,
    correct_answers: Response,
) -> Vec<BlockAnswered> {
    assert_eq!(user_answers.len(), correct_answers.len());
    blocks
        .into_iter()
        .zip(user_answers.into_iter().zip(correct_answers))
        .map(|(block, (user_answer, correct_answer))| {
            BlockAnswered::from((block, user_answer, correct_answer))
        })
        .collect()
}
