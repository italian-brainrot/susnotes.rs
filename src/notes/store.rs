use crate::config::Settings;
use crate::crypto;
use super::model::Note;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct NoteStore {
    pub notes: HashMap<String, Note>,
    settings: Settings,
    password: Option<String>,
}

impl NoteStore {
    pub fn new(settings: Settings) -> Self {
        Self {
            notes: HashMap::new(),
            settings,
            password: None,
        }
    }

    pub fn set_password(&mut self, password: &str) {
        self.password = Some(password.to_string());
    }

    #[allow(dead_code)]
    pub fn has_password(&self) -> bool {
        self.password.as_ref().map(|p| !p.is_empty()).unwrap_or(false)
    }

    pub fn check_password(&self, password: &str) -> bool {
        match (&self.settings.password_hash, &self.settings.password_salt) {
            (Some(hash), Some(salt)) => {
                crypto::verify_password(password, hash, salt)
            }
            _ => false,
        }
    }

    pub fn set_initial_password(&mut self, password: &str) {
        let (hash, salt) = crypto::hash_password(password);
        self.settings.password_hash = Some(hash);
        self.settings.password_salt = Some(salt);
        self.password = Some(password.to_string());
        self.settings.save().unwrap_or_else(|e| log::warn!("Failed to save settings: {}", e));
    }

    pub fn change_password(&mut self, old_password: &str, new_password: &str) -> Result<(), String> {
        if !self.check_password(old_password) {
            return Err("Current password is incorrect".to_string());
        }
        self.update_hash(new_password);
        Ok(())
    }

    /// Update the password hash directly without re-verifying (for internal use after check_password).
    fn update_hash(&mut self, password: &str) {
        let (hash, salt) = crypto::hash_password(password);
        self.settings.password_hash = Some(hash);
        self.settings.password_salt = Some(salt);
        self.password = Some(password.to_string());
        self.settings.save().unwrap_or_else(|e| log::warn!("Failed to save settings: {}", e));
    }

    pub fn ensure_folders(&self) -> Result<PathBuf, String> {
        let folder = self.settings.ensure_notes_folder()?;
        let pages = self.settings.pages_folder();
        let assets = self.settings.assets_folder();

        fs::create_dir_all(&pages).map_err(|e| format!("Failed to create pages folder: {}", e))?;
        fs::create_dir_all(&assets).map_err(|e| format!("Failed to create assets folder: {}", e))?;

        Ok(folder)
    }

