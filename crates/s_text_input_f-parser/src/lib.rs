use chumsky::prelude::*;
use s_text_input_f::{Block, BlocksWithAnswer};

impl FromIterator<CorrectBlock> for BlocksWithAnswer {
    fn from_iter<T: IntoIterator<Item = CorrectBlock>>(iter: T) -> Self {
        let (blocks, answer) = iter.into_iter().map(|x| (x.block, x.answer)).unzip();
        Self { blocks, answer }
    }
}

/// Represents a parsed interactive block along with its correct answers
#[derive(Debug)]
pub struct CorrectBlock {
    /// The interactive block content (Paragraph, OneOf, or AnyOf)
    pub block: Block,
    /// The correct answers for this block
    pub answer: Vec<String>,
}
impl From<paragraph::CorrectParagraph> for CorrectBlock {
    fn from(value: paragraph::CorrectParagraph) -> Self {
        Self {
            block: Block::Paragraph(value.input),
            answer: value.answer,
        }
    }
}
impl From<one_of::CorrectOneOf> for CorrectBlock {
    fn from(value: one_of::CorrectOneOf) -> Self {
        Self {
            block: Block::OneOf(value.variants),
            answer: vec![value.correct.to_string()],
        }
    }
}
impl From<any_of::CorrectAnyOf> for CorrectBlock {
    fn from(value: any_of::CorrectAnyOf) -> Self {
        Self {
            block: Block::AnyOf(value.variants),
            answer: value.correct.into_iter().map(|x| x.to_string()).collect(),
        }
    }
}

/// Parses a paragraph with placeholders marked by backticks
///
/// # Examples
///
/// ```
/// use s_text_input_f_parser::parse_paragraph;
///
/// let result = parse_paragraph("hello `world`!").unwrap();
/// assert_eq!(result.answer, vec!["world"]);
/// ```
pub fn parse_paragraph(input: &str) -> Result<paragraph::CorrectParagraph, Vec<Simple<char>>> {
    paragraph::paragraph_parser()
        .then_ignore(end())
        .parse(input)
}

/// Parses a single-choice question where one option is marked with asterisk
///
/// # Examples
///
/// ```
/// use s_text_input_f_parser::parse_one_of;
///
/// let result = parse_one_of("- Wrong\n* Correct\n- Wrong").unwrap();
/// assert_eq!(result.correct, 1);
/// ```
pub fn parse_one_of(input: &str) -> Result<one_of::CorrectOneOf, Vec<Simple<char>>> {
    one_of::one_of_parser().then_ignore(end()).parse(input)
}

/// Parses a multiple-choice question where correct options are marked with "[x]"
///
/// # Examples
///
/// ```
/// use s_text_input_f_parser::parse_any_of;
///
/// let result = parse_any_of("- [ ] Wrong\n- [x] Correct\n- [x] Also Correct").unwrap();
/// assert_eq!(result.correct, vec![1, 2]);
/// ```
pub fn parse_any_of(input: &str) -> Result<any_of::CorrectAnyOf, Vec<Simple<char>>> {
    any_of::any_of_parser().then_ignore(end()).parse(input)
}

pub mod any_of;
pub mod one_of;
pub mod paragraph;

/// Parses a single block of any supported type
///
/// # Examples
///
/// ```
/// use s_text_input_f_parser::parse_block;
///
/// let result = parse_block("hello `world`!").unwrap();
/// assert_eq!(result.answer, vec!["world"]);
/// ```
pub fn parse_block(input: &str) -> Result<CorrectBlock, Vec<Simple<char>>> {
    block_parser().then_ignore(end()).parse(input)
}

fn block_parser() -> impl Parser<char, CorrectBlock, Error = Simple<char>> {
    choice((
        any_of::any_of_parser().map(CorrectBlock::from),
        one_of::one_of_parser().map(CorrectBlock::from),
        paragraph::paragraph_parser().map(CorrectBlock::from),
        // TODO: Order
    ))
}

/// Parses a complete document containing multiple blocks
///
/// Blocks must be separated by at least one empty line.
///
/// # Examples
///
/// ```
/// use s_text_input_f_parser::parse_blocks;
///
/// let result = parse_blocks("hello `world`!\n\n- [ ] Wrong\n- [x] Correct").unwrap();
/// assert_eq!(result.blocks.len(), 2);
/// assert_eq!(result.answer.len(), 2);
/// ```
pub fn parse_blocks(input: &str) -> Result<BlocksWithAnswer, Vec<Simple<char>>> {
    blocks_parser().then_ignore(end()).parse(input)
}
fn blocks_parser() -> impl Parser<char, BlocksWithAnswer, Error = Simple<char>> {
    block_parser()
        .separated_by(just('\n').repeated().at_least(1))
        .at_least(1)
        .collect()
}
