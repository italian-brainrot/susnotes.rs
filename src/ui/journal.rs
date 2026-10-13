use crate::ui::app::App;
use crate::ui::markdown::render_markdown;
use eframe::egui;

pub struct JournalPanel;

impl JournalPanel {
    pub fn show(ui: &mut egui::Ui, app: &mut App) {
        if let Some(ref mut journal_state) = app.journal_state {
            let today = journal_state.current_date.clone();

            // Date header
            ui.horizontal(|ui| {
                ui.strong(format!("📅 {}", today));
                if ui.small_button("✏️").clicked() {
                    app.editor_state = Some(crate::ui::app::EditorState::Edit);
                }
            });
            ui.separator();

            // Get or create daily note
            let note_id = app.store.get_or_create_daily_note(&today);

            // Show note content
            match app.editor_state.clone() {
                Some(crate::ui::app::EditorState::Edit) => {
                    Self::show_edit_mode(ui, app, &note_id);
                }
                Some(crate::ui::app::EditorState::View) | None => {
                    Self::show_view_mode(ui, app, &note_id);
                }
            }

            ui.separator();

            // Previous daily notes
            ui.strong("📜 Previous Days");
            ui.separator();

            let daily_notes: Vec<String> = app.store.get_daily_notes().iter().map(|n| n.id.clone()).collect();
            for id in &daily_notes {
                if let Some(note) = app.store.get_note(id) {
                    if note.name != today && !note.content.is_empty() {
                        let note_id = id.clone();
                        let note_content = note.content.clone();
                        let note_name = note.name.clone();
                        ui.collapsing(format!("📅 {}", note_name), |ui| {
                            ui.horizontal(|ui| {
                                if ui.small_button("👁 View").clicked() {
                                    app.select_note(&note_id);
                                    app.editor_state = Some(crate::ui::app::EditorState::View);
                                }
                                if ui.small_button("✏️ Edit").clicked() {
                                    app.select_note(&note_id);
                                    app.editor_state = Some(crate::ui::app::EditorState::Edit);
                                }
                            });
                            ui.separator();
                            render_markdown(ui, &note_content);
                            ui.separator();
                        });
                    }
                }
            }
        }
    }

    fn show_edit_mode(ui: &mut egui::Ui, app: &mut App, note_id: &str) {
        let mut content = String::new();
        if let Some(note) = app.store.get_note(note_id) {
            content = note.content.clone();
        }

        let response = ui.text_edit_multiline(&mut content);
        let changed = response.changed();

        if changed {
            if let Some(note) = app.store.get_note_mut(note_id) {
                note.update_content(&content);
            }
        }

        if ui.button("💾 Save & View").clicked() {
            if let Err(e) = app.store.update_and_save_note(note_id, &content) {
                log::warn!("Failed to save note: {}", e);
            }
            app.editor_state = Some(crate::ui::app::EditorState::View);
        }
    }

    fn show_view_mode(ui: &mut egui::Ui, app: &mut App, note_id: &str) {
        if let Some(note) = app.store.get_note(note_id) {
            if note.content.is_empty() {
                ui.label("Start writing your journal entry...");
                return;
            }
            render_markdown(ui, &note.content);
        }
    }
}
