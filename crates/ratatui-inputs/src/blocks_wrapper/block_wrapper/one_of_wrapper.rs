use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
};

use crate::ResultKind;

#[derive(Debug)]
pub struct OneOfWrapper {
    items: Vec<String>,
    cursor: usize,
    selected: Option<usize>,
}
impl From<Vec<String>> for OneOfWrapper {
    fn from(items: Vec<String>) -> Self {
        Self {
            items,
            cursor: 0,
            selected: None,
        }
    }
}
impl OneOfWrapper {
    pub fn finalize(self) -> Vec<String> {
        if let Some(selected) = self.selected {
            vec![selected.to_string()]
        } else {
            vec!["0".into()]
        }
    }
    pub fn get_input(
        &mut self,
        start_from_left: bool,
        render: &mut impl FnMut(Vec<Line>) -> std::io::Result<()>,
    ) -> Option<std::io::Result<ResultKind>> {
        if start_from_left {
            self.select_first_placeholder()?;
        } else {
            self.select_last_placeholder()?;
        }
        let mut render = |one_of: &OneOfWrapper| {
            let mut lines = one_of.as_lines();
            lines[one_of.cursor] = lines[one_of.cursor]
                .to_owned()
                .patch_style(current_line_styles());
            render(lines)
        };
        Some((|| loop {
            render(self)?;
            if let Some(event) = self.handle_event(Self::get_event()?) {
                break Ok(event);
            }
        })())
    }
    /// # Errors
    /// if there is no items
    fn select_first_placeholder(&mut self) -> Option<()> {
        if self.items.is_empty() {
            None
        } else {
            self.cursor = 0;
            Some(())
        }
    }
    /// # Errors
    /// if there is no items
    fn select_last_placeholder(&mut self) -> Option<()> {
        if self.items.is_empty() {
            None
        } else {
            self.cursor = self.items.len() - 1;
            Some(())
        }
    }
    /// # Returns
    /// - `Some(true)`  if next item selected
    /// - `Some(false)` if it's last item already
    /// - `None`        if there is no items
    fn select_next_placeholder(&mut self) -> Option<bool> {
        if self.items.is_empty() {
            None
        } else {
            let start = self.cursor;
            self.cursor += 1;
            if (0..(self.items.len())).contains(&self.cursor) {
                Some(true)
            } else {
                self.cursor = start;
                Some(false)
            }
        }
    }
    /// # Returns
    /// - `Some(true)`  if prev item selected
    /// - `Some(false)` if it's first item already
    /// - `None`        if there is no items
    fn select_prev_placeholder(&mut self) -> Option<bool> {
        if self.items.is_empty() {
            None
        } else if let Some(x) = self.cursor.checked_sub(1) {
            self.cursor = x;
            Some(true)
        } else {
            Some(false)
        }
    }

    pub fn as_lines(&self) -> Vec<Line> {
        let mut lines = self.items.iter().map(|x| as_line(x)).collect::<Vec<_>>();
        if let Some(selected) = self.selected {
            lines[selected] = lines[selected]
                .to_owned()
                .patch_style(selected_line_styles());
        }
        lines
    }
}
#[derive(Clone, Copy)]
enum Event {
    Select,
    NextItem,
    PrevItem,
    NextBlock,
    EnterKey,
    PrevBlock,
    Redraw,
    Cancel,
}
impl OneOfWrapper {
    #[allow(clippy::too_many_lines)]
    fn get_event() -> std::io::Result<Event> {
        Ok({
            loop {
                if let Some(x) = match crossterm::event::read()? {
                    crossterm::event::Event::Key(k) => {
                        if k.kind == KeyEventKind::Press {
                            match k.code {
                                KeyCode::Char(' ') => Some(Event::Select),
                                KeyCode::Enter => Some(Event::EnterKey),
                                KeyCode::Down | KeyCode::Char('j' | 'J') => Some(Event::NextItem),
                                KeyCode::Up | KeyCode::Char('k' | 'K') => Some(Event::PrevItem),
                                KeyCode::Tab => {
                                    if cfg!(feature = "fast_tab_scroll") {
                                        Some(Event::NextBlock)
                                    } else {
                                        Some(Event::NextItem)
                                    }
                                }
                                KeyCode::BackTab => {
                                    if cfg!(feature = "fast_tab_scroll") {
                                        Some(Event::PrevBlock)
                                    } else {
                                        Some(Event::PrevItem)
                                    }
                                }
                                KeyCode::Esc | KeyCode::Char('q' | 'Q') => Some(Event::Cancel),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }
                    crossterm::event::Event::Resize(_, _) => Some(Event::Redraw),
                    _ => None,
                } {
                    break x;
                }
            }
        })
    }
    fn handle_event(&mut self, event: Event) -> Option<ResultKind> {
        match event {
            Event::Select => {
                self.selected = Some(self.cursor);
                None
            }
            Event::NextItem => {
                let already_last_elem = !self.select_next_placeholder().unwrap();
                if already_last_elem {
                    Some(ResultKind::NextBlock)
                } else {
                    None
                }
            }
            Event::PrevItem => {
                let already_first_elem = !self.select_prev_placeholder().unwrap();
                if already_first_elem {
                    Some(ResultKind::PrevBlock)
                } else {
                    None
                }
            }
            Event::NextBlock => {
                if self.selected.is_some() {
                    Some(ResultKind::NextBlock)
                } else {
                    let already_last_elem = !self.select_next_placeholder().unwrap();
                    if already_last_elem {
                        Some(ResultKind::NextBlock)
                    } else {
                        None
                    }
                }
            }
            #[allow(clippy::collapsible_else_if)]
            Event::EnterKey => {
                if self.selected.is_some() {
                    Some(ResultKind::Ok)
                } else {
                    if cfg!(feature = "fast_select_with_enter") {
                        self.selected = Some(self.cursor);
                        Some(ResultKind::Ok)
                    } else {
                        self.select_next_placeholder().unwrap();
                        None
                    }
                }
            }
            Event::PrevBlock => {
                if self.selected.is_some() {
                    Some(ResultKind::PrevBlock)
                } else {
                    let already_first_elem = !self.select_prev_placeholder().unwrap();
                    if already_first_elem {
                        Some(ResultKind::PrevBlock)
                    } else {
                        None
                    }
                }
            }
            Event::Redraw => None,
            Event::Cancel => Some(ResultKind::Canceled),
        }
    }
}
fn as_line(s: &str) -> Line {
    Line::from(vec![Span::raw(" -  ").blue(), Span::raw(s)]).italic()
}
fn current_line_styles() -> Style {
    Style::new().bold().fg(ratatui::style::Color::Magenta)
}
fn selected_line_styles() -> Style {
    Style::new().bold().not_italic()
}
