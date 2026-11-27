use std::io::{self, IsTerminal};

use nib_core::{Hit, Summary};

use crate::util;

/// ANSI only when stdout is a terminal and NO_COLOR is unset.
pub struct Paint {
    on: bool,
}

impl Paint {
    pub fn stdout() -> Paint {
        Paint {
            on: io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
        }
    }

    fn wrap(&self, code: &str, text: &str) -> String {
        if self.on {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    pub fn accent(&self, text: &str) -> String {
        self.wrap("38;2;224;122;69", text)
    }

    pub fn dim(&self, text: &str) -> String {
        self.wrap("38;5;244", text)
    }

    fn highlight(&self, text: &str) -> String {
        if self.on {
            text.replace('\u{1}', "\x1b[4m")
                .replace('\u{2}', "\x1b[24m")
        } else {
            text.replace(['\u{1}', '\u{2}'], "")
        }
    }
}

pub fn paste_row(paste: &Summary, now_ms: i64, paint: &Paint) -> String {
    format!(
        "{}  {:>4}  {:>7}  {}",
        paint.accent(&paste.id),
        util::ago(paste.created_at, now_ms),
        util::human_size(paste.size),
        paint.dim(&paste.name)
    )
}

pub fn hit_row(hit: &Hit, paint: &Paint) -> String {
    format!(
        "{}  {}",
        paint.accent(&hit.id),
        paint.highlight(&hit.snippet)
    )
}
