//! The application state as plain data: no terminal, no store, no clock.
//! Keys go in, effects come out, and every transition is testable.

use std::time::{Duration, Instant};

use nib_core::{Hit, Paste, Summary};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Home,
    Add,
    Search,
    Recent,
    Preview,
}

#[derive(Debug, PartialEq)]
pub enum Effect {
    Save(String),
    Search(String, u64),
    Open(String),
    Copy(String),
    Reveal(String, String),
    Quit,
}

pub struct Toast {
    pub text: String,
    pub error: bool,
    until: Instant,
}

pub struct Preview {
    pub id: String,
    pub body: Option<String>,
    pub sealed: bool,
    pub pass: String,
    pub typing: bool,
    pub error: Option<String>,
    pub scroll: u16,
}

pub struct App {
    pub screen: Screen,
    pub menu: usize,
    pub recent: Vec<Summary>,
    pub list: usize,
    pub editor: TextArea<'static>,
    pub discard: bool,
    pub query: String,
    pub hits: Vec<Hit>,
    pub preview: Option<Preview>,
    pub toast: Option<Toast>,
    pub back: Screen,
    pub store: String,
    pub seq: u64,
}

pub const MENU: [(&str, &str); 3] = [
    ("Add a paste", "write it, Ctrl+S saves"),
    ("Find a paste", "search the text, pick a hit"),
    ("Recent pastes", "the last ten, open one"),
];

impl App {
    pub fn new(store: String) -> App {
        App {
            screen: Screen::Home,
            menu: 0,
            recent: Vec::new(),
            list: 0,
            editor: new_editor(),
            discard: false,
            query: String::new(),
            hits: Vec::new(),
            preview: None,
            toast: None,
            back: Screen::Home,
            store,
            seq: 0,
        }
    }

    pub fn note(&mut self, text: impl Into<String>, error: bool) {
        self.toast = Some(Toast {
            text: text.into(),
            error,
            until: Instant::now() + Duration::from_millis(2600),
        });
    }

    pub fn toast(&self) -> Option<(&str, bool)> {
        let now = Instant::now();
        self.toast
            .as_ref()
            .filter(|t| t.until > now)
            .map(|t| (t.text.as_str(), t.error))
    }

    pub fn refresh(&mut self, recent: Vec<Summary>) {
        self.recent = recent;
        self.list = self.list.min(self.recent.len().saturating_sub(1));
    }

    pub fn draft(&self) -> String {
        self.editor.lines().join("\n")
    }

    pub fn reset_editor(&mut self) {
        self.editor = new_editor();
    }

    pub fn saved(&mut self, id: String) {
        self.reset_editor();
        self.discard = false;
        self.screen = Screen::Home;
        self.note(format!("saved {id}"), false);
    }

    pub fn save_failed(&mut self, message: String) {
        self.note(message, true);
    }

    pub fn search_done(&mut self, seq: u64, hits: Vec<Hit>) {
        if seq != self.seq {
            return;
        }
        self.hits = hits;
        self.list = 0;
    }

    pub fn open_preview(&mut self, id: String, back: Screen) {
        self.preview = Some(Preview {
            id,
            body: None,
            sealed: false,
            pass: String::new(),
            typing: false,
            error: None,
            scroll: 0,
        });
        self.back = back;
        self.screen = Screen::Preview;
    }

    /// Called once the store has answered for the paste behind `Open`.
    pub fn loaded(&mut self, paste: Paste) {
        let Some(preview) = self.preview.as_mut() else {
            return;
        };
        if preview.id != paste.id {
            return;
        }
        preview.sealed = paste.enc;
        preview.body = if paste.enc { None } else { Some(paste.body) };
        if paste.enc {
            preview.typing = true;
        }
    }

    pub fn revealed(&mut self, text: String) {
        if let Some(preview) = self.preview.as_mut() {
            preview.body = Some(text);
            preview.error = None;
            preview.typing = false;
            preview.scroll = 0;
        }
        self.note("unlocked", false);
    }

    pub fn reveal_failed(&mut self, message: String) {
        if let Some(preview) = self.preview.as_mut() {
            preview.error = Some(message);
            preview.pass.clear();
            preview.typing = true;
        }
    }

