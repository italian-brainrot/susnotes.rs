use eframe::egui;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

/// Render markdown content in an egui UI.
/// Returns true if any note links were clicked.
pub fn render_markdown(ui: &mut egui::Ui, content: &str) {
    let parser = Parser::new_ext(content, Options::all());
    let mut in_code_block = false;
    let mut code_block_content = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                match level {
                    pulldown_cmark::HeadingLevel::H1 => { ui.heading(""); },
                    pulldown_cmark::HeadingLevel::H2 => { ui.strong(""); },
                    _ => { ui.label(""); },
                }
            }
            Event::End(TagEnd::Heading(_)) => {}
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                ui.separator();
            }
            Event::Start(Tag::BlockQuote(_)) => {}
            Event::End(TagEnd::BlockQuote(_)) => {}
            Event::Start(Tag::List(_)) => {}
            Event::End(TagEnd::List(_)) => {}
            Event::Start(Tag::Item) => {
                ui.label("• ");
            }
            Event::End(TagEnd::Item) => {}
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_block_content.clear();
                match kind {
                    CodeBlockKind::Fenced(_) => {}
                    CodeBlockKind::Indented => {}
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                ui.horizontal(|ui| {
                    ui.code(code_block_content.clone());
                    if ui.small_button("📋 Copy").clicked() {
                        ui.ctx().copy_text(code_block_content.clone());
                    }
                });
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                if ui.link(dest_url.to_string()).clicked() {
                    // Handle link click
                }
            }
            Event::End(TagEnd::Link) => {}
            Event::Text(text) => {
                if in_code_block {
                    code_block_content.push_str(&text);
                } else {
                    ui.label(text.to_string());
                }
            }
            Event::Code(code) => {
                if in_code_block {
                    code_block_content.push_str(&code);
                } else {
                    ui.code(code.to_string());
                }
            }
            Event::Html(html) => {
                ui.label(html.to_string());
            }
            Event::SoftBreak => {
                if !in_code_block {
                    ui.label("");
                }
            }
            Event::HardBreak => {
                if !in_code_block {
                    ui.separator();
                }
            }
            Event::Rule => {
                ui.separator();
            }
            Event::Start(Tag::Emphasis) => {}
            Event::End(TagEnd::Emphasis) => {}
            Event::Start(Tag::Strong) => {}
            Event::End(TagEnd::Strong) => {}
            Event::Start(Tag::Strikethrough) => {}
            Event::End(TagEnd::Strikethrough) => {}
            _ => {}
        }
    }
}
