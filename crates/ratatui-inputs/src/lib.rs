#![warn(clippy::doc_markdown)]
#![warn(clippy::too_many_lines)]
#![allow(dead_code)] // FIXME: remove this

use ratatui::text::Text;
use s_text_input_f::Block;
use std::fmt::Write;

pub fn get_input(
    input_request: s_text_input_f::Blocks,
    render: &mut impl FnMut(ratatui::text::Text) -> std::io::Result<()>,
) -> Option<std::io::Result<(ResultKind, s_text_input_f::Response)>> {
    let mut blocks_wrapper = blocks_wrapper::BlocksWrapper::from(input_request);
    match blocks_wrapper.get_input(render)? {
        Ok(result_kind) => Some(Ok((result_kind, blocks_wrapper.finalize()))),
        Err(err) => Some(Err(err)),
    }
}

// TODO: Create custom handled for end of interaction
pub fn display_answer(
    input_blocks: s_text_input_f::Blocks,
    user_answer: Vec<Vec<String>>,
    correct_answer: Vec<Vec<String>>,
    render: &mut impl FnMut(ratatui::text::Text) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let answered = {
        let mut temp = s_text_input_f::to_answered(input_blocks, user_answer, correct_answer)
            .into_iter()
            .map(s_text_input_f::Block::Answered)
            .collect::<Vec<_>>();
        temp.push(Block::Paragraph(vec![
            s_text_input_f::ParagraphItem::Placeholder,
        ]));
        temp
    };

    get_input(answered, render)
        .expect("Last elem must be blank field by design. It's a bug")
        .map(|_| ())
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResultKind {
    Ok,
    Canceled,
    NextBlock,
    PrevBlock,
}

mod blank_field;
mod multiline_input;

fn split_at_mid<T>(slice: &mut [T], mid: usize) -> Option<(&mut [T], &mut T, &mut [T])> {
    let (head, tail) = slice.split_at_mut(mid);
    let (mid, tail) = tail.split_first_mut()?;
    Some((head, mid, tail))
}

mod blocks_wrapper;

pub fn get_text_input(
    render: &mut impl FnMut(ratatui::text::Text, String) -> std::io::Result<()>,
) -> std::io::Result<(ResultKind, String)> {
    let mut multyline_input = multiline_input::MultilineInput::default();
    loop {
        match multyline_input.get_input(&mut |x| render(x.style(), x.text()))? {
            ResultKind::Ok => return Ok((ResultKind::Ok, multyline_input.text().to_owned())),
            ResultKind::Canceled => {
                return Ok((ResultKind::Canceled, multyline_input.text().to_owned()))
            }
            ResultKind::NextBlock => (),
            ResultKind::PrevBlock => (),
        }
    }
}

pub fn get_block(
    render: &mut impl FnMut(Text, String) -> std::io::Result<()>,
) -> std::io::Result<Option<s_text_input_f_parser::CorrectBlock>> {
    let (result_kind, inputs) = get_text_input(&mut |styled, text| {
        let support_text = s_text_input_f_parser::parse_block(text.trim())
            .map(|parsed| {
                let mut buffer = String::new();
                let _ = writeln!(buffer, "{parsed:#?}");
                buffer
            })
            .map_err(|err| {
                let mut buffer = String::new();
                for err in err {
                    let _ = writeln!(buffer, "Error: {err}.");
                }
                buffer
            });
        let support_text = match support_text {
            Ok(x) => x,
            Err(x) => x,
        };
        render(styled, support_text)
    })
    .unwrap();
    match result_kind {
        ResultKind::Ok => Ok(s_text_input_f_parser::parse_block(&inputs).ok()),
        ResultKind::Canceled => Ok(None),
        _ => unreachable!(),
    }
}
pub fn get_blocks(
    render: &mut impl FnMut(Text, String) -> std::io::Result<()>,
) -> std::io::Result<Option<s_text_input_f::BlocksWithAnswer>> {
    let (result_kind, inputs) = get_text_input(&mut |styled, text| {
        let support_text = s_text_input_f_parser::parse_blocks(text.trim())
            .map(|parsed| {
                let mut buffer = String::new();
                let _ = writeln!(buffer, "{parsed:#?}");
                buffer
            })
            .map_err(|err| {
                let mut buffer = String::new();
                for err in err {
                    let _ = writeln!(buffer, "Error: {err}.");
                }
                buffer
            });
        let support_text = match support_text {
            Ok(x) => x,
            Err(x) => x,
        };
        render(styled, support_text)
    })
    .unwrap();
    match result_kind {
        ResultKind::Ok => Ok(s_text_input_f_parser::parse_blocks(&inputs).ok()),
        ResultKind::Canceled => Ok(None),
        _ => unreachable!(),
    }
}
