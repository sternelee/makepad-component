#![allow(dead_code)]
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub mood: Option<String>,
    pub timestamp: f64,
    pub date: String,
    pub image_cover: Option<String>,
    pub messages: Vec<Message>,
}

/// Summary info for listing memories (used by main.rs and memory page)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub date: String,
    pub timestamp: f64,
    pub mood: Option<String>,
}

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir).ok();
        let db_path = data_dir.join("gemini_talker.db");
        let conn = Connection::open(&db_path)?;
        let storage = Self { conn };
        storage.init_tables()?;
        Ok(storage)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                summary TEXT NOT NULL,
                mood TEXT,
                timestamp REAL NOT NULL,
                date TEXT NOT NULL,
                image_cover TEXT
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                memory_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp REAL NOT NULL,
                FOREIGN KEY (memory_id) REFERENCES memories(id)
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_memory ON messages(memory_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memories_date ON memories(date)",
            [],
        )?;
        Ok(())
    }

    pub fn save_memory(&self, memory: &Memory) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO memories (id, title, summary, mood, timestamp, date, image_cover)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                memory.id,
                memory.title,
                memory.summary,
                memory.mood,
                memory.timestamp,
                memory.date,
                memory.image_cover
            ],
        )?;

        self.conn.execute(
            "DELETE FROM messages WHERE memory_id = ?1",
            params![memory.id],
        )?;

        for msg in &memory.messages {
            self.conn.execute(
                "INSERT INTO messages (id, memory_id, role, content, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    format!("{}_{}", memory.id, msg.timestamp),
                    memory.id,
                    msg.role,
                    msg.content,
                    msg.timestamp
                ],
            )?;
        }
        Ok(())
    }

    pub fn load_memory(&self, id: &str) -> Result<Memory> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, summary, mood, timestamp, date, image_cover FROM memories WHERE id = ?1"
        )?;
        let memory = stmt.query_row(params![id], |row| {
            Ok(Memory {
                id: row.get(0)?,
                title: row.get(1)?,
                summary: row.get(2)?,
                mood: row.get(3)?,
                timestamp: row.get(4)?,
                date: row.get(5)?,
                image_cover: row.get(6)?,
                messages: Vec::new(),
            })
        })?;

        let mut msg_stmt = self.conn.prepare(
            "SELECT role, content, timestamp FROM messages WHERE memory_id = ?1 ORDER BY timestamp",
        )?;
        let messages = msg_stmt
            .query_map(params![id], |row| {
                Ok(Message {
                    role: row.get(0)?,
                    content: row.get(1)?,
                    timestamp: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(Memory { messages, ..memory })
    }

    pub fn list_memories(&self) -> Result<Vec<MemorySummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, summary, timestamp, date, mood FROM memories ORDER BY timestamp DESC",
        )?;
        let memories = stmt
            .query_map([], |row| {
                Ok(MemorySummary {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    summary: row.get(2)?,
                    timestamp: row.get(3)?,
                    date: row.get(4)?,
                    mood: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(memories)
    }

    pub fn delete_memory(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM messages WHERE memory_id = ?1", params![id])?;
        self.conn
            .execute("DELETE FROM memories WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_memories_by_month(&self, year: i32, month: u32) -> Result<Vec<MemorySummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, summary, timestamp, date, mood FROM memories 
             WHERE date LIKE ?1 ORDER BY timestamp DESC",
        )?;
        let pattern = format!("{:02}/%/{:02}", month, year % 100);
        let memories = stmt
            .query_map(params![pattern], |row| {
                Ok(MemorySummary {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    summary: row.get(2)?,
                    timestamp: row.get(3)?,
                    date: row.get(4)?,
                    mood: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(memories)
    }

    pub fn get_calendar_dates(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT date FROM memories ORDER BY timestamp DESC")?;
        let dates = stmt
            .query_map([], |row| {
                let full_date: String = row.get(0)?;
                Ok(full_date
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string())
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(dates)
    }

    pub fn summarize_with_ai(
        &self,
        messages: &[Message],
        api_key: &str,
    ) -> Result<(String, String, Option<String>)> {
        if messages.is_empty() {
            return Ok((
                "Empty Conversation".to_string(),
                "No messages".to_string(),
                None,
            ));
        }

        let conversation_text = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Summarize this conversation into a memory card.\n\nConversation:\n{}\n\nReturn in this exact format:\nTITLE: <short evocative title>\nSUMMARY: <warm natural summary>\nMOOD: <one word only or empty>",
            conversation_text
        );

        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent")
            .query(&[("key", api_key)])
            .json(&serde_json::json!({
                "contents": [{
                    "parts": [{"text": prompt}]
                }],
                "generationConfig": {
                    "temperature": 0.7,
                    "maxOutputTokens": 200
                }
            }))
            .send();

        match response {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>() {
                    if let Some(candidates) = json.get("candidates").and_then(|c| c.as_array()) {
                        if let Some(first) = candidates.first() {
                            if let Some(content) = first.get("content").and_then(|c| c.get("parts"))
                            {
                                if let Some(parts) = content.as_array() {
                                    if let Some(text) = parts.first().and_then(|p| p.get("text")) {
                                        if let Some(text_str) = text.as_str() {
                                            let mut title = "Conversation".to_string();
                                            let mut summary = "A memorable chat".to_string();
                                            let mut mood: Option<String> = None;

                                            for line in text_str.lines() {
                                                let line = line.trim();
                                                if line.starts_with("TITLE:") {
                                                    title =
                                                        line["TITLE:".len()..].trim().to_string();
                                                } else if line.starts_with("SUMMARY:") {
                                                    summary =
                                                        line["SUMMARY:".len()..].trim().to_string();
                                                } else if line.starts_with("MOOD:") {
                                                    let mood_str =
                                                        line["MOOD:".len()..].trim().to_string();
                                                    if !mood_str.is_empty() {
                                                        mood = Some(mood_str);
                                                    }
                                                }
                                            }
                                            return Ok((title, summary, mood));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                self.fallback_summarize(messages)
            }
            Err(_) => self.fallback_summarize(messages),
        }
    }

    fn fallback_summarize(&self, messages: &[Message]) -> Result<(String, String, Option<String>)> {
        let title = messages
            .iter()
            .find(|m| m.role == "user")
            .map(|m| {
                let text = m.content.chars().take(25).collect::<String>();
                if m.content.len() > 25 {
                    format!("{}...", text)
                } else {
                    text
                }
            })
            .unwrap_or_else(|| "Conversation".to_string());

        let summary = messages
            .iter()
            .take(4)
            .map(|m| m.content.chars().take(40).collect::<String>())
            .collect::<Vec<_>>()
            .join(" | ");

        Ok((title, summary, None))
    }
}
