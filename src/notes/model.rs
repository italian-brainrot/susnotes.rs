use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new(name: &str, content: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            content: content.to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_content(&mut self, content: &str) {
        self.content = content.to_string();
        self.updated_at = Utc::now();
    }

    pub fn update_name(&mut self, name: &str) {
        self.name = name.to_string();
        self.updated_at = Utc::now();
    }

    /// Check if this is a daily journal note
    pub fn is_daily(&self) -> bool {
        chrono::NaiveDate::parse_from_str(&self.name, "%Y-%m-%d").is_ok()
    }

    /// Extract note links from content (#note_name or [[note name]])
    #[allow(dead_code)]
    pub fn extract_links(&self) -> Vec<String> {
        let mut links = Vec::new();
        // Match [[note name]]
        for cap in regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap().find_iter(&self.content) {
            links.push(cap.as_str().to_string());
        }
        // Match #note_name (word characters and spaces)
        for cap in regex::Regex::new(r"#([A-Za-z0-9_][A-Za-z0-9_ ]{0,99})")
            .unwrap()
            .find_iter(&self.content)
        {
            let link = cap.as_str().to_string();
            if !links.contains(&link) {
                links.push(link);
            }
        }
        links
    }
}
