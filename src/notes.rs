use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Note {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
}

pub fn notes_dir() -> PathBuf {
    std::env::var("SN_NOTES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("notes")
        })
}

pub fn left_panel_width() -> u16 {
    std::env::var("SN_LEFT_WIDTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(32)
}

pub fn load_notes(dir: &Path) -> io::Result<Vec<Note>> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
        return Ok(Vec::new());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext == "md" || ext == "txt")
        })
        .collect();

    entries.sort();

    entries
        .into_iter()
        .map(|path| {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string();
            let content = fs::read_to_string(&path).unwrap_or_default();
            Ok(Note { name, path, content })
        })
        .collect()
}
