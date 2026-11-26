use std::path::Path;

use nib_core::{AddOptions, Store};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SummaryDto {
    id: String,
    name: String,
    size: u64,
    created_at: i64,
    enc: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PasteDto {
    id: String,
    body: String,
    enc: bool,
    label: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HitDto {
    id: String,
    name: String,
    snippet: String,
}

fn open() -> Result<Store, String> {
    Store::open_default().map_err(|e| e.to_string())
}

fn plaintext(store: &Store, id: &str, passphrase: Option<String>) -> Result<String, String> {
    let paste = store.get(id).map_err(|e| e.to_string())?;
    if !paste.enc {
        return Ok(paste.body);
    }
    let pass = passphrase.ok_or_else(|| format!("{} is sealed, passphrase required", paste.id))?;
    store
        .reveal(&paste.id, &pass)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_pastes(limit: Option<usize>) -> Result<Vec<SummaryDto>, String> {
    let store = open()?;
    let rows = store.list(limit.unwrap_or(500)).map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|p| SummaryDto {
            id: p.id,
            name: p.name,
            size: p.size,
            created_at: p.created_at,
            enc: p.enc,
        })
        .collect())
}

#[tauri::command]
fn get_paste(id: String) -> Result<PasteDto, String> {
    let store = open()?;
    let paste = store.get(&id).map_err(|e| e.to_string())?;
    Ok(PasteDto {
        id: paste.id,
        body: paste.body,
        enc: paste.enc,
        label: paste.label,
    })
}

#[tauri::command]
fn reveal_paste(id: String, passphrase: String) -> Result<String, String> {
    let store = open()?;
    store
        .reveal(&id, &passphrase)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn search_pastes(query: String, limit: Option<usize>) -> Result<Vec<HitDto>, String> {
    let store = open()?;
    let hits = store
        .search(&query, limit.unwrap_or(20))
        .map_err(|e| e.to_string())?;
    Ok(hits
        .into_iter()
        .map(|h| HitDto {
            id: h.id,
            name: h.name,
            snippet: h.snippet,
        })
        .collect())
}

#[tauri::command]
fn add_paste(
    body: String,
    label: Option<String>,
    secret: Option<String>,
    ttl_seconds: Option<u64>,
) -> Result<String, String> {
    let store = open()?;
    let ttl = ttl_seconds.map(std::time::Duration::from_secs);
    let label = label.filter(|l| !l.trim().is_empty());
    store
        .add(&body, AddOptions { ttl, secret, label })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_paste(id: String, body: String) -> Result<(), String> {
    let store = open()?;
    store.update(&id, &body).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_paste(id: String) -> Result<(), String> {
    let store = open()?;
    store.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn copy_paste(id: String, passphrase: Option<String>) -> Result<(), String> {
    let store = open()?;
    let text = plaintext(&store, &id, passphrase)?;
    let mut board = arboard::Clipboard::new().map_err(|e| format!("no clipboard: {e}"))?;
    board
        .set_text(text)
        .map_err(|e| format!("no clipboard: {e}"))
}

#[tauri::command]
fn export_paste(id: String, path: String, passphrase: Option<String>) -> Result<(), String> {
    let store = open()?;
    let text = plaintext(&store, &id, passphrase)?;
    std::fs::write(Path::new(&path), text).map_err(|e| format!("cannot write {path}: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_pastes,
            get_paste,
            reveal_paste,
            search_pastes,
            add_paste,
            update_paste,
            delete_paste,
            copy_paste,
            export_paste
        ])
        .run(tauri::generate_context!())
        .expect("error while running nib");
}
