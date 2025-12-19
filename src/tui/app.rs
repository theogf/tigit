use crate::diff::types::DiffSet;
use crate::diff::selection;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct App {
    pub diff_set: DiffSet,
    pub current_file: usize,
    pub current_hunk: usize,
    pub scroll_offset: usize,
    pub show_help: bool,
    pub status_message: Option<String>,
    pub should_quit: bool,
}

pub enum AppAction {
    Continue,
    Quit,
    ApplyReversions,
}

impl App {
    pub fn new(diff_set: DiffSet) -> Self {
        Self {
            diff_set,
            current_file: 0,
            current_hunk: 0,
            scroll_offset: 0,
            show_help: false,
            status_message: None,
            should_quit: false,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> AppAction {
        // Help panel toggle
        if key.code == KeyCode::Char('?') {
            self.show_help = !self.show_help;
            return AppAction::Continue;
        }

        // If help is showing, any key closes it
        if self.show_help {
            self.show_help = false;
            return AppAction::Continue;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                AppAction::Quit
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                AppAction::Quit
            }

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous_hunk();
                AppAction::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.next_hunk();
                AppAction::Continue
            }
            KeyCode::PageUp => {
                for _ in 0..10 {
                    self.previous_hunk();
                }
                AppAction::Continue
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    self.next_hunk();
                }
                AppAction::Continue
            }
            KeyCode::Home => {
                self.current_hunk = 0;
                AppAction::Continue
            }
            KeyCode::End => {
                if let Some(file) = self.diff_set.files.get(self.current_file) {
                    if !file.hunks.is_empty() {
                        self.current_hunk = file.hunks.len() - 1;
                    }
                }
                AppAction::Continue
            }
            KeyCode::Tab => {
                self.next_file();
                AppAction::Continue
            }
            KeyCode::BackTab => {
                self.previous_file();
                AppAction::Continue
            }

            // Selection
            KeyCode::Char(' ') => {
                self.toggle_current_hunk();
                AppAction::Continue
            }
            KeyCode::Char('a') => {
                self.select_all_in_current_file();
                AppAction::Continue
            }
            KeyCode::Char('n') => {
                self.deselect_all_in_current_file();
                AppAction::Continue
            }

            // Actions
            KeyCode::Enter => AppAction::ApplyReversions,
            KeyCode::Char('r') => {
                self.status_message = Some("Refresh not yet implemented".to_string());
                AppAction::Continue
            }

            _ => AppAction::Continue,
        }
    }

    fn next_hunk(&mut self) {
        if let Some(file) = self.diff_set.files.get(self.current_file) {
            if self.current_hunk + 1 < file.hunks.len() {
                self.current_hunk += 1;
            } else {
                // Move to next file
                self.next_file();
            }
        }
    }

    fn previous_hunk(&mut self) {
        if self.current_hunk > 0 {
            self.current_hunk -= 1;
        } else {
            // Move to previous file's last hunk
            self.previous_file();
            if let Some(file) = self.diff_set.files.get(self.current_file) {
                if !file.hunks.is_empty() {
                    self.current_hunk = file.hunks.len() - 1;
                }
            }
        }
    }

    fn next_file(&mut self) {
        if self.current_file + 1 < self.diff_set.files.len() {
            self.current_file += 1;
            self.current_hunk = 0;
        }
    }

    fn previous_file(&mut self) {
        if self.current_file > 0 {
            self.current_file -= 1;
            self.current_hunk = 0;
        }
    }

    fn toggle_current_hunk(&mut self) {
        selection::toggle_hunk_selection(&mut self.diff_set, self.current_file, self.current_hunk);
        self.status_message = Some(format!(
            "{} hunks selected",
            self.diff_set.selected_hunks()
        ));
    }

    fn select_all_in_current_file(&mut self) {
        if let Some(file) = self.diff_set.files.get_mut(self.current_file) {
            selection::select_all_in_file(file);
            self.status_message = Some("All hunks in file selected".to_string());
        }
    }

    fn deselect_all_in_current_file(&mut self) {
        if let Some(file) = self.diff_set.files.get_mut(self.current_file) {
            selection::deselect_all_in_file(file);
            self.status_message = Some("All hunks in file deselected".to_string());
        }
    }

    pub fn get_current_file(&self) -> Option<&crate::diff::types::FileDiff> {
        self.diff_set.files.get(self.current_file)
    }

    pub fn get_current_hunk(&self) -> Option<&crate::diff::types::Hunk> {
        self.get_current_file()
            .and_then(|f| f.hunks.get(self.current_hunk))
    }
}
