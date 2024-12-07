use crate::model::Tab;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Layout, Margin, Rect};
use ratatui::prelude::{Constraint, Direction, Line, Span, Style, Widget};
use ratatui::style::Color;
use ratatui::symbols::{border, Marker};
use ratatui::text::ToLine;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::{Block, Clear, Paragraph, StatefulWidgetRef, WidgetRef};
use std::iter::{once, repeat};

/*
[x+y\*z], [(x+y)\*z], [x+y\*z], [x+y#{sym.div}z],
[x++x], [x\*x], [x+x], [x\*x],
[x++y], [x-y], [x\*y], [x+y],
[x-y+z], [x-(y+z)], [-x+y+z], [x-y+z],
[(x-y)\*z], [x-y\*z], [x-y#{sym.div}z], [(x-y)#{sym.div}z],
[x#{sym.div}y+z], [(x+z)#{sym.div}y], table.cell(colspan: 2)[x#{sym.div}y+z]
 */
const CALCULATIONS : [(&str, i64, Color); 6] = [
    ("77++75", 2, Color::Red),
    ("34-19+26", 41, Color::Blue),
    ("12+16*4", 16, Color::Blue),
    ("(26-24)*6", 22, Color::Green),
    ("40÷12+8", 4, Color::Red),
    ("20++20", 40, Color::Green),
];

#[derive(Debug, Clone)]
pub enum TimeTrialState {
    Calculations(usize, String, bool),
    Connections([(usize, usize); 4], (usize, usize), Option<usize>),
}

const CONNECTION_COLORS : [Color; 4] = [Color::Red, Color::White, Color::Green, Color::Blue];

#[derive(Debug, Clone)]
pub struct TimeTrial;

impl StatefulWidgetRef for TimeTrial {
    type State = TimeTrialState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state {
            TimeTrialState::Calculations(calc, text, error) => {
                let layout_p = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Fill(1),
                        Constraint::Length(34),
                        Constraint::Fill(1),
                    ].into_iter())
                    .split(area);

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Fill(1),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Fill(1),
                    ].into_iter())
                    .split(layout_p[1]);

                let top = Block::bordered()
                    .title("Installation (voir appendice A.1)")
                    .border_set(border::PLAIN);
                let top = top.style(CALCULATIONS[*calc].2);
                let bot = Block::bordered()
                    .title("Résultat")
                    .border_set(border::PLAIN);
                let bot = if *error {
                    bot.style(Color::Red)
                } else {
                    bot.style(Color::Reset)
                };

                Paragraph::new(CALCULATIONS[*calc].0.to_line().style(CALCULATIONS[*calc].2).left_aligned())
                    .centered()
                    .block(top)
                    .render(layout[1], buf);
                Paragraph::new(text.to_line().style(if *error { Color::Red } else { Color::Reset }).left_aligned())
                    .centered()
                    .block(bot)
                    .render(layout[2], buf);
            }
            TimeTrialState::Connections(targets, cursor, selected) => {
                let layout_p = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Fill(1),
                        Constraint::Length(78),
                        Constraint::Fill(1),
                    ].into_iter())
                    .split(area);

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Fill(1),
                        Constraint::Length(16),
                        Constraint::Fill(1),
                    ].into_iter())
                    .split(layout_p[1]);

                let block = Block::new();

                let top = vec![
                    Span::from("┌─────────"),
                    Span::styled("┬", Style::default().fg(CONNECTION_COLORS[0])),
                    Span::from("────────────────"),
                    Span::styled("┬", Style::default().fg(CONNECTION_COLORS[1])),
                    Span::from("─────────────────────────"),
                    Span::styled("┬", Style::default().fg(CONNECTION_COLORS[2])),
                    Span::from("───────────────────"),
                    Span::styled("┬", Style::default().fg(CONNECTION_COLORS[3])),
                    Span::from("───┐"),
                ];
                let mid = "│                                                                            │";
                let bot = "└────────────────┴─────────────────────┴────────────────────┴──────────────┴─┘";
                let par = Paragraph::new(Vec::from_iter(
                    once(Line::from(top))
                        .chain(repeat(mid).map(Line::from).take(14))
                        .chain(once(Line::from(bot)))
                ));
                par.block(block).render_ref(layout[1], buf);
                let canvas = Canvas::default()
                    .x_bounds([1.0, 77.0])
                    .y_bounds([1.0, 15.0])
                    .marker(Marker::HalfBlock)
                    .paint(|ctx| {
                        use ratatui::widgets::canvas;
                        const BASE_X : [f64; 4] = [11.0, 28.0, 54.0, 74.0];
                        for (index, pos) in targets.iter().enumerate() {
                            ctx.draw(&canvas::Line {
                                x1: BASE_X[index],
                                y1: 16.0-1.0,
                                x2: pos.0 as f64,
                                y2: 16.0-pos.1 as f64,
                                color: CONNECTION_COLORS[index],
                            });
                        }

                        ctx.print(cursor.0 as f64, 16.0-cursor.1 as f64, "X".to_line());
                    });
                Clear::default().render_ref(layout[1].inner(Margin::new(1, 1)), buf);
                canvas.render_ref(layout[1].inner(Margin::new(1, 1)), buf);
            }
        }
    }
}

impl Tab for TimeTrial {
    fn handle_input(&self, event: Event, state: &mut Self::State) -> Option<usize> {
        match state {
            TimeTrialState::Calculations(calc, text, err) => {
                if let Event::Key(KeyEvent { code: KeyCode::Char(c), kind: KeyEventKind::Press, .. }) = event {
                    if text.len() < 20 {
                        text.push(c);
                        *err = false;
                    }
                } else if let Event::Key(KeyEvent { code: KeyCode::Backspace, kind: KeyEventKind::Press, .. }) = event {
                    text.pop();
                    *err = false;
                } else if let Event::Key(KeyEvent { code: KeyCode::Enter, kind: KeyEventKind::Press, .. }) = event {
                    if *text == CALCULATIONS[*calc].1.to_string() {
                        if *calc == CALCULATIONS.len() - 1 {
                            *state = TimeTrialState::Connections([
                                (11, 2),
                                (28, 2),
                                (54, 2),
                                (74, 2),
                            ], (10, 10), None);
                        } else {
                            *calc += 1;
                            text.clear();
                        }
                    } else {
                        *err = true;
                    }
                }
                None
            }
            TimeTrialState::Connections(pos, cursor, selected) => {
                if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) = event {
                    match code {
                        KeyCode::Left => {
                            if cursor.0 > 1 {
                                cursor.0 -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if cursor.0 < 77 {
                                cursor.0 += 1;
                            }
                        }
                        KeyCode::Up => {
                            if cursor.1 > 1 {
                                cursor.1 -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if cursor.1 < 15 {
                                cursor.1 += 1;
                            }
                        }
                        KeyCode::Enter => {
                            // If selected, drop
                            // If not selected, select
                            *selected = match selected {
                                Some(_) => {
                                    const SOLUTION: [(usize, usize); 4] = [
                                        (76, 15),
                                        (61, 15),
                                        (18, 15),
                                        (40, 15),
                                    ];
                                    if *pos == SOLUTION {
                                        return Some(1);
                                    }

                                    None
                                },
                                None => {
                                    // Figure out if there's a pos in pos that matches cursor
                                    match pos.iter().position(|x| *x == *cursor) {
                                        Some(index) => Some(index),
                                        None => None,
                                    }
                                },
                            }
                        }
                        _ => {}
                    }
                    // If something is selected, move it to the cursor:
                    if let Some(index) = selected {
                        pos[*index] = *cursor;
                    }
                }
                None
            }
        }
    }
}