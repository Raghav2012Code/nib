use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use nib_core::{AddOptions, Store};

use crate::render::{Paint, hit_row, paste_row};
use crate::util;

const MARK: [&str; 9] = [
    "  ▄███▄  ",
    "▄███████▄",
    "█████████",
    "█████████",
    "█████████",
    "████▀████",
    "████ ████",
    " ███ ███ ",
    "  ██ ██  ",
];
const ROWS: [&str; 9] = [
    "215;236;254",
    "188;220;247",
    "166;201;236",
    "147;183;221",
    "130;168;206",
    "113;153;191",
    "98;139;176",
    "84;125;162",
    "72;112;148",
];
const EMBER: &str = "224;122;69";
const INK: &str = "232;230;225";

pub fn run(store: &Store) -> Result<(), String> {
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    if width < 60 || height < 24 {
        println!("nib — offline pastebin");
        println!("nib add · nib ls · nib search <query>");
        return Ok(());
    }

    draw(store, width as usize)?;

    match read_key()? {
        '1' => add(store),
        '2' => search(store),
        '3' => recent(store),
        _ => Ok(()),
    }
}

fn draw(store: &Store, width: usize) -> Result<(), String> {
    let mut out = io::stdout().lock();
    let paint = Paint::stdout();
    let on = paint_on();

    for (row, colour) in MARK.iter().zip(ROWS) {
        let _ = writeln!(out, "{}{}", pad(9, width), rgb(colour, row, on));
    }
    let tip = format!(
        "{}{}{}",
        rgb(ROWS[8], "  ▀▀", on),
        rgb(EMBER, "█", on),
        rgb(ROWS[8], "▀▀  ", on)
    );
    let _ = writeln!(out, "{}{}", pad(9, width), tip);

    let _ = writeln!(out);
    let _ = writeln!(out, "{}{}", pad(5, width), rgb(INK, "n i b", on));
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{}\x1b[1mOffline pastebin for your terminal.\x1b[0m",
        pad(34, width)
    );
    let _ = writeln!(
        out,
        "{}{}",
        pad(41, width),
        paint.dim("text in · searchable · stays on this disk")
    );
    let _ = writeln!(out);
    let store_line = format!("{}   {}", paint.dim("store"), store.path().display());
    let _ = writeln!(out, "{}{}", pad(visible(&store_line), width), store_line);
    let _ = writeln!(out);

    let inner = 54;
    let margin = pad(inner + 2, width);
    let _ = writeln!(out, "{margin}┌{}┐", "─".repeat(inner));
    for (marker, label, desc) in [
        ("› 1", "Add a paste", "type in your editor, then save"),
        ("  2", "Find a paste", "search the text, pick a hit"),
        ("  3", "Recent pastes", "the last ten, copy one"),
    ] {
        let plain = format!("{marker} {label:<12}{desc}");
        let line = format!(
            "{} {}{}",
            paint.accent(marker),
            paint.bold(&format!("{label:<12}")),
            paint.dim(desc)
        );
        let _ = writeln!(out, "{margin}│{line}{}│", " ".repeat(inner - plain.len()));
    }
    let _ = writeln!(out, "{margin}└{}┘", "─".repeat(inner));

    let _ = writeln!(out);
    let footer = paint.dim("1-3 choose  ·  q quit");
    let _ = writeln!(out, "{}{}", pad(visible(&footer), width), footer);
    out.flush().ok();
    Ok(())
}

fn add(store: &Store) -> Result<(), String> {
    let text = util::edit_text("")?;
    if text.trim().is_empty() {
        return Ok(());
    }
    let id = store
        .add(&text, AddOptions::default())
        .map_err(|e| e.to_string())?;
    println!("{id}");
    Ok(())
}

fn search(store: &Store) -> Result<(), String> {
    let query = util::ask("search › ")?;
    if query.trim().is_empty() {
        return Ok(());
    }
    let hits = store.search(&query, 5).map_err(|e| e.to_string())?;
    if hits.is_empty() {
        println!("no matches");
        return Ok(());
    }
    let paint = Paint::stdout();
    for hit in &hits {
        println!("{}", hit_row(hit, &paint));
    }
    println!();
    let id = util::ask("copy which? ")?;
    if id.trim().is_empty() {
        return Ok(());
    }
    copy(store, id.trim())
}

fn recent(store: &Store) -> Result<(), String> {
    let paint = Paint::stdout();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let pastes = store.list(10).map_err(|e| e.to_string())?;
    if pastes.is_empty() {
        println!("nothing stored yet");
        return Ok(());
    }
    for paste in &pastes {
        println!("{}", paste_row(paste, now, &paint));
    }
    println!();
    let id = util::ask("copy which? ")?;
    if id.trim().is_empty() {
        return Ok(());
    }
    copy(store, id.trim())
}

fn copy(store: &Store, id: &str) -> Result<(), String> {
    let paste = store.get(id).map_err(|e| e.to_string())?;
    let text = if paste.enc {
        let pass = util::passphrase("passphrase: ")?;
        store.reveal(&paste.id, &pass).map_err(|e| e.to_string())?
    } else {
        paste.body
    };
    util::copy_to_clipboard(&text)?;
    println!("copied {}", paste.id);
    Ok(())
}

fn read_key() -> Result<char, String> {
    crossterm::terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    let key = loop {
        match event::read() {
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release => match key.code {
                KeyCode::Char(c) => break c.to_ascii_lowercase(),
                KeyCode::Esc => break 'q',
                KeyCode::Enter => break '\n',
                _ => continue,
            },
            Ok(_) => continue,
            Err(e) => {
                crossterm::terminal::disable_raw_mode().ok();
                return Err(format!("cannot read a key: {e}"));
            }
        }
    };
    crossterm::terminal::disable_raw_mode().ok();
    Ok(key)
}

fn paint_on() -> bool {
    std::env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal()
}

use std::io::IsTerminal;

fn rgb(rgb: &str, text: &str, on: bool) -> String {
    if on {
        let mut parts = rgb.split(';').map(|n| n.parse::<u8>().unwrap_or(0));
        let (r, g, b) = (
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
        );
        format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn pad(visible: usize, width: usize) -> String {
    " ".repeat(width.saturating_sub(visible) / 2)
}

fn visible(text: &str) -> usize {
    let mut count = 0;
    let mut in_escape = false;
    for c in text.chars() {
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\u{1b}' {
            in_escape = true;
        } else {
            count += 1;
        }
    }
    count
}
