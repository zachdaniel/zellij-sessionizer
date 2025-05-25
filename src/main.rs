use zellij_tile::prelude::*;

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use config::Config;

mod config;
mod dirlist;
mod filter;
mod sesslist;
mod textinput;
use dirlist::DirList;
use sesslist::{SessList, Session};
use textinput::TextInput;

const ROOT: &str = "/host";

#[derive(Debug)]
enum Screen {
    SearchDirs,
    SearchSessions,
}

impl Default for Screen {
    fn default() -> Self {
        return Screen::SearchDirs;
    }
}

#[derive(Debug, Default)]
struct State {
    dirlist: DirList,
    sesslist: SessList,
    cwd: PathBuf,
    textinput: TextInput,
    current_session: String,
    screen: Screen,
    textinput_dump: String,

    config: Config,
    debug: String,
}

register_plugin!(State);

impl State {
    fn change_root(&mut self, path: &Path) -> PathBuf {
        self.cwd.join(path.strip_prefix(ROOT).unwrap())
    }

    fn switch_session_with_cwd(&self, dir: &Path) -> Result<(), String> {
        let session_name = dir.file_name().unwrap().to_str().unwrap();
        let cwd = dir.to_path_buf();
        let host_layout_path = PathBuf::from(ROOT)
            .join(dir.strip_prefix("/").unwrap())
            .join("layout.kdl");
        let layout = if host_layout_path.exists() {
            LayoutInfo::File(host_layout_path.to_str().unwrap().into())
        } else {
            self.config.layout.clone()
        };
        // Switch session will panic if the session is the current session
        if session_name != self.current_session {
            switch_session_with_layout(Some(session_name), layout, Some(cwd));
        }
        Ok(())
    }

    fn make_dirlist(&mut self, paths: &[(PathBuf, Option<FileMetadata>)]) -> Vec<String> {
        paths
            .iter()
            .filter(|(p, _)| p.is_dir() && !is_hidden(p))
            .map(|(p, _)| {
                if p.starts_with(ROOT) {
                    self.change_root(p)
                } else {
                    p.to_path_buf()
                }
            })
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.cwd = get_plugin_ids().initial_cwd;
        self.config = Config::from(configuration);
        request_permission(&[
            PermissionType::RunCommands,
            PermissionType::ChangeApplicationState,
            PermissionType::ReadApplicationState,
        ]);
        subscribe(&[
            EventType::Key,
            EventType::FileSystemUpdate,
            EventType::SessionUpdate,
        ]);
        self.dirlist.reset();
        self.sesslist.reset();
        self.textinput.reset();
        let host = PathBuf::from(ROOT);
        for dir in &self.config.dirs {
            let relative_path = match dir.strip_prefix(self.cwd.as_path()) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let host_path = host.join(relative_path);
            scan_host_folder(&host_path);
        }
        self.screen = Screen::SearchDirs;
        self.textinput_dump = "".to_string();
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::FileSystemUpdate(paths) => {
                let dirs = self.make_dirlist(&paths);
                self.dirlist.update_dirs(dirs);
                should_render = true;
            }
            Event::SessionUpdate(sessions, resurrectables) => {
                //TODO: I may want to handle this inside the sess list
                //and also set the cursor always to the current session
                let alive_sessions = sessions.into_iter().map(|s| {
                    if s.is_current_session {
                        self.current_session = s.name.clone();
                        Session {
                            name: s.name,
                            icon: " ".to_string(),
                        }
                    } else {
                        Session {
                            name: s.name,
                            icon: " ".to_string(),
                        }
                    }
                });
                let resurrectable_sessions = resurrectables.into_iter().map(|(name, _)| Session {
                    name,
                    icon: "󰤄".to_string(),
                });
                self.sesslist
                    .update_sessions(alive_sessions.chain(resurrectable_sessions).collect());
                should_render = true;
            }
            Event::Key(key) => {
                should_render = true;
                match key {
                    KeyWithModifier {
                        bare_key: BareKey::Tab,
                        key_modifiers: _,
                    } => {
                        self.textinput_dump =
                            self.textinput.replace_text(self.textinput_dump.as_str());
                        self.screen = match self.screen {
                            Screen::SearchDirs => Screen::SearchSessions,
                            Screen::SearchSessions => Screen::SearchDirs,
                        }
                    }
                    KeyWithModifier {
                        bare_key: BareKey::Esc,
                        key_modifiers: _,
                    } => {
                        close_self();
                    }
                    KeyWithModifier {
                        bare_key: BareKey::Char('n'),
                        key_modifiers: km,
                    } if km.contains(&KeyModifier::Ctrl) => match self.screen {
                        Screen::SearchDirs => self.dirlist.handle_down(),
                        Screen::SearchSessions => self.sesslist.handle_down(),
                    },
                    KeyWithModifier {
                        bare_key: BareKey::Char('p'),
                        key_modifiers: km,
                    } if km.contains(&KeyModifier::Ctrl) => match self.screen {
                        Screen::SearchDirs => self.dirlist.handle_up(),
                        Screen::SearchSessions => self.sesslist.handle_up(),
                    },
                    KeyWithModifier {
                        bare_key: BareKey::Char('x'),
                        key_modifiers: km,
                    } if km.contains(&KeyModifier::Ctrl) => match self.screen {
                        Screen::SearchDirs => {}
                        Screen::SearchSessions => {
                            self.sesslist.kill_selected();
                        }
                    },
                    KeyWithModifier {
                        bare_key: BareKey::Enter,
                        key_modifiers: _,
                    } => match self.screen {
                        Screen::SearchDirs => {
                            if let Some(selected) = self.dirlist.get_selected() {
                                let _ = self.switch_session_with_cwd(Path::new(&selected));
                                close_self();
                            }
                        }
                        Screen::SearchSessions => {
                            if let Some(selected) = self.sesslist.get_selected() {
                                let _ = switch_session(Some(&selected));
                                close_self();
                            }
                        }
                    },
                    KeyWithModifier {
                        bare_key: BareKey::Backspace,
                        key_modifiers: _,
                    } => {
                        self.textinput.handle_backspace();
                        match self.screen {
                            Screen::SearchDirs => self
                                .dirlist
                                .set_search_term(self.textinput.get_text().as_str()),
                            Screen::SearchSessions => self
                                .sesslist
                                .set_search_term(self.textinput.get_text().as_str()),
                        }
                    }
                    KeyWithModifier {
                        bare_key: BareKey::Char(c),
                        key_modifiers: _,
                    } => {
                        self.textinput.handle_char(c);
                        match self.screen {
                            Screen::SearchDirs => self
                                .dirlist
                                .set_search_term(self.textinput.get_text().as_str()),
                            Screen::SearchSessions => self
                                .sesslist
                                .set_search_term(self.textinput.get_text().as_str()),
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        };
        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        println!();
        match self.screen {
            Screen::SearchDirs => self.dirlist.render(rows.saturating_sub(4), cols),
            Screen::SearchSessions => self.sesslist.render(rows.saturating_sub(4), cols),
        }
        println!();
        self.textinput.render(rows, cols);
        println!();
        if !self.debug.is_empty() {
            println!();
            println!("{}", self.debug);
        }
    }
}

fn is_hidden(path: &Path) -> bool {
    const WHITELIST: [&str; 1] = [".config"];

    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.starts_with('.') && !WHITELIST.contains(&s))
        .unwrap_or(false)
}
