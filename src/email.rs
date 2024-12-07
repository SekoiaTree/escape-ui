use std::cmp::Reverse;
use crate::model::Tab;
use anyhow::{Context, Result};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::prelude::*;
use ratatui::text::ToSpan;
use ratatui::widgets::{Block, Borders, List, ListState, Paragraph, ScrollbarState, StatefulWidgetRef, Wrap};
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct Email {
    // A list of paragraphs. Each paragraph is a list of owned spans.
    body: Vec<Vec<(String, Style)>>,
    from: String,
    to: String,
    cc: String,
    subject: String,
    date: String,
}

impl Email {
    pub fn new_from_file(path: &Path) -> Result<Self> {
        let file = std::fs::read_to_string(path)?;

        let mut lines = file.lines();
        let from = lines.next().context("No `from` line")?.trim_start_matches("from:").trim().to_string();
        let to = lines.next().context("No `to` line")?.trim_start_matches("to:").trim().to_string();
        let cc = lines.next().context("No `cc` line")?.trim_start_matches("cc:").to_string();
        let date = lines.next().context("No `date` line")?.trim_start_matches("date:").trim().to_string();
        let subject = lines.next().context("No `subject` line")?.trim_start_matches("subject:").trim().to_string();

        let mut body = vec![];

        for line in lines.skip_while(|x| x.is_empty()) {
            if line.is_empty() {
                body.push(vec![]);
                continue;
            }

            let mut current_paragraph = vec![];
            let mut current_span = String::new();
            let mut current_style = Style::new();

            let mut cancel_next_delim = false;
            for split in line.split_inclusive(&['*', '\\', '_', '~']) {
                if split.len() == 1 && cancel_next_delim {
                    current_span.push_str(split);
                    cancel_next_delim = false;
                    continue;
                }

                match split.chars().last().unwrap() {
                    '*' => {
                        current_span.push_str(&split[..split.len()-1]);
                        current_paragraph.push((current_span, current_style.clone()));
                        current_span = String::new();
                        current_style.add_modifier.toggle(Modifier::BOLD);
                    }
                    '_' => {
                        current_span.push_str(&split[..split.len()-1]);
                        current_paragraph.push((current_span, current_style.clone()));
                        current_span = String::new();
                        current_style.add_modifier.toggle(Modifier::ITALIC);
                    }
                    '~' => {
                        current_span.push_str(&split[..split.len()-1]);
                        current_paragraph.push((current_span, current_style.clone()));
                        current_span = String::new();
                        match current_style.fg {
                            None => {
                                current_style.fg = Some(Color::Red)
                            },
                            Some(_) => {
                                current_style.fg = None
                            },
                        }
                    }
                    '\\' => {
                        cancel_next_delim = true;
                    }
                    _ => {
                        current_span.push_str(split);
                    }
                }
            }

            current_paragraph.push((current_span, current_style.clone()));
            body.push(current_paragraph);
        }

        Ok(Self {
            body,
            from,
            to,
            cc,
            subject,
            date,
        })
    }

    pub fn get_stub(&self) -> Text {
        Text::from(vec![
            Span::styled(&self.from, Style::new()).into_left_aligned_line(),
            Span::styled(&self.date, Modifier::BOLD).into_left_aligned_line(),
            Span::styled(&self.subject, Style::new()).into_left_aligned_line(),
            Line::raw(""),
        ])
    }
}

impl<'a> Into<Text<'a>> for &'a Email {
    fn into(self) -> Text<'a> {
        Text {
            lines: self.body.iter().map(|line| Line::from_iter(line.iter().map(|span| {
                Span::styled(&span.0, span.1)
            }))).collect(),
            style: Default::default(),
            alignment: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmailProgram {
    emails: Vec<Email>,
}

impl EmailProgram {
    pub fn new(folder_path: PathBuf) -> Result<Self> {
        let mut paths: Vec<_> = std::fs::read_dir(folder_path).unwrap()
            .map(|entry| Ok(entry?))
            .collect::<Result<Vec<DirEntry>>>()?;

        paths.sort_by_key(|dir| dir.path());

        let mut emails = vec![];
        for i in paths {
            let file_type = i.file_type()?;

            if !file_type.is_file() {
                continue;
            }

            if i.path().extension() == Some(std::ffi::OsStr::new("email")) {
                let email = Email::new_from_file(&i.path())?;
                emails.push(email);
            }
        }
        emails.sort_by_key(|email| Reverse(email.date.clone()));

        Ok(EmailProgram { emails })
    }
}

#[derive(Debug, Clone)]
pub struct EmailProgramState {
    list_state: ListState,
    scrollbar_state: ScrollbarState,
}

impl EmailProgramState {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
            scrollbar_state: ScrollbarState::default(),
        }
    }
}