    pub fn handle(&mut self, key: KeyEvent) -> Option<Effect> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Some(Effect::Quit);
        }
        match self.screen {
            Screen::Home => self.home(key),
            Screen::Add => self.add(key),
            Screen::Search => self.search(key),
            Screen::Recent => self.recent(key),
            Screen::Preview => self.preview(key),
        }
    }

    fn home(&mut self, key: KeyEvent) -> Option<Effect> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.menu = self.menu.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.menu = (self.menu + 1).min(MENU.len() - 1),
            KeyCode::Enter => return self.open_menu(),
            KeyCode::Char('n') => {
                self.menu = 0;
                return self.open_menu();
            }
            KeyCode::Char('/') => {
                self.menu = 1;
                return self.open_menu();
            }
            KeyCode::Char('q') => return Some(Effect::Quit),
            _ => {}
        }
        None
    }

    fn open_menu(&mut self) -> Option<Effect> {
        match self.menu {
            0 => {
                self.reset_editor();
                self.discard = false;
                self.screen = Screen::Add;
            }
            1 => {
                self.query.clear();
                self.hits.clear();
                self.list = 0;
                self.screen = Screen::Search;
            }
            _ => {
                self.list = 0;
                self.screen = Screen::Recent;
            }
        }
        None
    }

    fn add(&mut self, key: KeyEvent) -> Option<Effect> {
        if self.discard {
            self.discard = false;
            if matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y')) {
                self.reset_editor();
                self.screen = Screen::Home;
            }
            return None;
        }
        match key.code {
            KeyCode::Esc => {
                if self.draft().trim().is_empty() {
                    self.reset_editor();
                    self.screen = Screen::Home;
                } else {
                    self.discard = true;
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let body = self.draft();
                if body.trim().is_empty() {
                    self.note("nothing to store", true);
                    return None;
                }
                return Some(Effect::Save(body));
            }
            _ => {
                self.editor.input(key);
            }
        }
        None
    }

    fn search(&mut self, key: KeyEvent) -> Option<Effect> {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Home;
                return None;
            }
            KeyCode::Up => self.list = self.list.saturating_sub(1),
            KeyCode::Down => {
                if !self.hits.is_empty() {
                    self.list = (self.list + 1).min(self.hits.len() - 1);
                }
            }
            KeyCode::Enter => {
                if let Some(hit) = self.hits.get(self.list) {
                    let id = hit.id.clone();
                    self.open_preview(id, Screen::Search);
                    return Some(Effect::Open(self.preview.as_ref().unwrap().id.clone()));
                }
                return None;
            }
            KeyCode::Char(c) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.query.push(c);
                }
            }
            KeyCode::Backspace => {
                self.query.pop();
            }
            _ => return None,
        }
        self.seq += 1;
        Some(Effect::Search(self.query.clone(), self.seq))
    }

    fn recent(&mut self, key: KeyEvent) -> Option<Effect> {
        match key.code {
            KeyCode::Esc => self.screen = Screen::Home,
            KeyCode::Up | KeyCode::Char('k') => self.list = self.list.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.recent.is_empty() {
                    self.list = (self.list + 1).min(self.recent.len() - 1);
                }
            }
            KeyCode::Enter => {
                if let Some(item) = self.recent.get(self.list) {
                    let id = item.id.clone();
                    self.open_preview(id.clone(), Screen::Recent);
                    return Some(Effect::Open(id));
                }
            }
            _ => {}
        }
        None
    }

    fn preview(&mut self, key: KeyEvent) -> Option<Effect> {
        let Some(preview) = self.preview.as_mut() else {
            self.screen = Screen::Home;
            return None;
        };

        if preview.typing {
            match key.code {
                KeyCode::Esc => preview.typing = false,
                KeyCode::Enter => {
                    let (id, pass) = (preview.id.clone(), preview.pass.clone());
                    return Some(Effect::Reveal(id, pass));
                }
                KeyCode::Backspace => {
                    preview.pass.pop();
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    preview.pass.push(c);
                }
                _ => {}
            }
            return None;
        }

        match key.code {
            KeyCode::Esc => {
                self.preview = None;
                self.screen = self.back;
            }
            KeyCode::Enter => {
                if let Some(body) = preview.body.clone() {
                    return Some(Effect::Copy(body));
                }
                if preview.sealed {
                    preview.typing = true;
                }
            }
            KeyCode::Char('u') if preview.sealed && preview.body.is_none() => {
                preview.typing = true;
            }
            KeyCode::Up => preview.scroll = preview.scroll.saturating_sub(1),
            KeyCode::Down => preview.scroll = preview.scroll.saturating_add(1),
            KeyCode::PageUp => preview.scroll = preview.scroll.saturating_sub(10),
            KeyCode::PageDown => preview.scroll = preview.scroll.saturating_add(10),
            _ => {}
        }
        None
    }
}

