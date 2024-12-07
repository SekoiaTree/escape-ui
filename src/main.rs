use std::io::Write;
mod email;
mod model;
mod time_trial;
mod password;
mod victory;
mod music;
mod decrypt;
mod successful_install;
mod failed_install;

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use anyhow::Context;
use notify::event::{ModifyKind, RemoveKind, RenameMode};
use notify::EventKind;
use crate::email::{EmailProgram, EmailProgramState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::*;
use ratatui::{crossterm, crossterm::event::{self, KeyCode, KeyEventKind}, DefaultTerminal};
use ratatui::crossterm::event::{Event, KeyModifiers};
use ratatui::crossterm::style::Stylize;
use ratatui::text::ToLine;
use ratatui::widgets::{Paragraph, StatefulWidgetRef};
use crate::decrypt::{Decrypt, DecryptState};
use crate::failed_install::{FailedInstall, FailedInstallState};
use crate::model::Tab;
use crate::music::{MusicPlayer, MusicPlayerState};
use crate::password::{PasswordEntry, PasswordEntryState};
use crate::successful_install::{SuccessfulInstall, SuccessfulInstallState};
use crate::time_trial::{TimeTrial, TimeTrialState};
use crate::victory::{Victory, VictoryState};

#[derive(Debug, Clone)]
enum TabUi {
    Password(PasswordEntry),
    Email(EmailProgram),
    Decrypt(Decrypt),
    Music(MusicPlayer),
    TimeTrial(TimeTrial),
    SuccessfulInstall(SuccessfulInstall),
    FailedInstall(FailedInstall),
    Victory(Victory),
}

impl StatefulWidgetRef for TabUi {
    type State = TabState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match self {
            TabUi::Email(email) => {
                let TabState::Email(state) = state else {
                    panic!("State did not correspond to tab")
                };
                email.render_ref(area, buf, state)
            }
            TabUi::Password(pass) => {
                let TabState::Password(state) = state else {
                    panic!("State did not correspond to tab")
                };
                pass.render_ref(area, buf, state)
            }
            TabUi::TimeTrial(time) => {
                let TabState::TimeTrial(state) = state else {
                    panic!("State did not correspond to tab")
                };
                time.render_ref(area, buf, state)
            }
            TabUi::Victory(victory) => {
                let TabState::Victory(state) = state else {
                    panic!("State did not correspond to tab")
                };
                victory.render_ref(area, buf, state);
            }
            TabUi::Music(music) => {
                let TabState::Music(state) = state else {
                    panic!("State did not correspond to tab")
                };
                music.render_ref(area, buf, state);
            }
            TabUi::Decrypt(decrypt) => {
                let TabState::Decrypt(state) = state else {
                    panic!("State did not correspond to tab")
                };
                decrypt.render_ref(area, buf, state);
            }
            TabUi::SuccessfulInstall(install) => {
                let TabState::SuccessfulInstall(state) = state else {
                    panic!("State did not correspond to tab")
                };
                install.render_ref(area, buf, state);
            }
            TabUi::FailedInstall(install) => {
                let TabState::FailedInstall(state) = state else {
                    panic!("State did not correspond to tab")
                };
                install.render_ref(area, buf, state);
            }
        }
    }
}

