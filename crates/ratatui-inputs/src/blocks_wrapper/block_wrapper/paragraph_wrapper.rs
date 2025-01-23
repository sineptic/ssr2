use paragraph_item_wrapper::*;
use ratatui::text::Line;

use crate::{split_at_mid, ResultKind};
#[derive(Debug)]
pub struct ParagraphWrapper {
    items: Vec<ParagraphItemWrapper>,
    cursor: usize,
}
impl From<s_text_input_f::Paragraph> for ParagraphWrapper {
    fn from(value: s_text_input_f::Paragraph) -> Self {
        Self {
            items: value.into_iter().map(|x| x.into()).collect(),
            cursor: 0,
        }
    }
}
impl ParagraphWrapper {
    pub fn finalize(self) -> Vec<String> {
        self.items
            .into_iter()
            .filter_map(|x| x.finalize().ok())
            .collect()
    }
    #[allow(clippy::too_many_lines)]
    pub fn get_input(
        &mut self,
        start_from_left: bool,
        render: &mut impl FnMut(Line) -> std::io::Result<()>,
    ) -> Option<std::io::Result<ResultKind>> {
        if start_from_left {
            self.select_first_placeholder()?;
        } else {
            self.select_last_placeholder()?;
        }

        let result_kind = loop {
            let (head, current_placeholder, tail) =
                split_at_mid(&mut self.items, self.cursor).unwrap();
            let get_input_result = get_input(current_placeholder, head, tail, render).unwrap();
            if let Ok(result_kind) = get_input_result {
                match result_kind {
                    ResultKind::Ok => {
                        let next_elem_exist = self.select_next_placeholder().unwrap();
                        if !next_elem_exist {
                            break ResultKind::Ok;
                        }
                    }
                    ResultKind::Canceled => break ResultKind::Canceled,
                    ResultKind::NextBlock => {
                        let next_elem_exist = self.select_next_placeholder().unwrap();
                        if !next_elem_exist {
                            break ResultKind::NextBlock;
                        }
                    }
                    ResultKind::PrevBlock => {
                        let prev_item_exist = self.select_prev_placeholder().unwrap();
                        if !prev_item_exist {
                            break ResultKind::PrevBlock;
                        }
                    }
                };
            } else {
                return Some(get_input_result);
            }
        };
        Some(Ok(result_kind))
    }
    /// # Errors
    /// if there is no placeholders
    fn select_first_placeholder(&mut self) -> Option<()> {
        self.cursor = 0;
        if !self.get_current()?.is_placeholder() {
            let its_wrongly_last = !self.select_next_placeholder()?;
            if its_wrongly_last {
                return None;
            }
        }
        Some(())
    }
    /// # Errors
    /// if there is no placeholders
    fn select_last_placeholder(&mut self) -> Option<()> {
        self.cursor = self.items.len() - 1;
        if !self.get_current()?.is_placeholder() {
            let its_wrongly_first = !self.select_prev_placeholder()?;
            if its_wrongly_first {
                return None;
            }
        }
        Some(())
    }
    /// # Returns
    /// - `Some(true)`  if next placeholder selected
    /// - `Some(false)` if it's last placeholder already
    /// - `None`        if there is no placeholders
    fn select_next_placeholder(&mut self) -> Option<bool> {
        let starting = self.cursor;
        self.cursor += 1;
        if self.cursor == self.items.len() {
            self.cursor = starting;
            return Some(false);
        }
        while (0..(self.items.len() - 1)).contains(&self.cursor)
            && !self.get_current()?.is_placeholder()
        {
            self.cursor += 1;
        }
        if self.get_current()?.is_placeholder() {
            Some(true)
        } else {
            self.cursor = starting;
            Some(false)
        }
    }
    /// # Returns
    /// - `Some(true)`  if prev placeholder selected
    /// - `Some(false)` if it's first placeholder already
    /// - `None`        if there is no placeholders
    fn select_prev_placeholder(&mut self) -> Option<bool> {
        if self.cursor == 0 {
            return Some(false);
        }

        let starting = self.cursor;
        self.cursor -= 1;

        while (1..self.items.len()).contains(&self.cursor) && !self.get_current()?.is_placeholder()
        {
            self.cursor -= 1;
        }
        if self.get_current()?.is_placeholder() {
            Some(true)
        } else {
            self.cursor = starting;
            Some(false)
        }
    }
    fn get_current(&mut self) -> Option<&mut ParagraphItemWrapper> {
        self.items.get_mut(self.cursor)
    }

