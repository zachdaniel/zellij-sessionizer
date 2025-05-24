use zellij_tile::prelude::*;

use crate::filter;

#[derive(Debug, Default)]
pub struct SessList {
    sessions: Vec<String>,
    cursor: usize,

    search_term: String,
    filtered_sessions: Vec<String>,
}

impl SessList {
    pub fn reset(&mut self) {
        self.sessions.clear();
        self.cursor = 0;
        self.filtered_sessions.clear();
    }

    pub fn update_sessions(&mut self, sessions: Vec<String>) {
        self.sessions = sessions;
        self.filter();
    }

    pub fn handle_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn handle_down(&mut self) {
        if self.cursor < self.filtered_sessions.len().saturating_sub(1) {
            self.cursor += 1;
        }
    }

    pub fn get_selected(&self) -> Option<String> {
        if self.cursor < self.filtered_sessions.len() {
            Some(self.filtered_sessions[self.cursor].clone())
        } else {
            None
        }
    }

    pub fn kill_selected(&mut self) {
        if let Some(selected) = self.get_selected() {
            delete_dead_session(&selected);
        }
    }

    pub fn set_search_term(&mut self, search_term: &str) {
        self.search_term = search_term.to_string();
        self.filter();
    }

    pub fn filter(&mut self) {
        self.filtered_sessions = filter::fuzzy_filter(&self.sessions, self.search_term.as_str());
        // self.cursor = self.filtered_sessions.len().saturating_sub(1);
    }

    pub fn render(&self, rows: usize, _cols: usize) {
        let from = self
            .cursor
            .saturating_sub(rows.saturating_sub(1) / 2)
            .min(self.filtered_sessions.len().saturating_sub(rows));
        let missing_rows = rows.saturating_sub(self.filtered_sessions.len());
        if missing_rows > 0 {
            for _ in 0..missing_rows {
                println!();
            }
        }
        self.filtered_sessions
            .iter()
            .enumerate()
            .skip(from)
            .take(rows)
            .for_each(|(i, dir)| {
                let text = dir.to_string();
                let text_len = text.len();
                let item = Text::new(text);
                let item = match i == self.cursor {
                    true => item.color_range(0, 0..text_len).selected(),
                    false => item,
                };
                print_text(item);
                println!();
            })
    }
}
