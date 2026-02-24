use crate::diff::selection;
use crate::diff::types::DiffSet;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct App {
    pub diff_set: DiffSet,
    pub current_file: usize,
    pub current_hunk: usize,
    pub show_help: bool,
    pub status_message: Option<String>,
    pub should_quit: bool,
    pub vertical_scroll: u16,
    pub horizontal_scroll: u16,
    pub show_confirmation: bool,
    pub needs_refresh: bool,
}

pub enum AppAction {
    Continue,
    Quit,
    ApplyReversions,
    Refresh,
}

impl App {
    pub fn new(diff_set: DiffSet) -> Self {
        Self {
            diff_set,
            current_file: 0,
            current_hunk: 0,
            show_help: false,
            status_message: None,
            should_quit: false,
            vertical_scroll: 0,
            horizontal_scroll: 0,
            show_confirmation: false,
            needs_refresh: false,
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

        // Confirmation dialog handling
        if self.show_confirmation {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    self.show_confirmation = false;
                    return AppAction::ApplyReversions;
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => {
                    self.show_confirmation = false;
                    self.status_message = Some("Cancelled".to_string());
                    return AppAction::Continue;
                }
                _ => return AppAction::Continue,
            }
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

            // Vertical scroll within hunk (Shift+Up/Down)
            KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.scroll_up();
                AppAction::Continue
            }
            KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.scroll_down();
                AppAction::Continue
            }

            // Horizontal scroll within hunk (Shift+Left/Right)
            KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.scroll_left();
                AppAction::Continue
            }
            KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.scroll_right();
                AppAction::Continue
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
                if let Some(file) = self.diff_set.files.get(self.current_file)
                    && !file.hunks.is_empty()
                {
                    self.current_hunk = file.hunks.len() - 1;
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
            KeyCode::Char('A') => {
                self.select_all_global();
                AppAction::Continue
            }
            KeyCode::Char('N') => {
                self.deselect_all_global();
                AppAction::Continue
            }

            KeyCode::Char('s') => {
                self.split_current_hunk();
                AppAction::Continue
            }

            // Actions
            KeyCode::Enter => {
                let selected = self.diff_set.selected_hunks();
                if selected == 0 {
                    self.status_message = Some("No hunks selected".to_string());
                    AppAction::Continue
                } else {
                    self.show_confirmation = true;
                    self.status_message = None;
                    AppAction::Continue
                }
            }
            KeyCode::Char('r') => {
                self.needs_refresh = true;
                AppAction::Refresh
            }

            _ => AppAction::Continue,
        }
    }

    fn scroll_up(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
    }

    fn scroll_down(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_add(1);
    }

    fn scroll_left(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
    }

    fn scroll_right(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_add(1);
    }

    fn next_hunk(&mut self) {
        if let Some(file) = self.diff_set.files.get(self.current_file) {
            if self.current_hunk + 1 < file.hunks.len() {
                self.current_hunk += 1;
                self.vertical_scroll = 0;
                self.horizontal_scroll = 0;
            } else {
                // Move to next file
                self.next_file();
            }
        }
    }

    fn previous_hunk(&mut self) {
        if self.current_hunk > 0 {
            self.current_hunk -= 1;
            self.vertical_scroll = 0;
            self.horizontal_scroll = 0;
        } else {
            // Move to previous file's last hunk
            self.previous_file();
            if let Some(file) = self.diff_set.files.get(self.current_file)
                && !file.hunks.is_empty()
            {
                self.current_hunk = file.hunks.len() - 1;
                self.vertical_scroll = 0;
                self.horizontal_scroll = 0;
            }
        }
    }

    fn next_file(&mut self) {
        if self.current_file + 1 < self.diff_set.files.len() {
            self.current_file += 1;
            self.current_hunk = 0;
            self.vertical_scroll = 0;
            self.horizontal_scroll = 0;
        }
    }

    fn previous_file(&mut self) {
        if self.current_file > 0 {
            self.current_file -= 1;
            self.current_hunk = 0;
            self.vertical_scroll = 0;
            self.horizontal_scroll = 0;
        }
    }

    fn split_current_hunk(&mut self) {
        if selection::split_hunk(&mut self.diff_set, self.current_file, self.current_hunk) {
            self.status_message = Some("Hunk split into two".to_string());
        } else {
            self.status_message = Some("Cannot split: no context gap between changes".to_string());
        }
    }

    fn toggle_current_hunk(&mut self) {
        selection::toggle_hunk_selection(&mut self.diff_set, self.current_file, self.current_hunk);
        self.status_message = Some(format!("{} hunks selected", self.diff_set.selected_hunks()));
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

    fn select_all_global(&mut self) {
        selection::select_all_global(&mut self.diff_set);
        self.status_message = Some(format!(
            "All {} hunks selected globally",
            self.diff_set.total_hunks()
        ));
    }

    fn deselect_all_global(&mut self) {
        selection::deselect_all_global(&mut self.diff_set);
        self.status_message = Some("All hunks deselected globally".to_string());
    }

    pub fn get_current_file(&self) -> Option<&crate::diff::types::FileDiff> {
        self.diff_set.files.get(self.current_file)
    }

    pub fn get_current_hunk(&self) -> Option<&crate::diff::types::Hunk> {
        self.get_current_file()
            .and_then(|f| f.hunks.get(self.current_hunk))
    }
}
