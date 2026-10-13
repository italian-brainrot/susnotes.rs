use crate::notes::NoteStore;
use eframe::egui;

pub struct PasswordDialog {
    mode: PasswordMode,
    password: String,
    confirm_password: String,
    old_password: String,
    error: String,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum PasswordMode {
    Enter,
    Create,
    Change,
}

impl PasswordDialog {
    pub fn new() -> Self {
        Self {
            mode: PasswordMode::Enter,
            password: String::new(),
            confirm_password: String::new(),
            old_password: String::new(),
            error: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, store: &mut NoteStore) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("🔐 susnotes");
                ui.separator();

                match self.mode {
                    PasswordMode::Enter => {
                        ui.label("Enter your password to unlock your notes.");
                    }
                    PasswordMode::Create => {
                        ui.label("Create a password to encrypt your notes.");
                        ui.label("This password cannot be recovered if lost.");
                    }
                    PasswordMode::Change => {
                        ui.label("Change your password.");
                        // Old password input
                        egui::TextEdit::singleline(&mut self.old_password)
                            .password(true)
                            .hint_text("Current password")
                            .desired_width(250.0)
                            .show(ui);
                    }
                }

                ui.separator();

                // Password input
                egui::TextEdit::singleline(&mut self.password)
                    .password(true)
                    .hint_text("Password")
                    .desired_width(250.0)
                    .show(ui);

                // Confirm password (for create/change)
                if self.mode != PasswordMode::Enter {
                    egui::TextEdit::singleline(&mut self.confirm_password)
                        .password(true)
                        .hint_text("Confirm password")
                        .desired_width(250.0)
                        .show(ui);
                }

                if !self.error.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.error);
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Submit").clicked() {
                        self.submit(store);
                    }

                    if self.mode == PasswordMode::Create {
                        if ui.button("Cancel").clicked() {
                            self.mode = PasswordMode::Enter;
                            self.password.clear();
                            self.confirm_password.clear();
                            self.error.clear();
                        }
                    }

                    if self.mode == PasswordMode::Enter {
                        if ui.button("Create new vault").clicked() {
                            self.mode = PasswordMode::Create;
                            self.password.clear();
                            self.confirm_password.clear();
                            self.error.clear();
                        }
                    }
                });

                // Help text
                ui.separator();
                if self.mode == PasswordMode::Create {
                    ui.small("💡 Leave both fields empty to create a vault without a password (less secure). Note: existing encrypted notes will be lost.");
                } else if self.mode == PasswordMode::Enter {
                    ui.small("💡 Forgot your password? Click \"Create new vault\" to start fresh. Note: existing encrypted notes will be lost.");
                }
            });
        });
    }

    fn submit(&mut self, store: &mut NoteStore) {
        self.error.clear();

        match self.mode {
            PasswordMode::Enter => {
                if store.check_password(&self.password) {
                    store.set_password(&self.password);
                    let _ = store.load_all();
                    self.mode = PasswordMode::Enter;
                    self.password.clear();
                    self.confirm_password.clear();
                } else {
                    self.error = "Incorrect password".to_string();
                }
            }
            PasswordMode::Create => {
                // Empty password is allowed (no encryption)
                if !self.password.is_empty() && self.password.len() < 4 {
                    self.error = "Password must be at least 4 characters".to_string();
                    return;
                }
                if !self.password.is_empty() && self.password != self.confirm_password {
                    self.error = "Passwords do not match".to_string();
                    return;
                }
                if self.password.is_empty() {
                    // No password: clear any existing hash and proceed
                    store.clear_password();
                } else {
                    store.set_initial_password(&self.password);
                }
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                store.get_or_create_daily_note(&today);
                self.mode = PasswordMode::Enter;
                self.password.clear();
                self.confirm_password.clear();
            }
            PasswordMode::Change => {
                if self.old_password.is_empty() {
                    self.error = "Please enter your current password".to_string();
                    return;
                }
                if !store.check_password(&self.old_password) {
                    self.error = "Current password is incorrect".to_string();
                    return;
                }
                if self.password.len() < 4 {
                    self.error = "Password must be at least 4 characters".to_string();
                    return;
                }
                if self.password != self.confirm_password {
                    self.error = "Passwords do not match".to_string();
                    return;
                }
                if let Err(e) = store.change_password(&self.old_password, &self.password) {
                    self.error = e;
                    return;
                }
                self.mode = PasswordMode::Enter;
                self.password.clear();
                self.confirm_password.clear();
                self.old_password.clear();
            }
        }
    }
}
