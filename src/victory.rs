use crate::model::Tab;
use rand::{Rng, SeedableRng};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::{Flex, Rect};
use ratatui::prelude::{Color, Layout, Text, Widget};
use ratatui::style::Style;
use ratatui::widgets::StatefulWidgetRef;
use ratatui::{crossterm, Frame};

#[derive(Debug, Clone)]
pub struct VictoryState {
    // Each vec is 1 column
    grid: Vec<Vec<char>>,
    init_frame: usize,
    matrix_done_frame: usize,
}

impl VictoryState {
    pub fn new() -> Self {
        Self {
            grid: vec![vec![]; 100],
            init_frame: usize::MAX,
            matrix_done_frame: usize::MAX,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Victory;

impl StatefulWidgetRef for Victory {
    type State = VictoryState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {


    }
}

impl Tab for Victory {
    fn handle_input(&self, event: Event, state: &mut Self::State) -> Option<usize> {
        None
    }
}

/// delay the start of the animation so it doesn't start immediately
const DELAY: usize = 10;

impl Victory {
    pub fn destroy(frame: &mut Frame, state: &mut VictoryState) {
        // We want a matrix effect. To do this, we use a big grid of text, each column having a "depth"
        // Everything ticks down at a random rate, we tick every 100ms
        let area = frame.area();
        if state.init_frame == usize::MAX {
            state.init_frame = frame.count();
        }

        let curr_frame = frame.count() - state.init_frame;
        let frame_count = curr_frame.saturating_sub(DELAY);
        if frame_count == 0 {
            return;
        }

        if state.grid.len() != area.width as usize {
            state.grid.resize(area.width as usize, vec![]);
        }
        const POSSIBLE_CHARS : [char; 92] = [
            'ｦ','ｧ','ｨ','ｩ','ｪ','ｫ','ｬ','ｭ','ｮ','ｯ','ｰ','ｱ','ｲ','ｳ','ｴ','ｵ','ｶ','ｷ','ｸ','ｹ','ｺ','ｻ','ｼ','ｽ','ｾ','ｿ','ﾀ','ﾁ','ﾂ','ﾃ','ﾄ','ﾅ','ﾆ','ﾇ','ﾈ','ﾉ','ﾊ','ﾋ','ﾌ','ﾍ','ﾎ','ﾏ','ﾐ','ﾑ','ﾒ','ﾓ','ﾔ','ﾕ','ﾖ','ﾗ','ﾘ','ﾙ','ﾚ','ﾛ','ﾜ','ﾝ',
            'A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
            '0','1','2','3','4','5','6','7','8','9',
        ];
        for i in &mut state.grid {
            if i.len() < area.height as usize && rand::random::<f32>() <= 0.75 {
                i.push(POSSIBLE_CHARS[rand::random::<usize>() % POSSIBLE_CHARS.len()]);
            }
        }

        let width = area.width as usize;
        let height = area.height as usize;
        let content = &mut frame.buffer_mut().content;

        for (col, i) in state.grid.iter().enumerate() {
            for (row, c) in i.iter().enumerate() {
                content[row * width + col].set_char(*c);
                content[row * width + col].set_fg(crossterm::style::Color::DarkGreen.into());
            }
            if i.len() > 0 && i.len() != height {
                content[(i.len()-1) * width + col].set_fg(crossterm::style::Color::Green.into());
            }
        }

        if !state.grid.iter().all(|i| i.len() == height) {
            return;
        }

        if state.matrix_done_frame == usize::MAX {
            state.matrix_done_frame = curr_frame;
        }

        const VICTORY_TEXT: &str ="
    ██    ██ ██  ██████ ████████  ██████  ██ ██████  ███████ ██
    ██    ██ ██ ██         ██    ██    ██ ██ ██   ██ ██      ██
    ██    ██ ██ ██         ██    ██    ██ ██ ██████  █████   ██
     ██  ██  ██ ██         ██    ██    ██ ██ ██   ██ ██
      ████   ██  ██████    ██     ██████  ██ ██   ██ ███████ ██


 ██████  ██████  ██████  ███████        ██████   ██████  ██   ██ ██████
██      ██    ██ ██   ██ ██      ██          ██ ██       ██   ██      ██
██      ██    ██ ██   ██ █████           █████  ███████  ███████  █████
██      ██    ██ ██   ██ ██      ██          ██ ██    ██      ██ ██
 ██████  ██████  ██████  ███████        ██████   ██████       ██ ███████
";

        let logo_text = Text::styled(VICTORY_TEXT, Style::default().fg(Color::Green));
        let area = centered_rect(area, logo_text.width() as u16, logo_text.height() as u16);

        let mask_buf = &mut Buffer::empty(area);
        logo_text.render(area, mask_buf);

        let sub_frame = curr_frame - state.matrix_done_frame;

        let percentage = (sub_frame as f64 / 30.0).clamp(0.0, 1.1);

        // Use the same RNG every frame
        let mut random = rand::rngs::SmallRng::seed_from_u64(42);
        for row in area.rows() {
            for col in row.columns() {
                if random.gen::<f64>() <= percentage {
                    let cell = &mut frame.buffer_mut()[(col.x, col.y)];
                    let mask_cell = &mut mask_buf[(col.x, col.y)];

                    if mask_cell.symbol() == " " {
                        continue;
                    }
                    cell.set_symbol(mask_cell.symbol());

                    cell.set_style(Style::new().fg(Color::Green));
                }
            }
        }
    }
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let horizontal = Layout::horizontal([width]).flex(Flex::Center);
    let vertical = Layout::vertical([height]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}