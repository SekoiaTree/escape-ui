use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Rect};
use ratatui::prelude::{Layout, Line, Stylize, Widget};
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::text::ToLine;
use ratatui::widgets::{Block, Paragraph, StatefulWidgetRef};
use crate::model::Tab;

#[derive(Debug, Clone)]
pub struct PasswordEntryState {
    entry: String,
    error: bool,
}

impl PasswordEntryState {
    pub fn new() -> Self {
        Self {
            entry: "".to_string(),
            error: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PasswordEntry {
    password: String,
}

impl PasswordEntry {
    pub fn new(password: String) -> PasswordEntry {
        PasswordEntry { password }
    }
}

impl StatefulWidgetRef for PasswordEntry {
    type State = PasswordEntryState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let text = "*".repeat(state.entry.chars().count());
        let block = Block::bordered()
            .title(Line::from("Entrez mot de passe").left_aligned())
            .border_set(border::PLAIN);
        
        let block = if state.error {
            block.border_style(Style::default().red())
        } else {
            block.border_style(Style::new())
        };

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(22),
                Constraint::Fill(1),
            ].into_iter())
            .split(area);

        let layout2 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ].into_iter())
            .split(layout[1]);

        Paragraph::new(text.to_line().left_aligned())
            .centered()
            .block(block)
            .render(layout2[1], buf);
    }
}

impl Tab for PasswordEntry {
    fn handle_input(&self, event: Event, state: &mut Self::State) -> Option<usize> {
        if let Event::Key(KeyEvent { code: KeyCode::Char(c), kind: KeyEventKind::Press, .. }) = event {
            if state.entry.len() < 20 {
                state.entry.push(c);
                state.error = false;
            }
            None
        } else if let Event::Key(KeyEvent { code: KeyCode::Backspace, kind: KeyEventKind::Press, .. }) = event {
            state.entry.pop();
            state.error = false;
            None
        } else if let Event::Key(KeyEvent { code: KeyCode::Enter, kind: KeyEventKind::Press, .. }) = event {
            if state.entry == self.password {
                Some(1) // Email program
            } else {
                state.error = true;
                None
            }
        } else {
            None
        }
    }
}