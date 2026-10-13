use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: Theme,
    pub password_hash: Option<String>,
    pub password_salt: Option<String>,
    pub notes_folder: Option<PathBuf>,
    pub last_folder: Option<PathBuf>,
    pub window_width: f32,
    pub window_height: f32,
    pub sidebar_width: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            password_hash: None,
            password_salt: None,
            notes_folder: None,
            last_folder: None,
            window_width: 1200.0,
            window_height: 800.0,
            sidebar_width: 250.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn to_egui(&self) -> egui::ThemePreference {
        match self {
            Theme::Light => egui::ThemePreference::Light,
            Theme::Dark => egui::ThemePreference::Dark,
            Theme::System => egui::ThemePreference::System,
        }
    }
}

impl Settings {
    pub fn config_path() -> PathBuf {
        let dir = directories::BaseDirs::new()
            .map(|d| d.config_dir().join("susnotes"))
            .unwrap_or_else(|| PathBuf::from(".susnotes"));
        dir.join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(settings) => {
                            log::info!("Loaded settings from {}", path.display());
                            settings
                        }
                        Err(e) => {
                            log::warn!("Failed to parse settings: {}, using defaults", e);
                            Self::default()
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read settings: {}, using defaults", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| format!("Failed to serialize settings: {}", e))?;
        fs::write(&path, &content).map_err(|e| format!("Failed to save settings: {}", e))?;
        #[cfg(unix)]
        std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600))
            .unwrap_or_else(|e| log::warn!("Failed to set permissions on settings file: {}", e));
        log::info!("Saved settings to {}", path.display());
        Ok(())
    }

    pub fn ensure_notes_folder(&self) -> Result<PathBuf, String> {
        if let Some(ref folder) = self.notes_folder {
            if folder.exists() {
                return Ok(folder.clone());
            }
        }
        // Create default folder
        let default = directories::BaseDirs::new()
            .map(|d| d.home_dir().join("susnotes"))
            .unwrap_or_else(|| PathBuf::from("./susnotes"));
        fs::create_dir_all(&default).map_err(|e| format!("Failed to create notes folder: {}", e))?;
        Ok(default)
    }

    pub fn pages_folder(&self) -> PathBuf {
        self.notes_folder
            .clone()
            .unwrap_or_else(|| PathBuf::from("./susnotes"))
            .join("pages")
    }

    pub fn assets_folder(&self) -> PathBuf {
        self.notes_folder
            .clone()
            .unwrap_or_else(|| PathBuf::from("./susnotes"))
            .join("assets")
    }
}
