use crate::config::{Settings, Theme};
use crate::notes::NoteStore;
use crate::ui::{editor::EditorPanel, journal::JournalPanel, sidebar::SidebarPanel, password_dialog::PasswordDialog};
use eframe::egui;

pub struct App {
    pub settings: Settings,
    pub store: NoteStore,
    pub selected_note_id: Option<String>,
    pub show_journal: bool,
    pub show_settings: bool,
    pub show_password_dialog: bool,
    pub password_dialog_state: Option<PasswordDialog>,
    pub sidebar_width: f32,
    pub editor_state: Option<EditorState>,
    pub journal_state: Option<JournalState>,
    pub _pending_note_link: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EditorState {
    View,
    Edit,
}

#[derive(Debug, Clone)]
pub struct JournalState {
    pub current_date: String,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();

        let store = NoteStore::new(settings.clone());

        // If password is set, we need to verify it
        let needs_password = store.get_settings().password_hash.is_some();

        let mut app = App {
            settings: settings.clone(),
            store,
            selected_note_id: None,
            show_journal: false,
            show_settings: false,
            show_password_dialog: needs_password,
            password_dialog_state: if needs_password {
                Some(PasswordDialog::new())
            } else {
                None
            },
            sidebar_width: settings.sidebar_width,
            editor_state: None,
            journal_state: None,
            _pending_note_link: None,
        };

        if !needs_password {
            app.ensure_folders();
            let _ = app.store.load_all();
        }

        app
    }

    fn ensure_folders(&mut self) {
        if let Err(e) = self.store.ensure_folders() {
            log::warn!("Failed to ensure folders: {}", e);
        }
    }

    #[allow(dead_code)]
    pub fn verify_password(&mut self, password: &str) -> bool {
        if self.store.check_password(password) {
            self.store.set_password(password);
            let _ = self.store.load_all();
            self.show_password_dialog = false;
            self.password_dialog_state = None;
            true
        } else {
            false
        }
    }

    fn set_theme(&mut self, theme: Theme) {
        self.settings.theme = theme;
        if let Err(e) = self.settings.save() {
            log::warn!("Failed to save settings: {}", e);
        }
    }

    pub fn select_note(&mut self, id: &str) {
        self.selected_note_id = Some(id.to_string());
        self.show_journal = false;
    }

    pub fn create_note(&mut self, name: &str) {
        let id = self.store.find_note_by_name(name);
        self.selected_note_id = Some(id);
        self.show_journal = false;
    }

    pub fn open_journal(&mut self) {
        self.show_journal = true;
        self.selected_note_id = None;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.journal_state = Some(JournalState {
            current_date: today.clone(),
        });
        // Ensure daily note exists
        self.store.get_or_create_daily_note(&today);
    }

    pub fn save_current_note(&mut self) {
        if let Some(ref id) = self.selected_note_id {
            let id_clone = id.clone();
            if let Some(note) = self.store.get_note(&id_clone) {
                let content = note.content.clone();
                if let Err(e) = self.store.update_and_save_note(&id_clone, &content) {
                    log::warn!("Failed to save note: {}", e);
                }
            }
        }
    }

    pub fn delete_current_note(&mut self) {
        if let Some(ref id) = self.selected_note_id {
            let id_clone = id.clone();
            if let Err(e) = self.store.delete_note(&id_clone) {
                log::warn!("Failed to delete note: {}", e);
            }
            self.selected_note_id = None;
            self.editor_state = None;
        }
    }

    pub fn handle_note_link(&mut self, link: &str) {
        // Remove # or [[ ]] wrapper
        let name = link.trim_start_matches('#')
            .trim_start_matches("[[")
            .trim_end_matches("]]")
            .trim()
            .to_string();

        if !name.is_empty() {
            let id = self.store.find_note_by_name(&name);
            self.selected_note_id = Some(id);
            self.show_journal = false;
        }
    }

