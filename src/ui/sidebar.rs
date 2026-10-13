use crate::ui::app::App;
use eframe::egui;

pub struct SidebarPanel;

impl SidebarPanel {
    pub fn show(ui: &mut egui::Ui, app: &mut App, settings: &crate::config::Settings) {
        let _ = settings; // suppress unused warning

        // Search box
        ui.horizontal(|ui| {
            let mut search = String::new();
            ui.text_edit_singleline(&mut search);
            if !search.is_empty() {
                // Filter notes by search term
                let names: Vec<String> = app
                    .store
                    .get_all_names()
                    .into_iter()
                    .filter(|n| n.to_lowercase().contains(&search.to_lowercase()))
                    .collect();
                for name in names {
                    Self::note_item(ui, app, &name);
                }
            }
        });

        ui.separator();

        // Daily notes section
        ui.strong("📅 Daily Notes");
        let daily_notes: Vec<String> = app.store.get_daily_notes().iter().map(|n| n.id.clone()).collect();
        for id in &daily_notes {
            if let Some(note) = app.store.get_note(id) {
                if Self::note_item(ui, app, &note.name).clicked() {
                    app.select_note(id);
                }
            }
        }

        ui.separator();

        // All notes section
        ui.strong("📄 Notes");
        let notes: Vec<String> = app.store.get_non_daily_notes().iter().map(|n| n.id.clone()).collect();
        for id in &notes {
            if let Some(note) = app.store.get_note(id) {
                if Self::note_item(ui, app, &note.name).clicked() {
                    app.select_note(id);
                }
            }
        }

        if notes.is_empty() && daily_notes.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No notes yet.\nCreate one to get started!");
            });
        }
    }

    fn note_item(ui: &mut egui::Ui, app: &App, name: &str) -> egui::Response {
        let is_selected = app
            .selected_note_id
            .as_ref()
            .map(|id| app.store.get_note(id).map(|n| n.name == name).unwrap_or(false))
            .unwrap_or(false);

        let response = ui.horizontal(|ui| {
            if is_selected {
                ui.visuals_mut().override_text_color = Some(ui.style().visuals.selection.stroke.color);
            }
            let icon = if name.starts_with("20") && name.len() == 10 {
                "📅"
            } else {
                "📄"
            };
            ui.label(icon);
            ui.label(name);
        });

        response.response
    }
}