    pub fn as_line(&self) -> Line {
        self.items.iter().flat_map(|x| x.as_spans()).collect()
    }
}

fn get_input(
    current_placeholder: &mut ParagraphItemWrapper,
    head: &mut [ParagraphItemWrapper],
    tail: &mut [ParagraphItemWrapper],
    render: &mut impl FnMut(Line) -> Result<(), std::io::Error>,
) -> Option<Result<ResultKind, std::io::Error>> {
    current_placeholder.get_input(&mut |current_placeholder_spans| {
        let head_spans = head.iter().flat_map(|x| x.as_spans());
        let tail_spans = tail.iter().flat_map(|x| x.as_spans());
        let line: Line = head_spans
            .chain(current_placeholder_spans)
            .chain(tail_spans)
            .collect();
        render(line)
    })
}

pub mod paragraph_item_wrapper {
    use crate::{blank_field::BlankField, ResultKind};
    use ratatui::{style::Stylize, text::Span};

    #[derive(Debug)]
    pub enum ParagraphItemWrapper {
        Text(String),
        Placeholder(BlankField),
    }
    impl From<s_text_input_f::ParagraphItem> for ParagraphItemWrapper {
        fn from(value: s_text_input_f::ParagraphItem) -> Self {
            match value {
                s_text_input_f::ParagraphItem::Text(s) => Self::Text(s),
                s_text_input_f::ParagraphItem::Placeholder => {
                    Self::Placeholder(BlankField::default())
                }
            }
        }
    }
    impl ParagraphItemWrapper {
        pub fn finalize(self) -> Result<String, Self> {
            self.try_into_placeholder().map(|x| x.text().to_owned())
        }
        pub fn get_input(
            &mut self,
            render: &mut impl FnMut(Vec<Span>) -> std::io::Result<()>,
        ) -> Option<std::io::Result<ResultKind>> {
            let a = self.as_placeholder()?;
            Some((|| {
                Ok(
                    match a.get_input(&mut |c| render_active_blank_field(c, render))? {
                        ResultKind::Ok => ResultKind::Ok,
                        ResultKind::Canceled => ResultKind::Canceled,
                        ResultKind::NextBlock => ResultKind::NextBlock,
                        ResultKind::PrevBlock => ResultKind::PrevBlock,
                    },
                )
            })())
        }
        pub fn as_spans(&self) -> Vec<Span> {
            match self {
                ParagraphItemWrapper::Text(s) => vec![s.into()],
                ParagraphItemWrapper::Placeholder(blank_field) => {
                    if blank_field.is_empty() {
                        vec![Span::raw("<empty>").dark_gray().italic()]
                    } else {
                        vec![Span::raw(blank_field.text()).underlined().gray().italic()]
                    }
                }
            }
        }

        fn try_into_placeholder(self) -> Result<BlankField, Self> {
            if let Self::Placeholder(v) = self {
                Ok(v)
            } else {
                Err(self)
            }
        }

        fn as_placeholder(&mut self) -> Option<&mut BlankField> {
            if let Self::Placeholder(v) = self {
                Some(v)
            } else {
                None
            }
        }

        /// Returns `true` if the paragraph item wrapper is [`Placeholder`].
        ///
        /// [`Placeholder`]: ParagraphItemWrapper::Placeholder
        #[must_use]
        pub fn is_placeholder(&self) -> bool {
            matches!(self, Self::Placeholder(..))
        }
    }
    fn render_active_blank_field(
        blank_field: &BlankField,
        render: &mut impl FnMut(Vec<Span>) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        render(style_active_blank_field(blank_field))
    }
    pub fn style_active_blank_field(blank_field: &BlankField) -> Vec<Span> {
        let chars = &blank_field.text;
        let (a, b) = chars.split_at(blank_field.cursor);
        vec![
            Span::raw(a.iter().collect::<String>())
                .underlined()
                .italic(),
            Span::raw("|").blue(),
            Span::raw(b.iter().collect::<String>())
                .underlined()
                .italic(),
        ]
    }
}
