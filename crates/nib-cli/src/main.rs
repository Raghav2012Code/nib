mod args;
mod render;
mod splash;
mod util;

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use clap::Parser;

use args::{Cli, Command};
use nib_core::{AddOptions, Store};
use render::Paint;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn open() -> Result<Store, String> {
    Store::open_default().map_err(|e| e.to_string())
}

fn run(cli: Cli) -> Result<(), String> {
    let Some(command) = cli.command else {
        if let Some(id) = cli.id {
            let store = open()?;
            let text = read_paste(&store, &id)?;
            return write_out(&text);
        }
        return bare();
    };
    let store = open()?;

    match command {
        Command::Add {
            file,
            ttl,
            secret,
            label,
        } => {
            let body = match file {
                Some(path) => std::fs::read_to_string(&path)
                    .map_err(|e| format!("cannot read {}: {e}", path.display()))?,
                None => util::read_stdin()?,
            };
            if body.trim().is_empty() {
                return Err("nothing to store".into());
            }
            let ttl = match ttl {
                Some(text) => Some(util::parse_ttl(&text)?),
                None => None,
            };
            let secret = if secret {
                let first = util::passphrase("passphrase: ")?;
                if io::stdin().is_terminal() {
                    let again = util::passphrase("again: ")?;
                    if first != again {
                        return Err("passphrases differ".into());
                    }
                }
                Some(first)
            } else {
                None
            };
            let id = store
                .add(&body, AddOptions { ttl, secret, label })
                .map_err(|e| e.to_string())?;
            println!("{id}");
            Ok(())
        }

        Command::Ls { limit, all } => {
            let limit = if all { usize::MAX } else { limit };
            let paint = Paint::stdout();
            let now = now_ms();
            let pastes = store.list(limit).map_err(|e| e.to_string())?;
            if pastes.is_empty() {
                println!("{}", paint.dim("nothing stored yet, try: echo hi | nib"));
                return Ok(());
            }
            for paste in &pastes {
                println!("{}", render::paste_row(paste, now, &paint));
            }
            Ok(())
        }

        Command::Get { id, .. } => {
            let text = read_paste(&store, &id)?;
            write_out(&text)
        }

        Command::Copy { id } => {
            let text = read_paste(&store, &id)?;
            util::copy_to_clipboard(&text)?;
            eprintln!("copied {}", id.trim());
            Ok(())
        }

        Command::Edit { id } => {
            let paste = store.get(&id).map_err(|e| e.to_string())?;
            if paste.enc {
                return Err(format!("{} is sealed, edit needs its passphrase", paste.id));
            }
            let edited = util::edit_text(&paste.body)?;
            if edited.trim().is_empty() {
                return Err("empty paste, nothing changed".into());
            }
            store
                .update(&paste.id, &edited)
                .map_err(|e| e.to_string())?;
            eprintln!("updated {}", paste.id);
            Ok(())
        }

        Command::Search { query, limit } => {
            let paint = Paint::stdout();
            let hits = store
                .search(&query.join(" "), limit)
                .map_err(|e| e.to_string())?;
            if hits.is_empty() {
                println!("{}", paint.dim("no matches"));
                return Ok(());
            }
            for hit in &hits {
                println!("{}", render::hit_row(hit, &paint));
            }
            Ok(())
        }

        Command::Rm { id, force } => {
            let paste = store.get(&id).map_err(|e| e.to_string())?;
            if !force {
                if !util::is_interactive() {
                    return Err("use --force to delete without a terminal".into());
                }
                let answer = util::ask(&format!("delete {}? [y/N] ", paste.id))?;
                if !matches!(answer.as_str(), "y" | "Y" | "yes") {
                    return Err("cancelled".into());
                }
            }
            store.delete(&paste.id).map_err(|e| e.to_string())?;
            eprintln!("deleted {}", paste.id);
            Ok(())
        }
    }
}

fn bare() -> Result<(), String> {
    let store = open()?;
    if !io::stdin().is_terminal() {
        let body = util::read_stdin()?;
        if body.trim().is_empty() {
            return Err("nothing to store".into());
        }
        let id = store
            .add(&body, AddOptions::default())
            .map_err(|e| e.to_string())?;
        println!("{id}");
        return Ok(());
    }
    splash::run(&store)
}

fn read_paste(store: &Store, id: &str) -> Result<String, String> {
    let paste = store.get(id).map_err(|e| e.to_string())?;
    if !paste.enc {
        return Ok(paste.body);
    }
    let pass = util::passphrase("passphrase: ")?;
    store.reveal(&paste.id, &pass).map_err(|e| e.to_string())
}

/// Stored bytes go to stdout. A newline is added only for a human looking at a terminal.
fn write_out(text: &str) -> Result<(), String> {
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    if io::stdout().is_terminal() && !text.ends_with('\n') {
        out.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
