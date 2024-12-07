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
pub struct FailedInstallState {
}

impl FailedInstallState {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[derive(Debug, Clone)]
pub struct FailedInstall;

impl FailedInstall {
    pub fn new() -> FailedInstall {
        FailedInstall
    }
}

impl StatefulWidgetRef for FailedInstall {
    type State = FailedInstallState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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

        let block = Block::bordered()
            .title_bottom("<ESPACE>".to_line().red());

        Paragraph::new("Installation impossible (disque dur corrompu)".to_string().to_line().left_aligned())
            .centered()
            .block(block)
            .red()
            .render(layout2[1], buf);
    }
}

impl Tab for FailedInstall {
    fn handle_input(&self, event: Event, state: &mut Self::State) -> Option<usize> {
        if let Event::Key(KeyEvent { code: KeyCode::Char(' '), kind: KeyEventKind::Press, .. }) = event {
            return Some(1);
        }

        None
    }
}