impl Tab for EmailProgram {
    fn handle_input(&self, event: Event, state: &mut Self::State) -> Option<usize> {
        match event {
            Event::Key(event) => {
                if event.kind == KeyEventKind::Press {
                    let selected = state.list_state.selected_mut();
                    let n_emails = self.emails.len();
                    match event.code {
                        KeyCode::Down => {
                            *selected = Some(selected.map(|x| (x+1) % n_emails).unwrap_or(0));
                        },
                        KeyCode::Up => {
                            *selected = Some(selected.map(|x| (x+n_emails - 1) % n_emails).unwrap_or(n_emails-1));
                        },
                        _ => {}
                    }
                }
            }
            Event::Mouse(_) => {}
            _ => {}
        }
        None
    }
}

impl StatefulWidgetRef for EmailProgram {
    type State = EmailProgramState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        fn email_list<'a>(list: &'a [Email], block: Block<'a>) -> impl StatefulWidget<State=ListState> + 'a {
            List::new(
                list.iter()
                    .enumerate()
                    .map(|(i, x)| {
                        x.get_stub()
                    })
            )
                .highlight_style(Style::from((Color::Green, Modifier::BOLD)))
                .block(block)
        }

        fn render_body(area: Rect, buf: &mut Buffer, email: &Email) {
            let [from_date, to, cc, subject, body] = Layout::vertical([
                Constraint::Length(2), // from: + date:
                Constraint::Length(2), // to:
                Constraint::Length(2), // cc:
                Constraint::Length(2), // subject:
                Constraint::Fill(1), // body:
            ]).areas(area);

            let border_set_top = symbols::border::Set {
                top_left: symbols::line::NORMAL.horizontal_down,
                ..symbols::border::Set::default()
            };
            let border_set_mid = symbols::border::Set {
                top_left: symbols::line::NORMAL.vertical_right,
                top_right: symbols::line::NORMAL.vertical_left,
                ..symbols::border::Set::default()
            };
            let border_set_bottom = symbols::border::Set {
                top_left: symbols::line::NORMAL.vertical_right,
                top_right: symbols::line::NORMAL.vertical_left,
                bottom_left: symbols::line::NORMAL.horizontal_up,
                ..symbols::border::Set::default()
            };

            let top_par = Paragraph::new(Line::from(vec![
                "de: ".to_span().dim().bold(),
                email.from.to_span(),
                " le ".to_span().dim().bold(),
                email.date.to_span(),
            ])).block(Block::new()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_set(border_set_top)
                .title("Courriel actuel")
            );

            let mid_block = Block::new()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_set(border_set_mid);

            let to_par = Paragraph::new(Line::from(vec![
                "à: ".to_span().dim().bold(),
                email.to.to_span(),
            ])).block(mid_block.clone());
            let cc_par = Paragraph::new(Line::from(vec![
                "cc: ".to_span().dim().bold(),
                email.cc.to_span(),
            ])).block(mid_block.clone());
            let subject_par = Paragraph::new(Line::from(vec![
                "sujet: ".to_span().dim().bold(),
                email.subject.to_span(),
            ])).block(mid_block.clone());
            
            let body_par = Paragraph::new(email).wrap(Wrap { trim: true }).block(Block::new().borders(Borders::all()).border_set(border_set_bottom));
            
            top_par.render(from_date, buf);
            to_par.render(to, buf);
            cc_par.render(cc, buf);
            subject_par.render(subject, buf);
            body_par.render(body, buf);
        }

        let [left, right] = Layout::horizontal(
            [Constraint::Length(30), Constraint::Fill(1)]
        ).areas(area);

        let left_block = Block::new()
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .title("Courriels");

        let right_border_set = symbols::border::Set {
            top_left: symbols::line::NORMAL.horizontal_down,
            bottom_left: symbols::line::NORMAL.horizontal_up,
            ..symbols::border::PLAIN
        };


        let left_list = email_list(&self.emails, left_block);
        match state.list_state.selected() {
            None => {
                let block = Block::new()
                    .border_set(right_border_set)
                    .borders(Borders::all())
                    .title("Courriel actuel");
                let par = Paragraph::new("No email selected!").green().centered().block(block);
                par.render(right, buf);
            },
            Some(sel) => render_body(right, buf, &self.emails[sel])
        };

        left_list.render(left, buf, &mut state.list_state);
    }
}
