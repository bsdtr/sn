use std::io;
use std::path::{Path, PathBuf};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use ratatui::widgets::ListState;

use crate::notes::{self, Note};

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    CreateNote {
        input: String,
        error: Option<String>,
    },
}

pub struct App {
    pub notes_dir: PathBuf,
    pub left_width: u16,
    pub notes: Vec<Note>,
    pub selected: usize,
    pub list_state: ListState,
    pub preview_scroll: u16,
    pub mode: Mode,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let notes_dir = notes::notes_dir();
        let left_width = notes::left_panel_width();
        let notes = notes::load_notes(&notes_dir)?;

        Ok(Self {
            notes_dir,
            left_width,
            notes,
            selected: 0,
            list_state: ListState::default(),
            preview_scroll: 0,
            mode: Mode::Normal,
            should_quit: false,
        })
    }

    pub fn reload_notes(&mut self) -> io::Result<()> {
        let count = self.notes.len();
        self.notes = notes::load_notes(&self.notes_dir)?;

        if self.notes.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.notes.len() {
            self.selected = self.notes.len() - 1;
        } else if count != self.notes.len() {
            // keep selection when possible
        }

        self.preview_scroll = 0;
        Ok(())
    }

    fn reload_notes_and_select(&mut self, path: &Path) -> io::Result<()> {
        self.reload_notes()?;
        if let Some(idx) = self.notes.iter().position(|n| n.path == path) {
            self.selected = idx;
            self.list_state.select(Some(idx));
        }
        Ok(())
    }

    pub fn effective_left_width(&self, term_width: u16) -> u16 {
        let mut width = self.left_width.max(20);
        if term_width > 40 {
            width = width.min(term_width - 20);
        }
        width
    }

    pub fn selected_note(&self) -> Option<&Note> {
        self.notes.get(self.selected)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::CreateNote { .. } => self.handle_create_note_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('a') => self.open_create_prompt(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::SHIFT) => self.select_last(),
            KeyCode::Char('g') => self.select_first(),
            KeyCode::Char('G') => self.select_last(),
            KeyCode::Char(']') => self.scroll_preview_down(),
            KeyCode::Char('[') => self.scroll_preview_up(),
            _ => {}
        }
    }

    fn handle_create_note_key(&mut self, key: KeyEvent) {
        let Mode::CreateNote { input, error } = &mut self.mode else {
            return;
        };

        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => {
                let name = input.clone();
                match notes::create_note(&self.notes_dir, &name) {
                    Ok(path) => {
                        let _ = self.reload_notes_and_select(&path);
                        self.mode = Mode::Normal;
                    }
                    Err(err) => {
                        *error = Some(err.to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                input.pop();
                *error = None;
            }
            KeyCode::Char(c) if !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                input.push(c);
                *error = None;
            }
            _ => {}
        }
    }

    fn open_create_prompt(&mut self) {
        self.mode = Mode::CreateNote {
            input: String::new(),
            error: None,
        };
    }

    fn select_next(&mut self) {
        if self.selected + 1 < self.notes.len() {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
            self.preview_scroll = 0;
        }
    }

    fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
            self.preview_scroll = 0;
        }
    }

    fn select_first(&mut self) {
        if !self.notes.is_empty() {
            self.selected = 0;
            self.list_state.select(Some(0));
            self.preview_scroll = 0;
        }
    }

    fn select_last(&mut self) {
        if !self.notes.is_empty() {
            self.selected = self.notes.len() - 1;
            self.list_state.select(Some(self.selected));
            self.preview_scroll = 0;
        }
    }

    fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    pub fn clamp_preview_scroll(&mut self, visible_lines: u16) {
        let Some(note) = self.selected_note() else {
            self.preview_scroll = 0;
            return;
        };

        let total = note.content.lines().count() as u16;
        if total <= visible_lines {
            self.preview_scroll = 0;
        } else {
            let max = total - visible_lines;
            self.preview_scroll = self.preview_scroll.min(max);
        }
    }

    pub fn create_note_input(&self) -> Option<&str> {
        match &self.mode {
            Mode::CreateNote { input, .. } => Some(input.as_str()),
            Mode::Normal => None,
        }
    }

    pub fn create_note_error(&self) -> Option<&str> {
        match &self.mode {
            Mode::CreateNote { error, .. } => error.as_deref(),
            Mode::Normal => None,
        }
    }

    pub fn is_create_prompt_open(&self) -> bool {
        matches!(self.mode, Mode::CreateNote { .. })
    }
}

pub fn poll_event() -> io::Result<Option<Event>> {
    if event::poll(std::time::Duration::from_millis(100))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
