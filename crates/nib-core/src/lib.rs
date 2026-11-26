//! Nib's store: one SQLite file, full-text search, optional per-paste sealing.
//!
//! Only this crate opens the database. The CLI and the desktop app call it.

mod crypto;
mod error;
mod ids;
mod store;

pub use error::{Error, Result};
pub use store::{AddOptions, Hit, MAX_BODY, Paste, Store, Summary};