    pub fn load_all(&mut self) -> Result<(), String> {
        let pages_folder = self.settings.pages_folder();
        if !pages_folder.exists() {
            return Ok(());
        }

        self.notes.clear();

        let entries = match fs::read_dir(&pages_folder) {
            Ok(e) => e,
            Err(e) => return Err(format!("Failed to read pages: {}", e)),
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    log::warn!("Failed to read entry: {}", e);
                    continue;
                }
            };
            let path = entry.path();
            if path.extension().map(|e| e == "sn").unwrap_or(false) {
                if let Some(note) = self.load_note(&path) {
                    self.notes.insert(note.id.clone(), note);
                }
            }
        }

        log::info!("Loaded {} notes from {}", self.notes.len(), pages_folder.display());
        Ok(())
    }

    fn note_path(&self, id: &str) -> PathBuf {
        self.settings.pages_folder().join(format!("{}.sn", id))
    }

    fn load_note(&self, path: &Path) -> Option<Note> {
        let data = match fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("Failed to read note {}: {}", path.display(), e);
                return None;
            }
        };

        let content = match &self.password {
            Some(pwd) => match crypto::decrypt(&data, pwd) {
                Ok(d) => String::from_utf8(d).ok()?,
                Err(e) => {
                    log::warn!("Failed to decrypt note {}: {}", path.display(), e);
                    return None;
                }
            },
            None => String::from_utf8(data).ok()?,
        };

        let note: Note = match serde_json::from_str(&content) {
            Ok(n) => n,
            Err(e) => {
                log::warn!("Failed to parse note {}: {}", path.display(), e);
                return None;
            }
        };

        Some(note)
    }

    pub fn save_note(&mut self, note: Note) -> Result<(), String> {
        let path = self.note_path(&note.id);
        let content = serde_json::to_string_pretty(&note).map_err(|e| format!("Failed to serialize note: {}", e))?;

        let data = match &self.password {
            Some(pwd) => crypto::encrypt(content.as_bytes(), pwd)
                .map_err(|e| format!("Failed to encrypt note: {}", e))?,
            None => content.into_bytes(),
        };

        fs::write(&path, data).map_err(|e| format!("Failed to save note: {}", e))?;
        log::info!("Saved note: {}", note.name);
        Ok(())
    }

    pub fn update_and_save_note(&mut self, id: &str, content: &str) -> Result<(), String> {
        let note = self.notes.get(id).cloned().ok_or_else(|| format!("Note not found: {}", id))?;
        let mut note = note.clone();
        note.update_content(content);
        self.save_note(note)
    }

    pub fn delete_note(&mut self, id: &str) -> Result<(), String> {
        let path = self.note_path(id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to delete note file: {}", e))?;
        }
        self.notes.remove(id);
        Ok(())
    }

    pub fn get_note(&self, id: &str) -> Option<&Note> {
        self.notes.get(id)
    }

    pub fn get_note_mut(&mut self, id: &str) -> Option<&mut Note> {
        self.notes.get_mut(id)
    }

    pub fn get_daily_notes(&self) -> Vec<&Note> {
        let mut notes: Vec<&Note> = self.notes.values().filter(|n| n.is_daily()).collect();
        notes.sort_by(|a, b| b.name.cmp(&a.name));
        notes
    }

    pub fn get_non_daily_notes(&self) -> Vec<&Note> {
        let mut notes: Vec<&Note> = self.notes.values().filter(|n| !n.is_daily()).collect();
        notes.sort_by(|a, b| a.name.cmp(&b.name));
        notes
    }

    pub fn get_all_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.notes.values().map(|n| n.name.clone()).collect();
        names.sort();
        names
    }

    /// Get or create a daily note, returns the note id
    pub fn get_or_create_daily_note(&mut self, date: &str) -> String {
        // Check if daily note exists
        for (id, note) in self.notes.iter() {
            if note.name == date {
                return id.clone();
            }
        }
        // Create new daily note
        let note = Note::new(date, "");
        let id = note.id.clone();
        self.notes.insert(id.clone(), note);
        id
    }

    pub fn find_note_by_name(&mut self, name: &str) -> String {
        // Try exact match first
        for (id, note) in self.notes.iter() {
            if note.name == name {
                return id.clone();
            }
        }
        // Create new note on demand (e.g. when following a #link or [[link]])
        log::debug!("Creating new note: {}", name);
        let note = Note::new(name, "");
        let id = note.id.clone();
        self.notes.insert(id.clone(), note);
        id
    }

    #[allow(dead_code)]
    pub fn save_assets_folder(&self) -> PathBuf {
        self.settings.assets_folder()
    }

    #[allow(dead_code)]
    pub fn save_image(&self, note_id: &str, image_data: &[u8], filename: &str) -> Result<String, String> {
        let assets_folder = self.settings.assets_folder();
        fs::create_dir_all(&assets_folder).map_err(|e| format!("Failed to create assets folder: {}", e))?;

        let safe_name = format!("{}_{}", note_id, filename);
        let path = assets_folder.join(&safe_name);

        fs::write(&path, image_data).map_err(|e| format!("Failed to save image: {}", e))?;
        Ok(format!("/assets/{}", safe_name))
    }

    #[allow(dead_code)]
    pub fn get_image_path(&self, asset_path: &str) -> Option<PathBuf> {
        if asset_path.starts_with("/assets/") {
            Some(self.settings.assets_folder().join(&asset_path[8..]))
        } else {
            None
        }
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    #[allow(dead_code)]
    pub fn update_settings(&mut self, settings: Settings) {
        self.settings = settings;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_settings() -> Settings {
        let temp_dir = env::temp_dir().join("susnotes_test");
        let _ = fs::create_dir_all(&temp_dir);
        let _ = fs::create_dir_all(temp_dir.join("pages"));
        let _ = fs::create_dir_all(temp_dir.join("assets"));
        Settings {
            notes_folder: Some(temp_dir),
            ..Settings::default()
        }
    }

    #[test]
    fn test_save_load_roundtrip() {
        let settings = temp_settings();
        let mut store = NoteStore::new(settings);

        let note = Note::new("Test Note", "Hello, world!");
        let id = note.id.clone();
        store.notes.insert(id.clone(), note);
        store.save_note(store.notes.get(&id).unwrap().clone()).unwrap();

        // Reload from disk
        store.load_all().unwrap();
        assert_eq!(store.get_note(&id).unwrap().name, "Test Note");
        assert_eq!(store.get_note(&id).unwrap().content, "Hello, world!");
    }

    #[test]
    fn test_password_change_flow() {
        let settings = temp_settings();
        let mut store = NoteStore::new(settings);

        // Set initial password
        store.set_initial_password("old-password");
        assert!(store.check_password("old-password"));
        assert!(!store.check_password("wrong-password"));

        // Change password
        store.change_password("old-password", "new-password").unwrap();
        assert!(!store.check_password("old-password"));
        assert!(store.check_password("new-password"));

        // Wrong old password should fail
        let result = store.change_password("wrong-password", "another-password");
        assert!(result.is_err());
    }

    #[test]
    fn test_daily_note_detection() {
        let settings = temp_settings();
        let mut store = NoteStore::new(settings);

        let daily = Note::new("2026-10-13", "Today's notes");
        let regular = Note::new("Random Note", "Some content");

        store.notes.insert(daily.id.clone(), daily);
        store.notes.insert(regular.id.clone(), regular);

        let daily_notes = store.get_daily_notes();
        let non_daily = store.get_non_daily_notes();

        assert_eq!(daily_notes.len(), 1);
        assert_eq!(non_daily.len(), 1);
        assert!(daily_notes[0].name == "2026-10-13");
        assert!(non_daily[0].name == "Random Note");
    }

    #[test]
    fn test_find_note_by_name_creates_if_missing() {
        let settings = temp_settings();
        let mut store = NoteStore::new(settings);

        let id = store.find_note_by_name("New Note");
        assert!(!id.is_empty());
        assert!(store.get_note(&id).is_some());
        assert_eq!(store.get_note(&id).unwrap().name, "New Note");
    }

    #[test]
    fn test_delete_note() {
        let settings = temp_settings();
        let mut store = NoteStore::new(settings);

        let note = Note::new("ToDelete", "content");
        let id = note.id.clone();
        store.notes.insert(id.clone(), note);

        store.delete_note(&id).unwrap();
        assert!(store.get_note(&id).is_none());
    }

    #[test]
    fn test_get_or_create_daily_note() {
        let settings = temp_settings();
        let mut store = NoteStore::new(settings);

        let id1 = store.get_or_create_daily_note("2026-10-13");
        let id2 = store.get_or_create_daily_note("2026-10-13");

        // Should return the same ID
        assert_eq!(id1, id2);
        assert!(store.get_note(&id1).is_some());
    }
}