    #[allow(dead_code)]
    pub fn paste_image(&mut self, ctx: &egui::Context, image_bytes: Vec<u8>) {
        if let Some(ref id) = self.selected_note_id {
            let id_clone = id.clone();
            let filename = format!("{}.png", uuid::Uuid::new_v4());

            // Save image first
            match self.store.save_image(&id_clone, &image_bytes, &filename) {
                Ok(asset_path) => {
                    // Now update the note content
                    if let Some(note) = self.store.get_note(&id_clone) {
                        let img_markdown = format!("\n![{}]({})\n", filename, asset_path);
                        let new_content = format!("{}{}", note.content, img_markdown);
                        if let Err(e) = self.store.update_and_save_note(&id_clone, &new_content) {
                            log::warn!("Failed to save note after image paste: {}", e);
                        }
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("susnotes - {}", id_clone)));
                }
                Err(e) => {
                    log::error!("Failed to save image: {}", e);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title("susnotes - Error".to_string()));
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle theme
        let theme = self.settings.theme.to_egui();
        ctx.set_theme(theme);

        // Handle password dialog
        if self.show_password_dialog {
            if let Some(ref mut dialog) = self.password_dialog_state {
                dialog.show(ctx, &mut self.store);
                if !self.show_password_dialog {
                    self.ensure_folders();
                    let _ = self.store.load_all();
                }
            }
            return;
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("📝 susnotes");
                ui.separator();

                if ui.button("📓 Journal").clicked() {
                    self.open_journal();
                }

                if ui.button("➕ New Note").clicked() {
                    let name = format!("Note {}", self.store.get_all_names().len() + 1);
                    self.create_note(&name);
                }

                ui.separator();

                if ui.button("🗑 Delete").clicked() && self.selected_note_id.is_some() {
                    self.delete_current_note();
                }

                ui.separator();

                if ui.button("⚙ Settings").clicked() {
                    self.show_settings = !self.show_settings;
                }
            });
        });

        // Sidebar
        egui::SidePanel::left("sidebar")
            .default_width(self.sidebar_width)
            .min_width(150.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    ui.label("Notes");
                });
                ui.separator();

                // Clone settings to avoid borrow issues
                let settings = self.settings.clone();
                SidebarPanel::show(ui, self, &settings);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_journal {
                JournalPanel::show(ui, self);
            } else if let Some(ref id) = self.selected_note_id {
                if self.store.get_note(id).is_some() {
                    EditorPanel::show(ui, self);
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select or create a note to get started");
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Welcome to susnotes");
                        ui.separator();
                        ui.label("Select a note from the sidebar or create a new one.");
                        ui.label("Or open the Journal to start taking notes.");
                        ui.separator();
                        if ui.button("Open Journal").clicked() {
                            self.open_journal();
                        }
                    });
                });
            }
        });

        // Settings panel
        if self.show_settings {
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::RIGHT_TOP, (0.0, 0.0))
                .show(ctx, |ui| {
                    ui.heading("Settings");
                    ui.separator();

                    ui.label("Theme:");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.settings.theme == Theme::Light, "☀️ Light").clicked() {
                            self.set_theme(Theme::Light);
                        }
                        if ui.selectable_label(self.settings.theme == Theme::Dark, "🌙 Dark").clicked() {
                            self.set_theme(Theme::Dark);
                        }
                        if ui.selectable_label(self.settings.theme == Theme::System, "💻 System").clicked() {
                            self.set_theme(Theme::System);
                        }
                    });

                    ui.separator();

                    if ui.button("Change Password").clicked() {
                        self.show_password_dialog = true;
                        self.password_dialog_state = Some(PasswordDialog::new());
                    }

                    ui.separator();

                    if ui.button("Close").clicked() {
                        self.show_settings = false;
                    }
                });
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        if let Err(e) = self.settings.save() {
            log::warn!("Failed to save settings: {}", e);
        }
        self.save_current_note();
    }
}