fn new_editor() -> TextArea<'static> {
    let mut editor = TextArea::default();
    editor.set_placeholder_text("type or paste · Ctrl+S saves · Esc cancels");
    editor.set_block(
        ratatui::widgets::Block::default().padding(ratatui::widgets::Padding::new(0, 0, 0, 0)),
    );
    editor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::from(code)
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    fn app() -> App {
        let mut app = App::new("C:\\nib.db".into());
        app.refresh(vec![summary("k3x9", "first line")]);
        app
    }

    fn summary(id: &str, name: &str) -> Summary {
        Summary {
            id: id.into(),
            name: name.into(),
            size: 10,
            created_at: 0,
            enc: false,
        }
    }

    #[test]
    fn arrows_move_and_clamp_the_menu() {
        let mut app = app();
        app.handle(key(KeyCode::Down));
        app.handle(key(KeyCode::Down));
        app.handle(key(KeyCode::Down));
        assert_eq!(app.menu, MENU.len() - 1);
        app.handle(key(KeyCode::Up));
        assert_eq!(app.menu, MENU.len() - 2);
    }

    #[test]
    fn jk_move_the_menu_too() {
        let mut app = app();
        app.handle(key(KeyCode::Char('j')));
        assert_eq!(app.menu, 1);
        app.handle(key(KeyCode::Char('k')));
        assert_eq!(app.menu, 0);
    }

    #[test]
    fn enter_opens_the_selected_screen() {
        let mut app = app();
        app.handle(key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Add);
        assert!(app.draft().is_empty());
    }

    #[test]
    fn n_and_slash_jump_to_their_screens() {
        let mut from_n = app();
        from_n.handle(key(KeyCode::Char('n')));
        assert_eq!(from_n.screen, Screen::Add);

        let mut from_slash = app();
        from_slash.handle(key(KeyCode::Char('/')));
        assert_eq!(from_slash.screen, Screen::Search);
    }

    #[test]
    fn typing_goes_into_the_editor_and_saves_exactly() {
        let mut app = app();
        app.handle(key(KeyCode::Enter));
        for c in "hello".chars() {
            app.handle(KeyEvent::from(KeyCode::Char(c)));
        }
        app.handle(key(KeyCode::Enter));
        assert_eq!(app.draft(), "hello\n");
        assert_eq!(app.handle(ctrl('s')), Some(Effect::Save("hello\n".into())));
    }

    #[test]
    fn empty_save_is_refused_and_the_draft_kept() {
        let mut app = app();
        app.handle(key(KeyCode::Enter));
        assert_eq!(app.handle(ctrl('s')), None);
        assert!(app.toast().is_some());
        assert_eq!(app.screen, Screen::Add);
    }

    #[test]
    fn esc_on_a_draft_asks_once() {
        let mut app = app();
        app.handle(key(KeyCode::Enter));
        app.handle(key(KeyCode::Char('x')));
        app.handle(key(KeyCode::Esc));
        assert!(app.discard);
        assert_eq!(app.screen, Screen::Add);
        app.handle(key(KeyCode::Char('n')));
        assert_eq!(app.screen, Screen::Add);
        assert_eq!(app.draft(), "x");
        app.handle(key(KeyCode::Esc));
        app.handle(key(KeyCode::Char('y')));
        assert_eq!(app.screen, Screen::Home);
        assert!(app.draft().is_empty());
    }

    #[test]
    fn esc_on_an_empty_draft_goes_home() {
        let mut app = app();
        app.handle(key(KeyCode::Enter));
        app.handle(key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Home);
    }

    #[test]
    fn q_is_a_letter_inside_the_editor() {
        let mut app = app();
        app.handle(key(KeyCode::Enter));
        app.handle(key(KeyCode::Char('q')));
        assert_eq!(app.screen, Screen::Add);
        assert!(app.draft().contains('q'));
    }

    #[test]
    fn q_quits_from_home_only() {
        let mut app = app();
        assert_eq!(app.handle(key(KeyCode::Char('q'))), Some(Effect::Quit));
    }

    #[test]
    fn search_emits_an_effect_per_keystroke_with_an_increasing_sequence() {
        let mut app = app();
        app.handle(key(KeyCode::Char('/')));
        let first = app.handle(key(KeyCode::Char('a')));
        let second = app.handle(key(KeyCode::Char('b')));
        assert_eq!(first, Some(Effect::Search("a".into(), 1)));
        assert_eq!(second, Some(Effect::Search("ab".into(), 2)));
    }

    #[test]
    fn a_stale_search_response_is_ignored() {
        let mut app = app();
        app.handle(key(KeyCode::Char('/')));
        app.handle(key(KeyCode::Char('a')));
        app.handle(key(KeyCode::Char('b')));
        app.search_done(1, vec![]);
        assert!(app.hits.is_empty());
        app.search_done(app.seq, vec![hit("k3x9")]);
        assert_eq!(app.hits.len(), 1);
    }

    fn hit(id: &str) -> Hit {
        Hit {
            id: id.into(),
            name: "x".into(),
            snippet: "x".into(),
        }
    }

    #[test]
    fn search_enter_opens_a_preview_and_asks_for_the_paste() {
        let mut app = app();
        app.handle(key(KeyCode::Char('/')));
        app.search_done(app.seq, vec![hit("k3x9")]);
        let effect = app.handle(key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Preview);
        assert_eq!(effect, Some(Effect::Open("k3x9".into())));
    }

    #[test]
    fn preview_esc_returns_to_where_it_came_from() {
        let mut app = app();
        app.handle(key(KeyCode::Char('/')));
        app.search_done(app.seq, vec![hit("k3x9")]);
        app.handle(key(KeyCode::Enter));
        app.handle(key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Search);
    }

    #[test]
    fn recent_enter_opens_a_preview() {
        let mut app = app();
        app.handle(key(KeyCode::Down));
        app.handle(key(KeyCode::Down));
        app.handle(key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Recent);
        let effect = app.handle(key(KeyCode::Enter));
        assert_eq!(effect, Some(Effect::Open("k3x9".into())));
        assert_eq!(app.screen, Screen::Preview);
    }

    #[test]
    fn a_loaded_plain_paste_copies_on_enter() {
        let mut app = app();
        app.open_preview("k3x9".into(), Screen::Home);
        app.loaded(Paste {
            id: "k3x9".into(),
            body: "body".into(),
            enc: false,
            label: None,
        });
        assert_eq!(
            app.handle(key(KeyCode::Enter)),
            Some(Effect::Copy("body".into()))
        );
    }

    #[test]
    fn a_sealed_paste_asks_and_reveals_in_place() {
        let mut app = app();
        app.open_preview("r8ce".into(), Screen::Home);
        app.loaded(Paste {
            id: "r8ce".into(),
            body: "nibenc1:zzz".into(),
            enc: true,
            label: Some("deploy".into()),
        });
        assert!(app.preview.as_ref().unwrap().body.is_none());
        assert!(app.preview.as_ref().unwrap().typing);

        for c in "pw".chars() {
            app.handle(key(KeyCode::Char(c)));
        }
        assert_eq!(
            app.handle(key(KeyCode::Enter)),
            Some(Effect::Reveal("r8ce".into(), "pw".into()))
        );
        app.revealed("secret".into());
        assert_eq!(
            app.preview.as_ref().unwrap().body.as_deref(),
            Some("secret")
        );
        assert_eq!(
            app.handle(key(KeyCode::Enter)),
            Some(Effect::Copy("secret".into()))
        );
    }

    #[test]
    fn a_wrong_passphrase_keeps_the_field_and_shows_the_error() {
        let mut app = app();
        app.open_preview("r8ce".into(), Screen::Home);
        app.loaded(Paste {
            id: "r8ce".into(),
            body: "nibenc1:zzz".into(),
            enc: true,
            label: None,
        });
        app.handle(key(KeyCode::Char('x')));
        app.reveal_failed("cannot unlock r8ce".into());
        let preview = app.preview.as_ref().unwrap();
        assert!(preview.typing);
        assert!(preview.pass.is_empty());
        assert!(preview.error.is_some());
        assert_eq!(app.screen, Screen::Preview);
    }

    #[test]
    fn preview_scrolls_and_never_underflows() {
        let mut app = app();
        app.open_preview("k3x9".into(), Screen::Home);
        app.loaded(Paste {
            id: "k3x9".into(),
            body: "x".into(),
            enc: false,
            label: None,
        });
        app.handle(key(KeyCode::Up));
        assert_eq!(app.preview.as_ref().unwrap().scroll, 0);
        app.handle(key(KeyCode::PageDown));
        app.handle(key(KeyCode::PageDown));
        assert_eq!(app.preview.as_ref().unwrap().scroll, 20);
    }

    #[test]
    fn saved_returns_home_with_the_id_in_the_toast() {
        let mut app = app();
        app.handle(key(KeyCode::Enter));
        app.handle(key(KeyCode::Char('x')));
        app.saved("n1y".into());
        assert_eq!(app.screen, Screen::Home);
        assert_eq!(app.toast().map(|t| t.0), Some("saved n1y"));
    }
}
