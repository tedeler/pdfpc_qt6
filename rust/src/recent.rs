//! Persist recently opened PDFs and the last viewed page per file.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_RECENT: usize = 20;

#[derive(Clone, Debug)]
pub struct RecentEntry {
    pub path: String,
    pub page: i32,
}

pub fn config_dir() -> PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("pdfpc-qt6");
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("pdfpc-qt6")
}

fn recent_file_path() -> PathBuf {
    config_dir().join("recent.json")
}

pub fn load() -> Vec<RecentEntry> {
    let path = recent_file_path();
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    let Some(files) = value.get("files").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in files {
        let Some(path) = item.get("path").and_then(|v| v.as_str()) else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        let page = item
            .get("page")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .max(0) as i32;
        out.push(RecentEntry {
            path: path.to_string(),
            page,
        });
    }
    out
}

pub fn save(entries: &[RecentEntry]) {
    let dir = config_dir();
    if let Err(err) = fs::create_dir_all(&dir) {
        eprintln!("pdfpc: cannot create config dir {}: {err}", dir.display());
        return;
    }
    let files: Vec<Value> = entries
        .iter()
        .map(|e| json!({ "path": e.path, "page": e.page.max(0) }))
        .collect();
    let text = Value::Object(
        [("files".to_string(), Value::Array(files))]
            .into_iter()
            .collect(),
    )
    .to_string();
    let path = recent_file_path();
    if let Err(err) = fs::write(&path, text) {
        eprintln!("pdfpc: cannot write {}: {err}", path.display());
    }
}

pub fn to_json(entries: &[RecentEntry]) -> String {
    let files: Vec<Value> = entries
        .iter()
        .filter(|e| Path::new(&e.path).is_file())
        .map(|e| {
            let name = Path::new(&e.path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| e.path.clone());
            json!({
                "path": e.path,
                "page": e.page.max(0),
                "name": name,
            })
        })
        .collect();
    Value::Array(files).to_string()
}

/// Move `path` to the front (or insert it) and set its stored page.
pub fn touch(entries: &mut Vec<RecentEntry>, path: &str, page: i32) {
    entries.retain(|e| e.path != path);
    entries.insert(
        0,
        RecentEntry {
            path: path.to_string(),
            page: page.max(0),
        },
    );
    if entries.len() > MAX_RECENT {
        entries.truncate(MAX_RECENT);
    }
}

pub fn set_page(entries: &mut Vec<RecentEntry>, path: &str, page: i32) {
    if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        entry.page = page.max(0);
    } else {
        touch(entries, path, page);
        return;
    }
}

pub fn page_for(entries: &[RecentEntry], path: &str) -> i32 {
    entries
        .iter()
        .find(|e| e.path == path)
        .map(|e| e.page.max(0))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn touch_moves_to_front_and_caps() {
        let _g = LOCK.lock().unwrap();
        let mut entries = Vec::new();
        for i in 0..25 {
            touch(&mut entries, &format!("/tmp/f{i}.pdf"), i);
        }
        assert_eq!(entries.len(), MAX_RECENT);
        assert_eq!(entries[0].path, "/tmp/f24.pdf");
        assert_eq!(entries[0].page, 24);
        set_page(&mut entries, "/tmp/f24.pdf", 3);
        assert_eq!(page_for(&entries, "/tmp/f24.pdf"), 3);
        assert_eq!(entries[0].path, "/tmp/f24.pdf");
    }

    #[test]
    fn roundtrip_save_load() {
        let _g = LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("pdfpc-recent-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_var("XDG_CONFIG_HOME", &dir);

        let entries = vec![RecentEntry {
            path: "/tmp/demo.pdf".into(),
            page: 7,
        }];
        save(&entries);
        let loaded = load();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].path, "/tmp/demo.pdf");
        assert_eq!(loaded[0].page, 7);

        std::env::remove_var("XDG_CONFIG_HOME");
        let _ = fs::remove_dir_all(&dir);
    }
}
