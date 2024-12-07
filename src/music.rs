use std::path::PathBuf;
use awedio::manager::Manager;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::text::ToLine;
use ratatui::widgets::{Block, Paragraph, StatefulWidgetRef};
use crate::model::Tab;

#[derive(Debug, Clone)]
pub struct MusicPlayerState {
    playing: bool,
    manager: Manager,
    entry: String,
    error: bool,
}

impl MusicPlayerState {
    pub fn new(manager: Manager) -> Self {
        Self {
            playing: false,
            manager,
            entry: "".to_string(),
            error: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MusicPlayer {
    file: PathBuf
}

impl MusicPlayer {
    pub fn new(file: PathBuf) -> MusicPlayer {
        MusicPlayer {
            file
        }
    }
}

impl StatefulWidgetRef for MusicPlayer {
    type State = MusicPlayerState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered()
            .title(Line::from("Decryption du stockage (ESP pour indice)").left_aligned())
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
                Constraint::Length(50),
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

        Paragraph::new(state.entry.to_line().left_aligned())
            .centered()
            .block(block)
            .render(layout2[1], buf);
    }
}

impl Tab for MusicPlayer {
    fn handle_input(&self, event: Event, state: &mut Self::State) -> Option<usize> {
        if let Event::Key(KeyEvent { code: KeyCode::Char(' '), kind: KeyEventKind::Press, .. }) = event {
            state.manager.play(awedio::sounds::open_file(&self.file).unwrap());
        } else if let Event::Key(KeyEvent { code: KeyCode::Char(c), kind: KeyEventKind::Press, .. }) = event {
            if state.entry.len() < 20 {
                state.entry.push(c);
                state.error = false;
            }
        } else if let Event::Key(KeyEvent { code: KeyCode::Backspace, kind: KeyEventKind::Press, .. }) = event {
            state.entry.pop();
            state.error = false;
        } else if let Event::Key(KeyEvent { code: KeyCode::Enter, kind: KeyEventKind::Press, .. }) = event {
            if state.entry == "2.5.".to_string() || state.entry == "2.5".to_string() {
                return Some(1); // Email program
            } else {
                state.error = true;
            }
        }
        
        None
    }
}