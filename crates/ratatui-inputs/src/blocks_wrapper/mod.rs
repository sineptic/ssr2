use block_wrapper::BlockWrapper;
use ratatui::text::Text;

use crate::{split_at_mid, ResultKind};

#[derive(Debug)]
pub struct BlocksWrapper {
    items: Vec<BlockWrapper>,
    cursor: usize,
    start_from_left: bool,
}
impl From<s_text_input_f::Blocks> for BlocksWrapper {
    fn from(value: s_text_input_f::Blocks) -> Self {
        Self {
            items: value.into_iter().map(|x| x.into()).collect(),
            cursor: 0,
            start_from_left: true,
        }
    }
}
impl BlocksWrapper {
    pub fn finalize(self) -> Vec<Vec<String>> {
        self.items.into_iter().map(|x| x.finalize()).collect()
    }
    pub fn get_input(
        &mut self,
        render: &mut impl FnMut(Text) -> std::io::Result<()>,
    ) -> Option<std::io::Result<ResultKind>> {
        // TODO: add support for start from first and from last
        self.select_first_block()?;

        let result_kind = loop {
            let (head, current_block, tail) = split_at_mid(&mut self.items, self.cursor).unwrap();
            let get_input_result = {
                let maybe_result = current_block.get_input(
                    self.start_from_left,
                    &mut |current_placeholder_lines| {
                        let head_lines = head.iter().flat_map(|x| x.as_lines());
                        let tail_lines = tail.iter().flat_map(|x| x.as_lines());
                        let text: Text = head_lines
                            .chain(current_placeholder_lines)
                            .chain(tail_lines)
                            .collect();
                        render(text)
                    },
                );
                if let Some(result) = maybe_result {
                    result
                } else if self.select_next_block()? {
                    continue;
                } else {
                    return None;
                }
            };
            if let Ok(result_kind) = get_input_result {
                match result_kind {
                    ResultKind::Ok => {
                        let next_elem_exist = self.select_next_block().unwrap();
                        if !next_elem_exist {
                            break ResultKind::Ok;
                        }
                    }
                    ResultKind::Canceled => break ResultKind::Canceled,
                    ResultKind::NextBlock => {
                        self.select_next_block().unwrap();
                    }
                    ResultKind::PrevBlock => {
                        self.select_prev_block().unwrap();
                    }
                };
            } else {
                return Some(get_input_result);
            }
        };
        Some(Ok(result_kind))
    }
    /// # Errors
    /// if there is no blocks
    fn select_first_block(&mut self) -> Option<()> {
        if self.items.is_empty() {
            None
        } else {
            self.cursor = 0;
            Some(())
        }
    }
    /// # Returns
    /// - `Some(true)`  if next block selected
    /// - `Some(false)` if it's last block already
    /// - `None`        if there is no blocks
    fn select_next_block(&mut self) -> Option<bool> {
        if self.items.is_empty() {
            None
        } else {
            self.cursor += 1;
            if self.cursor < self.items.len() {
                self.start_from_left = true;
                Some(true)
            } else {
                self.cursor -= 1;
                self.start_from_left = false;
                Some(false)
            }
        }
    }
    /// # Returns
    /// - `Some(true)`  if prev block selected
    /// - `Some(false)` if it's first block already
    /// - `None`        if there is no blocks
    fn select_prev_block(&mut self) -> Option<bool> {
        if self.items.is_empty() {
            None
        } else if let Some(x) = self.cursor.checked_sub(1) {
            self.start_from_left = false;
            self.cursor = x;
            Some(true)
        } else {
            self.start_from_left = true;
            Some(false)
        }
    }
}

pub mod block_wrapper;
