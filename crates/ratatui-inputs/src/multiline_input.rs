use super::ResultKind;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    style::Stylize,
    text::{Line, Span, Text},
};

#[derive(Default, Clone)]
#[readonly::make]
#[derive(Debug)]
pub struct MultilineInput {
    pub text: Vec<char>,
    pub cursor: usize,
}

enum Event {
    AddChar(char),
    RemoveCurrentChar,
    RemoveNextChar,
    MoveCursorLeft,
    MoveCursorRight,
    Finish,
    NextField,
    PrevField,
    Redraw,
    AddString(String),
    Cancel,
}

impl MultilineInput {
    pub fn text(&self) -> String {
        self.text.iter().collect()
    }
    fn add_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.move_cursor_right();
    }
    fn remove_current_char(&mut self) {
        if self.cursor != 0 {
            self.text.remove(self.cursor - 1);
            self.move_cursor_left();
        }
    }
    fn remove_next_char(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }
    fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.cursor = self.cursor.clamp(0, self.text.len());
    }
    fn move_cursor_right(&mut self) {
        self.cursor += 1;
        self.cursor = self.cursor.clamp(0, self.text.len());
    }
    #[allow(clippy::too_many_lines)]
    fn get_event() -> std::io::Result<Event> {
        Ok({
            loop {
                if let Some(x) = match crossterm::event::read()? {
                    crossterm::event::Event::Key(k) => {
                        if k.kind == KeyEventKind::Press {
                            match k.code {
                                KeyCode::Backspace => Some(Event::RemoveCurrentChar),
                                KeyCode::Enter => {
                                    if k.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                                        Some(Event::Finish)
                                    } else {
                                        Some(Event::AddChar('\n'))
                                    }
                                }
                                KeyCode::Left => Some(Event::MoveCursorLeft),
                                KeyCode::Right => Some(Event::MoveCursorRight),
                                KeyCode::Tab => Some(Event::NextField),
                                KeyCode::BackTab => Some(Event::PrevField),
                                KeyCode::Delete => Some(Event::RemoveNextChar),
                                KeyCode::Char(c) => Some(Event::AddChar(c)),
                                KeyCode::Esc => Some(Event::Cancel),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }
                    crossterm::event::Event::Paste(s) => Some(Event::AddString(s)),
                    crossterm::event::Event::Resize(_, _) => Some(Event::Redraw),
                    _ => None,
                } {
                    break x;
                }
            }
        })
    }
    fn handle_event(&mut self, event: Event) -> Option<Event> {
        match event {
            Event::AddChar(c) => self.add_char(c),
            Event::RemoveCurrentChar => self.remove_current_char(),
            Event::RemoveNextChar => self.remove_next_char(),
            Event::MoveCursorLeft => self.move_cursor_left(),
            Event::MoveCursorRight => self.move_cursor_right(),
            Event::Finish => return Some(event),
            Event::NextField => return Some(event),
            Event::PrevField => return Some(event),
            Event::Redraw => (),
            Event::AddString(s) => s.chars().for_each(|c| self.add_char(c)),
            Event::Cancel => return Some(event),
        }
        None
    }
}

impl MultilineInput {
    pub fn get_input(
        &mut self,
        render: &mut impl FnMut(&Self) -> std::io::Result<()>,
    ) -> std::io::Result<ResultKind> {
        loop {
            render(self)?;
            if let Some(x) = self.handle_event(Self::get_event()?) {
                match x {
                    Event::Finish => return Ok(ResultKind::Ok),
                    Event::NextField => return Ok(ResultKind::NextBlock),
                    Event::PrevField => return Ok(ResultKind::PrevBlock),
                    Event::Cancel => return Ok(ResultKind::Canceled),
                    _ => unreachable!(),
                }
            }
        }
    }
    pub fn style(&self) -> Text {
        let chars = &self.text;
        let (a, b) = chars.split_at(self.cursor);
        let a = a
            .split(|ch| *ch == '\n')
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<_>>();
        let b = b
            .split(|ch| *ch == '\n')
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<_>>();

        let mut lines = vec![];
        let mut current_line = vec![];
        if let Some((mid_left, start)) = a.split_last() {
            lines.append(
                &mut start
                    .iter()
                    .cloned()
                    .map(|x| Line::raw(x).italic())
                    .collect(),
            );
            current_line.push(Span::raw(mid_left.to_owned()).italic());
        }
        current_line.push(Span::raw("|").blue());
        if let Some((mid_right, end)) = b.split_first() {
            current_line.push(Span::raw(mid_right.to_owned()).italic());
            lines.push(Line::from(current_line));
            lines.append(&mut end.iter().cloned().map(|x| Line::raw(x).italic()).collect())
        } else {
            lines.push(Line::from(current_line));
        }

        Text::from(lines)
    }
}
