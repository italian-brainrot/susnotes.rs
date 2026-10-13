use crate::ui::app::App;
use eframe::egui;

pub struct EditorPanel;

impl EditorPanel {
    pub fn show(ui: &mut egui::Ui, app: &mut App) {
        if let Some(ref id) = app.selected_note_id {
            let id_clone = id.clone();
            if let Some(note) = app.store.get_note(&id_clone) {
                let name = note.name.clone();
                let is_daily = note.is_daily();

                // Note name editor at the top
                ui.horizontal(|ui| {
                    let mut editable_name = name.clone();
                    let response = ui.text_edit_singleline(&mut editable_name);
                    if response.changed() {
                        let name_clone = editable_name.clone();
                        if let Some(note) = app.store.get_note_mut(&id_clone) {
                            note.update_name(&name_clone);
                        }
                    }
                    if !is_daily {
                        if ui.small_button("🗑").clicked() {
                            app.delete_current_note();
                            return;
                        }
                    }
                    ui.separator();
                    if ui.small_button("👁 View").clicked() {
                        app.editor_state = Some(crate::ui::app::EditorState::View);
                    }
                    if ui.small_button("✏️ Edit").clicked() {
                        app.editor_state = Some(crate::ui::app::EditorState::Edit);
                    }
                });
                ui.separator();

                match app.editor_state.clone() {
                    Some(crate::ui::app::EditorState::Edit) => {
                        Self::show_edit_mode(ui, app, &id_clone);
                    }
                    Some(crate::ui::app::EditorState::View) | None => {
                        Self::show_view_mode(ui, app, &id_clone);
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
        // Show hover text - need to get changed before consuming response
        let changed = response.changed();
        response.on_hover_text("Edit your markdown here. Use # for headings, **bold**, *italic*, etc.");

        // Auto-save on change
        if changed {
            if let Some(note) = app.store.get_note_mut(note_id) {
                note.update_content(&content);
            }
            app.mark_content_changed();
        }

        // Save button
        if ui.button("💾 Save").clicked() {
            if let Err(e) = app.store.update_and_save_note(note_id, &content) {
                log::warn!("Failed to save note: {}", e);
            }
            app.editor_state = Some(crate::ui::app::EditorState::View);
        }

    }

    fn show_view_mode(ui: &mut egui::Ui, app: &mut App, note_id: &str) {
        if let Some(note) = app.store.get_note(note_id) {
            if note.content.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("Nothing here yet.\nClick Edit to start writing.");
                });
                return;
            }

            let content = note.content.clone();
            Self::render_markdown_with_links(ui, app, &content);
        }

        // Handle pending note link
        if let Some(link) = app.pending_note_link.take() {
            app.handle_note_link(&link);
        }
    }

    /// Render markdown with note link support (#note_name and [[note name]])
    fn render_markdown_with_links(ui: &mut egui::Ui, app: &mut App, content: &str) {
        let parser = pulldown_cmark::Parser::new_ext(content, pulldown_cmark::Options::all());
        let mut in_code_block = false;
        let mut code_block_content = String::new();

        for event in parser {
            match event {
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading { level, .. }) => {
                    match level {
                        pulldown_cmark::HeadingLevel::H1 => { ui.heading(""); },
                        pulldown_cmark::HeadingLevel::H2 => { ui.strong(""); },
                        _ => { ui.label(""); },
                    }
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Heading(_)) => {}
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Paragraph) => {}
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Paragraph) => {
                    ui.separator();
                }
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::BlockQuote(_)) => {}
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::BlockQuote(_)) => {}
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::List(_)) => {}
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::List(_)) => {}
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Item) => {
                    ui.label("• ");
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Item) => {}
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_block_content.clear();
                    match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(_) => {}
                        pulldown_cmark::CodeBlockKind::Indented => {}
                    }
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                    in_code_block = false;
                    ui.horizontal(|ui| {
                        ui.code(code_block_content.clone());
                        if ui.small_button("📋 Copy").clicked() {
                            ui.ctx().copy_text(code_block_content.clone());
                        }
                    });
                }
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link { dest_url, .. }) => {
                    if ui.link(dest_url.to_string()).clicked() {
                        // Handle link click
                    }
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Link) => {}
                pulldown_cmark::Event::Text(text) => {
                    if in_code_block {
                        code_block_content.push_str(&text);
                    } else {
                        // Check for note links
                        let text = text.to_string();
                        let mut remaining = text.as_str();
                        let mut has_link = false;
                        let mut link_parts = Vec::new();

                        while let Some(pos) = remaining.find(|c: char| c == '#' || c == '[') {
                            let prefix = &remaining[..pos];
                            if !prefix.is_empty() {
                                link_parts.push(prefix.to_string());
                            }
                            remaining = &remaining[pos..];

                            if remaining.starts_with('#') {
                                let after_hash = &remaining[1..];
                                let end = after_hash.find(|c: char| !c.is_alphanumeric() && c != ' ' && c != '_').unwrap_or(after_hash.len());
                                let note_name = &after_hash[..end];
                                let link_text = &remaining[..end + 1];
                                link_parts.push(format!("[{}]({})", link_text, format!("#note:{}", note_name)));
                                remaining = &remaining[end + 1..];
                                has_link = true;
                            } else if remaining.starts_with("[[") {
                                let after_open = &remaining[2..];
                                let end = after_open.find("]]").map(|i| i + 2).unwrap_or(after_open.len());
                                let note_name = &after_open[..end];
                                let link_text = &remaining[..end + 4];
                                link_parts.push(format!("[{}]({})", link_text, format!("#note:{}", note_name)));
                                remaining = &remaining[end + 4..];
                                has_link = true;
                            }
                        }
                        if !remaining.is_empty() {
                            link_parts.push(remaining.to_string());
                        }

                        if has_link {
                            for part in link_parts {
                                if part.starts_with("[") && part.contains("(#note:") {
                                    if let Some(name_start) = part.find("#note:") {
                                        let name = &part[name_start + 6..part.len() - 1];
                                        if ui.link(&part[..part.len() - 1]).clicked() {
                                            app.pending_note_link = Some(name.to_string());
                                        }
                                    } else {
                                        ui.label(&part);
                                    }
                                } else {
                                    ui.label(&part);
                                }
                            }
                        } else {
                            ui.label(&text);
                        }
                    }
                }
                pulldown_cmark::Event::Code(code) => {
                    if in_code_block {
                        code_block_content.push_str(&code);
                    } else {
                        ui.code(code.to_string());
                    }
                }
                pulldown_cmark::Event::Html(html) => {
                    ui.label(html.to_string());
                }
                pulldown_cmark::Event::SoftBreak => {
                    if !in_code_block {
                        ui.label("");
                    }
                }
                pulldown_cmark::Event::HardBreak => {
                    if !in_code_block {
                        ui.separator();
                    }
                }
                pulldown_cmark::Event::Rule => {
                    ui.separator();
                }
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Emphasis) => {}
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Emphasis) => {}
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Strong) => {}
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Strong) => {}
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Strikethrough) => {}
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Strikethrough) => {}
                _ => {}
            }
        }
    }
}
