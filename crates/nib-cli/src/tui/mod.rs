//! The full-screen interface: a pure state machine, a renderer, and the loop
//! that joins them to the store.

mod app;
mod ui;

use std::time::Duration;

use nib_core::{AddOptions, Store};
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use app::{App, Effect};

pub fn run(store: &Store) -> Result<(), String> {
    let mut app = App::new(store.path().display().to_string());
    refresh(&mut app, store);

    let mut terminal = ratatui::init();
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        previous(info);
    }));

    let result = event_loop(&mut terminal, &mut app, store);
    ratatui::restore();
    result
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    store: &Store,
) -> Result<(), String> {
    loop {
        terminal
            .draw(|frame| ui::draw(frame, app))
            .map_err(|e| e.to_string())?;

        if !event::poll(Duration::from_millis(200)).map_err(|e| e.to_string())? {
            continue;
        }
        let Ok(event) = event::read() else { continue };
        let Event::Key(key) = event else { continue };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        if let Some(effect) = app.handle(key)
            && apply(effect, app, store)?
        {
            return Ok(());
        }
    }
}

fn apply(effect: Effect, app: &mut App, store: &Store) -> Result<bool, String> {
    match effect {
        Effect::Quit => return Ok(true),
        Effect::Save(body) => match store.add(&body, AddOptions::default()) {
            Ok(id) => {
                app.saved(id);
                refresh(app, store);
            }
            Err(error) => app.save_failed(error.to_string()),
        },
        Effect::Search(query, seq) => match store.search(&query, 50) {
            Ok(hits) => app.search_done(seq, hits),
            Err(error) => app.save_failed(error.to_string()),
        },
        Effect::Open(id) => match store.get(&id) {
            Ok(paste) => app.loaded(paste),
            Err(error) => app.note(error.to_string(), true),
        },
        Effect::Copy(text) => {
            let id = app
                .preview
                .as_ref()
                .map(|p| p.id.clone())
                .unwrap_or_default();
            match arboard::Clipboard::new().and_then(|mut board| board.set_text(text)) {
                Ok(()) => app.note(format!("copied {id}"), false),
                Err(error) => app.note(format!("no clipboard: {error}"), true),
            }
        }
        Effect::Reveal(id, passphrase) => match store.reveal(&id, &passphrase) {
            Ok(text) => app.revealed(text),
            Err(error) => app.reveal_failed(error.to_string()),
        },
    }
    Ok(false)
}

fn refresh(app: &mut App, store: &Store) {
    match store.list(50) {
        Ok(recent) => app.refresh(recent),
        Err(error) => app.note(error.to_string(), true),
    }
}
