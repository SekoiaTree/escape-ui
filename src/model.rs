use ratatui::crossterm::event::Event;
use ratatui::widgets::StatefulWidgetRef;

pub trait Tab: StatefulWidgetRef {
    fn handle_input(&self, event: Event, state: &mut Self::State) -> Option<usize>;
}