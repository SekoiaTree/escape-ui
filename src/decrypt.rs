use ratatui::widgets::{Block, Borders};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Style, Stylize, Widget};
use ratatui::symbols;
use ratatui::symbols::border;
use ratatui::text::ToLine;
use ratatui::widgets::{Paragraph, StatefulWidgetRef};
use crate::model::Tab;

#[derive(Debug, Clone)]
pub struct DecryptState {
    entry: String,
    error: bool,
}

impl DecryptState {
    pub fn new() -> Self {
        Self {
            entry: "".to_string(),
            error: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Decrypt;

impl Decrypt {
    pub fn new() -> Decrypt {
        Decrypt
    }
}

impl StatefulWidgetRef for Decrypt {
    type State = DecryptState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered()
            .title(Line::from("Mot de passe encrypté").left_aligned())
            .borders(Borders::ALL & !Borders::BOTTOM)
            .border_set(border::PLAIN);

        let block_top = if state.error {
            block.border_style(Style::default().red())
        } else {
            block.border_style(Style::new())
        };

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(50),
                Constraint::Fill(1),
            ].into_iter())
            .split(area);

        let layout2 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Fill(1),
            ].into_iter())
            .split(layout[1]);

        let block = Block::bordered()
            .title(Line::from("Entrez le mot de passe").left_aligned())
            .border_set(border::Set {
                top_left: symbols::line::NORMAL.vertical_right,
                top_right: symbols::line::NORMAL.vertical_left,
                ..border::Set::default()
            });
        let block = if state.error {
            block.border_style(Style::default().red())
        } else {
            block.border_style(Style::new())
        };

        Paragraph::new(state.entry.to_line().left_aligned())
            .centered()
            .block(block)
            .render(layout2[2], buf);

        Paragraph::new("ÉVLWÉÈJDJ".to_string().to_line().left_aligned())
            .centered()
            .block(block_top)
            .render(layout2[1], buf);
    }
}

impl Tab for Decrypt {
    fn handle_input(&self, event: Event, state: &mut Self::State) -> Option<usize> {
        if let Event::Key(KeyEvent { code: KeyCode::Char(c), kind: KeyEventKind::Press, .. }) = event {
            if state.entry.len() < 20 {
                state.entry.push(c);
                state.error = false;
            }
        } else if let Event::Key(KeyEvent { code: KeyCode::Backspace, kind: KeyEventKind::Press, .. }) = event {
            state.entry.pop();
            state.error = false;
        } else if let Event::Key(KeyEvent { code: KeyCode::Enter, kind: KeyEventKind::Press, .. }) = event {
            if state.entry == "ALMA.PAIX".to_string() {
                return Some(1); // Email program
            } else {
                state.error = true;
            }
        }

        None
    }
}