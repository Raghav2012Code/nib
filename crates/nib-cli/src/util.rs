use std::io::{self, IsTerminal, Read, Write};
use std::process::Command;
use std::time::Duration;

/// Everything on stdin, as UTF-8.
pub fn read_stdin() -> Result<String, String> {
    let mut text = String::new();
    io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| format!("cannot read stdin: {e}"))?;
    Ok(text)
}

/// Shown on stderr when the paste is typed rather than piped.
pub const STDIN_HINT: &str = "paste or type, then press Ctrl+Z and Enter to finish\n";

pub fn is_interactive() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

pub fn ask(prompt: &str) -> Result<String, String> {
    eprint!("{prompt}");
    io::stderr().flush().ok();
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("cannot read input: {e}"))?;
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

/// A passphrase typed without echo, or `NIB_PASSPHRASE` when there is no terminal.
pub fn passphrase(prompt: &str) -> Result<String, String> {
    if let Ok(p) = std::env::var("NIB_PASSPHRASE")
        && !p.is_empty()
    {
        return Ok(p);
    }
    if !io::stdin().is_terminal() {
        return Err("passphrase required: set NIB_PASSPHRASE".into());
    }

    use crossterm::event::{Event, KeyCode, KeyModifiers, read};
    use crossterm::terminal;

    eprint!("{prompt}");
    io::stderr().flush().ok();
    terminal::enable_raw_mode().map_err(|e| e.to_string())?;

    let mut out = String::new();
    let result = loop {
        match read() {
            Ok(Event::Key(key)) => match key.code {
                KeyCode::Enter => break Ok(out),
                KeyCode::Esc => break Err("cancelled".to_string()),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    break Err("cancelled".to_string());
                }
                KeyCode::Backspace => {
                    if out.pop().is_some() {
                        eprint!("\u{8} \u{8}");
                    }
                }
                KeyCode::Char(c) => {
                    out.push(c);
                    eprint!("*");
                }
                _ => {}
            },
            Ok(_) => {}
            Err(e) => break Err(format!("cannot read passphrase: {e}")),
        }
    };

    terminal::disable_raw_mode().ok();
    eprintln!();
    result
}

/// `30s`, `10m`, `24h`, `7d`, or plain seconds.
pub fn parse_ttl(text: &str) -> Result<Duration, String> {
    let text = text.trim();
    let (digits, unit) = match text.chars().last() {
        Some(c) if c.is_ascii_alphabetic() => (&text[..text.len() - 1], c),
        _ => (text, 's'),
    };
    let n: u64 = digits
        .parse()
        .map_err(|_| format!("not a duration: {text}"))?;
    let secs = match unit.to_ascii_lowercase() {
        's' => n,
        'm' => n * 60,
        'h' => n * 3600,
        'd' => n * 86_400,
        'w' => n * 604_800,
        _ => return Err(format!("not a duration: {text}")),
    };
    if secs == 0 {
        return Err("ttl must be longer than zero".into());
    }
    Ok(Duration::from_secs(secs))
}

pub fn editor_command() -> String {
    std::env::var("NIB_EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        })
}

/// Open `text` in the editor and return what came back.
pub fn edit_text(text: &str) -> Result<String, String> {
    let path = std::env::temp_dir().join(format!("nib-edit-{}.txt", std::process::id()));
    std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;

    let editor = editor_command();
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("vi");
    let status = Command::new(program)
        .args(parts)
        .arg(&path)
        .status()
        .map_err(|e| format!("cannot start {program}: {e}"))?;

    let edited = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&path);
    if !status.success() {
        return Err(format!("{program} exited with {status}"));
    }
    Ok(edited)
}

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut board = arboard::Clipboard::new().map_err(|e| format!("no clipboard: {e}"))?;
    board
        .set_text(text.to_string())
        .map_err(|e| format!("no clipboard: {e}"))
}

pub fn ago(created_at_ms: i64, now_ms: i64) -> String {
    let secs = ((now_ms - created_at_ms) / 1000).max(0);
    match secs {
        0..=59 => "now".to_string(),
        60..=3599 => format!("{}m", secs / 60),
        3600..=86_399 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86_400),
    }
}

pub fn human_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