impl Tab for TabUi {
    fn handle_input(&self, event: Event, state: &mut Self::State) -> Option<usize> {
        match self {
            TabUi::Email(email) => {
                let TabState::Email(state) = state else {
                    panic!("State did not correspond to tab")
                };
                email.handle_input(event, state)
            }
            TabUi::Password(pass) => {
                let TabState::Password(state) = state else {
                    panic!("State did not correspond to tab")
                };
                pass.handle_input(event, state)
            }
            TabUi::TimeTrial(time) => {
                let TabState::TimeTrial(state) = state else {
                    panic!("State did not correspond to tab")
                };
                time.handle_input(event, state)
            }
            TabUi::Victory(victory) => {
                let TabState::Victory(state) = state else {
                    panic!("State did not correspond to tab")
                };
                victory.handle_input(event, state)
            }
            TabUi::Music(music) => {
                let TabState::Music(state) = state else {
                    panic!("State did not correspond to tab")
                };
                music.handle_input(event, state)
            }
            TabUi::Decrypt(decrypt) => {
                let TabState::Decrypt(state) = state else {
                    panic!("State did not correspond to tab")
                };
                decrypt.handle_input(event, state)
            }
            TabUi::SuccessfulInstall(install) => {
                let TabState::SuccessfulInstall(state) = state else {
                    panic!("State did not correspond to tab")
                };
                install.handle_input(event, state)
            }
            TabUi::FailedInstall(install) => {
                let TabState::FailedInstall(state) = state else {
                    panic!("State did not correspond to tab")
                };
                install.handle_input(event, state)
            }
        }
    }
}

#[derive(Debug, Clone)]
enum TabState {
    Email(EmailProgramState),
    Password(PasswordEntryState),
    Music(MusicPlayerState),
    Decrypt(DecryptState),
    TimeTrial(TimeTrialState),
    SuccessfulInstall(SuccessfulInstallState),
    FailedInstall(FailedInstallState),
    Victory(VictoryState),
}

struct App {
    tabs: Vec<TabUi>,
    current_tab: usize,
    usbs_plugged: [bool; 5],
    watcher: mpsc::Receiver<notify::Result<notify::Event>>,
}

impl StatefulWidget for &App {
    type State = Vec<TabState>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized
    {
        let [window_control] = Layout::horizontal([Constraint::Fill(1)])
            .horizontal_margin(3)
            .vertical_margin(1)
            .areas(area);
        let [warning, count] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
            .horizontal_margin(4)
            .areas(area);

        self.tabs[self.current_tab].render_ref(window_control, buf, &mut state[self.current_tab]);
        let left_span = Span::styled("! AVERTISSEMENT: DISQUE PARTIELLEMENT CORROMPU. CERTAINES DONNÉES PEUVENT ÊTRE PERDUES.", Color::Red);
        let good_usb_count = self.usbs_plugged.iter().filter(|&&x| x).count();
        let right_span = if good_usb_count >= 2 {
            Span::styled(format!("{}/4", good_usb_count-1), Color::Green)
        } else {
            Span::styled("", Style::new())
        };
        Paragraph::new(Line::from(vec![left_span]).left_aligned()).render(warning, buf);
        Paragraph::new(Line::from(vec![right_span]).right_aligned()).render(count, buf);
    }
}

impl App {
    pub fn new(tabs: Vec<TabUi>, watcher: mpsc::Receiver<notify::Result<notify::Event>>) -> Result<Self, ()> {
        if tabs.len() == 0 {
            return Err(());
        }

        Ok(Self {
            tabs,
            current_tab: 0,
            usbs_plugged: [false; 5],
            watcher,
        })
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal, mut states: Vec<TabState>) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| {
                if self.current_tab == self.tabs.len() - 1 {
                    if let TabState::Victory(state) = &mut states[self.current_tab] {
                        Victory::destroy(frame, state);
                        return;
                    }
                }
                frame.render_stateful_widget(&*self, frame.area(), &mut states);
            })?;

            if event::poll(Duration::from_millis(50))? {
                let event = event::read()?;
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q')
                        && key.modifiers == KeyModifiers::ALT {
                        return Ok(());
                    }
                }
                match self.tabs[self.current_tab].handle_input(event, &mut states[self.current_tab]) {
                    Some(new_tab) => {
                        if self.current_tab >= 2 && self.current_tab < 6 {
                            // USB tab redirected, so it's approved!
                            self.usbs_plugged[self.current_tab - 2] = true;
                        }

                        self.current_tab = new_tab;

                        if new_tab == 1 && self.usbs_plugged.iter().all(|&x| x) { // Redirect back to victory
                            self.current_tab = self.tabs.len()-1;
                        }
                    }
                    None => {}
                }
            }

            if let Ok(Ok(event)) = self.watcher.try_recv() {
                if let EventKind::Modify(ModifyKind::Name(RenameMode::Both)) = event.kind {
                    if let Some(x) = event.paths[1].to_str() {
                        if x.ends_with("-ESCAPE") {
                            let index = x.chars().skip("/dev/disk/by-label/".len()).next().unwrap() as u8 - b'1';
                            // Password and email are first, then all 4 USBs
                            if self.current_tab != 0 {
                                self.current_tab = index as usize + 2;
                            }
                        }
                    }
                } else if let EventKind::Remove(_) = event.kind {
                    if let Some(x) = event.paths[0].to_str() {
                        if x.ends_with("-ESCAPE") {
                            self.current_tab = 1.min(self.current_tab);
                        }
                    }
                }
            }
        }
    }
}

