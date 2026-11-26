use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};

use crate::crypto;
use crate::error::{Error, Result};
use crate::ids;

pub const MAX_BODY: usize = 64 * 1024;

pub struct Store {
    conn: Connection,
}

#[derive(Debug, Default, Clone)]
pub struct AddOptions {
    pub ttl: Option<Duration>,
    pub secret: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Summary {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub created_at: i64,
    pub enc: bool,
}

#[derive(Debug, Clone)]
pub struct Hit {
    pub id: String,
    pub name: String,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct Paste {
    pub id: String,
    /// Plaintext for an ordinary paste, the sealed envelope for a secret one.
    pub body: String,
    pub enc: bool,
    pub label: Option<String>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Store> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)?;
        let store = Store { conn };
        store.migrate()?;
        store.purge()?;
        Ok(store)
    }

    pub fn open_default() -> Result<Store> {
        let path = match std::env::var_os("NIB_DB") {
            Some(p) => PathBuf::from(p),
            None => dirs::data_dir()
                .ok_or_else(|| Error::Message("no data directory".into()))?
                .join("nib")
                .join("nib.db"),
        };
        Store::open(&path)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;",
        )?;
        let version: i64 = self.conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version < 1 {
            self.conn.execute_batch(
                "BEGIN;
                 CREATE TABLE pastes (
                   id         TEXT PRIMARY KEY,
                   body       TEXT NOT NULL,
                   enc        INTEGER NOT NULL DEFAULT 0,
                   label      TEXT,
                   size       INTEGER NOT NULL,
                   created_at INTEGER NOT NULL,
                   updated_at INTEGER NOT NULL,
                   expires_at INTEGER
                 ) STRICT;
                 CREATE INDEX pastes_created ON pastes (created_at DESC);
                 CREATE INDEX pastes_expires ON pastes (expires_at);
                 CREATE VIRTUAL TABLE pastes_fts USING fts5 (
                   id UNINDEXED, body, label, tokenize = 'trigram'
                 );
                 PRAGMA user_version = 1;
                 COMMIT;",
            )?;
        }
        Ok(())
    }

    pub fn add(&self, body: &str, opts: AddOptions) -> Result<String> {
        check_size(body)?;
        let now = now_ms();
        let expires_at = opts.ttl.map(|d| now + d.as_millis() as i64);
        let enc = opts.secret.is_some();

        for _ in 0..16 {
            let id = ids::random_id();
            let stored = match &opts.secret {
                Some(pass) => crypto::seal(&id, pass, body.as_bytes())?,
                None => body.to_string(),
            };
            let inserted = self.conn.execute(
                "INSERT INTO pastes (id, body, enc, label, size, created_at, updated_at, expires_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7)
                 ON CONFLICT(id) DO NOTHING",
                params![id, stored, enc as i64, opts.label, body.len() as i64, now, expires_at],
            )?;
            if inserted == 1 {
                self.index(&id, if enc { "" } else { body }, opts.label.as_deref())?;
                return Ok(id);
            }
        }
        Err(Error::NoId)
    }

    pub fn get(&self, id_or_prefix: &str) -> Result<Paste> {
        let id = self.resolve(id_or_prefix)?;
        let (body, enc, label) = self
            .conn
            .query_row(
                "SELECT body, enc, label FROM pastes WHERE id = ?1",
                [&id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? != 0, r.get(2)?)),
            )
            .optional()?
            .ok_or_else(|| Error::NotFound(id_or_prefix.to_string()))?;
        Ok(Paste { id, body, enc, label })
    }

    pub fn reveal(&self, id_or_prefix: &str, passphrase: &str) -> Result<String> {
        let paste = self.get(id_or_prefix)?;
        if !paste.enc {
            return Ok(paste.body);
        }
        crypto::open(&paste.id, passphrase, &paste.body)
    }

    pub fn list(&self, limit: usize) -> Result<Vec<Summary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, body, enc, label, size, created_at
             FROM pastes ORDER BY created_at DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |r| {
            let id: String = r.get(0)?;
            let body: String = r.get(1)?;
            let enc: bool = r.get::<_, i64>(2)? != 0;
            let label: Option<String> = r.get(3)?;
            Ok(Summary {
                name: display_name(&id, &body, enc, label.as_deref()),
                id,
                size: r.get::<_, i64>(4)? as u64,
                created_at: r.get(5)?,
                enc,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Hit>> {
        let phrase = to_phrase(query);
        if phrase.is_empty() {
            return Ok(Vec::new());
        }
        let mut stmt = self.conn.prepare(
            "SELECT f.id, snippet(pastes_fts, -1, '', '', '…', 12), p.body, p.enc, p.label
             FROM pastes_fts f
             JOIN pastes p ON p.id = f.id
             WHERE pastes_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![phrase, limit as i64], |r| {
            let id: String = r.get(0)?;
            let body: String = r.get(2)?;
            let enc: bool = r.get::<_, i64>(3)? != 0;
            let label: Option<String> = r.get(4)?;
            Ok(Hit {
                snippet: r.get(1)?,
                name: display_name(&id, &body, enc, label.as_deref()),
                id,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn update(&self, id_or_prefix: &str, body: &str) -> Result<()> {
        check_size(body)?;
        let id = self.resolve(id_or_prefix)?;
        let enc: bool = self.conn.query_row(
            "SELECT enc FROM pastes WHERE id = ?1",
            [&id],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )?;
        if enc {
            return Err(Error::Message(format!("{id} is sealed, edit needs its passphrase")));
        }
        self.conn.execute(
            "UPDATE pastes SET body = ?2, size = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, body, body.len() as i64, now_ms()],
        )?;
        self.unindex(&id)?;
        self.index(&id, body, None)?;
        Ok(())
    }

    pub fn delete(&self, id_or_prefix: &str) -> Result<()> {
        let id = self.resolve(id_or_prefix)?;
        self.conn.execute("DELETE FROM pastes WHERE id = ?1", [&id])?;
        self.unindex(&id)?;
        Ok(())
    }

    pub fn purge(&self) -> Result<()> {
        let now = now_ms();
        self.conn.execute(
            "DELETE FROM pastes_fts WHERE id IN (SELECT id FROM pastes WHERE expires_at IS NOT NULL AND expires_at <= ?1)",
            [now],
        )?;
        self.conn.execute(
            "DELETE FROM pastes WHERE expires_at IS NOT NULL AND expires_at <= ?1",
            [now],
        )?;
        Ok(())
    }

    fn index(&self, id: &str, body: &str, label: Option<&str>) -> Result<()> {
        let label = label.unwrap_or("");
        if body.is_empty() && label.is_empty() {
            return Ok(());
        }
        self.conn.execute(
            "INSERT INTO pastes_fts (id, body, label) VALUES (?1, ?2, ?3)",
            params![id, body, label],
        )?;
        Ok(())
    }

    fn unindex(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM pastes_fts WHERE id = ?1", [id])?;
        Ok(())
    }

    fn resolve(&self, id_or_prefix: &str) -> Result<String> {
        let prefix = id_or_prefix.trim().to_ascii_lowercase();
        if !ids::is_valid_prefix(&prefix) {
            return Err(Error::NotFound(id_or_prefix.to_string()));
        }
        let exact: Option<String> = self
            .conn
            .query_row("SELECT id FROM pastes WHERE id = ?1", [&prefix], |r| r.get(0))
            .optional()?;
        if let Some(id) = exact {
            return Ok(id);
        }
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM pastes WHERE id LIKE ?1 || '%' ORDER BY id LIMIT 8")?;
        let matches: Vec<String> = stmt
            .query_map([&prefix], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        match matches.len() {
            0 => Err(Error::NotFound(id_or_prefix.to_string())),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => Err(Error::Ambiguous(id_or_prefix.to_string(), matches.join(", "))),
        }
    }
}

fn check_size(body: &str) -> Result<()> {
    if body.len() > MAX_BODY {
        return Err(Error::TooLarge(body.len()));
    }
    Ok(())
}

fn display_name(id: &str, body: &str, enc: bool, label: Option<&str>) -> String {
    if enc {
        return match label.map(str::trim) {
            Some(l) if !l.is_empty() => cut(l, 80),
            _ => format!("secret {id}"),
        };
    }
    let line = body.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let name = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        "untitled".to_string()
    } else {
        cut(&name, 80)
    }
}

fn cut(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

fn to_phrase(query: &str) -> String {
    let q = query.trim();
    if q.is_empty() {
        return String::new();
    }
    format!("\"{}\"", q.replace('"', "\"\""))
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("nib-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{name}.db"));
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
        path
    }

    fn opts() -> AddOptions {
        AddOptions::default()
    }

    #[test]
    fn round_trip_is_byte_exact() {
        let store = Store::open(&db("roundtrip")).unwrap();
        let body = "fn main() {\n\tprintln!(\"hi\");\n}\n";
        let id = store.add(body, opts()).unwrap();
        assert_eq!(id.len(), 4);
        assert_eq!(store.get(&id).unwrap().body, body);
    }

    #[test]
    fn size_limit_holds() {
        let store = Store::open(&db("size")).unwrap();
        let at_limit = "x".repeat(MAX_BODY);
        assert!(store.add(&at_limit, opts()).is_ok());
        let over = "x".repeat(MAX_BODY + 1);
        assert!(matches!(store.add(&over, opts()), Err(Error::TooLarge(_))));
    }

    #[test]
    fn name_comes_from_the_first_line() {
        let store = Store::open(&db("name")).unwrap();
        let id = store.add("\n\n   let  x = 1;   \nmore", opts()).unwrap();
        let listed = store.list(5).unwrap();
        assert_eq!(listed[0].id, id);
        assert_eq!(listed[0].name, "let x = 1;");
    }

    #[test]
    fn prefix_lookup_and_missing() {
        let store = Store::open(&db("prefix")).unwrap();
        let id = store.add("hello", opts()).unwrap();
        let prefix = &id[..2];
        assert_eq!(store.get(prefix).unwrap().id, id);
        assert!(matches!(store.get("zzzz"), Err(Error::NotFound(_))));
        assert!(matches!(store.get("nope!"), Err(Error::NotFound(_))));
    }

    #[test]
    fn ambiguous_prefix_lists_candidates() {
        let store = Store::open(&db("ambiguous")).unwrap();
        for id in ["aaaa", "aaab"] {
            store
                .conn
                .execute(
                    "INSERT INTO pastes (id, body, enc, size, created_at, updated_at)
                     VALUES (?1, 'x', 0, 1, 0, 0)",
                    [id],
                )
                .unwrap();
        }
        match store.get("aa") {
            Err(Error::Ambiguous(_, ids)) => assert!(ids.contains("aaaa") && ids.contains("aaab")),
            other => panic!("expected ambiguous, got {other:?}"),
        }
        assert_eq!(store.get("aaaa").unwrap().id, "aaaa");
    }

    #[test]
    fn search_finds_substrings() {
        let store = Store::open(&db("search")).unwrap();
        let id = store.add("fn parse_args() -> Args {", opts()).unwrap();
        for query in ["parse_args", "arse_arg", "Args"] {
            let hits = store.search(query, 10).unwrap();
            assert_eq!(hits.len(), 1, "query {query}");
            assert_eq!(hits[0].id, id);
            assert!(!hits[0].snippet.is_empty());
        }
        assert!(store.search("nothing-like-this", 10).unwrap().is_empty());
        assert!(store.search("ab", 10).unwrap().is_empty(), "trigram needs three characters");
    }

    #[test]
    fn sealed_pastes_hide_the_body_but_show_the_label() {
        let store = Store::open(&db("sealed")).unwrap();
        let id = store
            .add(
                "deploy-key-xyz",
                AddOptions { secret: Some("pw".into()), label: Some("deploy key".into()), ..Default::default() },
            )
            .unwrap();

        let paste = store.get(&id).unwrap();
        assert!(paste.enc);
        assert!(!paste.body.contains("deploy-key-xyz"));

        assert!(store.search("deploy-key-xyz", 10).unwrap().is_empty());
        let by_label = store.search("deploy key", 10).unwrap();
        assert_eq!(by_label.len(), 1);

        assert_eq!(store.reveal(&id, "pw").unwrap(), "deploy-key-xyz");
        assert!(matches!(store.reveal(&id, "wrong"), Err(Error::Unlock(_))));
        assert_eq!(store.list(5).unwrap()[0].name, "deploy key");
    }

    #[test]
    fn expiry_purges_on_next_open() {
        let path = db("expiry");
        {
            let store = Store::open(&path).unwrap();
            store
                .add("short lived", AddOptions { ttl: Some(Duration::from_millis(1)), ..Default::default() })
                .unwrap();
        }
        std::thread::sleep(Duration::from_millis(10));
        let store = Store::open(&path).unwrap();
        assert!(store.list(10).unwrap().is_empty());
        assert!(store.search("short lived", 10).unwrap().is_empty());
    }

    #[test]
    fn delete_removes_row_and_index() {
        let store = Store::open(&db("delete")).unwrap();
        let id = store.add("temporary words", opts()).unwrap();
        store.delete(&id).unwrap();
        assert!(store.get(&id).is_err());
        assert!(store.search("temporary", 10).unwrap().is_empty());
    }

    #[test]
    fn update_replaces_text_and_index() {
        let store = Store::open(&db("update")).unwrap();
        let id = store.add("before", opts()).unwrap();
        store.update(&id, "after").unwrap();
        assert_eq!(store.get(&id).unwrap().body, "after");
        assert!(store.search("before", 10).unwrap().is_empty());
        assert_eq!(store.search("after", 10).unwrap().len(), 1);
    }
}