fn render(terminal: DefaultTerminal) -> anyhow::Result<()> {
    use notify::{Event, RecursiveMode, Result, Watcher};
    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(Path::new("/dev/disk/by-label"), RecursiveMode::NonRecursive)?;
    let email = EmailProgram::new("./emails".into()).context("./emails folder not found")?;
    let mdp = String::from_utf8(std::fs::read("./password.txt").context("password.txt not found")?)?.trim().to_string();
    let mode = String::from_utf8(std::fs::read("./mode.txt").context("mode.txt not found")?)?;
    let (manager, _backend) = awedio::start()?;

    let ui = if mode.starts_with("disabled") {
        vec![
            TabUi::Password(PasswordEntry::new(mdp.clone())),
            TabUi::Email(email),
            TabUi::FailedInstall(FailedInstall {}),
            TabUi::FailedInstall(FailedInstall {}),
            TabUi::FailedInstall(FailedInstall {}),
            TabUi::FailedInstall(FailedInstall {}),
            TabUi::Victory(Victory {}),
        ]
    } else {
        vec![
            TabUi::Password(PasswordEntry::new(mdp.clone())),
            TabUi::Email(email),
            TabUi::Decrypt(Decrypt {}),
            TabUi::Music(MusicPlayer::new(Path::new("clairdelune.mp3").to_path_buf())),
            TabUi::SuccessfulInstall(SuccessfulInstall {}),
            TabUi::TimeTrial(TimeTrial {}),
            TabUi::Victory(Victory {}),
        ]
    };
    let states = if mode.starts_with("disabled") {
        vec![
            TabState::Password(PasswordEntryState::new()),
            TabState::Email(EmailProgramState::new()),
            TabState::FailedInstall(FailedInstallState::new()),
            TabState::FailedInstall(FailedInstallState::new()),
            TabState::FailedInstall(FailedInstallState::new()),
            TabState::FailedInstall(FailedInstallState::new()),
            TabState::Victory(VictoryState::new()),
        ]
    } else {
        vec![
            TabState::Password(PasswordEntryState::new()),
            TabState::Email(EmailProgramState::new()),
            TabState::Decrypt(DecryptState::new()),
            TabState::Music(MusicPlayerState::new(manager)),
            TabState::SuccessfulInstall(SuccessfulInstallState::new()),
            TabState::TimeTrial(TimeTrialState::Calculations(0, "".into(), false)),
            TabState::Victory(VictoryState::new()),
        ]
    };

    let mut app = App::new(ui, rx).expect("Can't fail creating");
    if mdp.is_empty() {
        app.current_tab = 1;
    }
    if !mode.starts_with("disabled") {
        // Enable victory screen
        app.usbs_plugged[4] = true;
    }

    app.run(terminal, states)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let result = render(terminal);

    ratatui::restore();
    result?;

    Ok(())
}