//! Memory Module - Save and load conversation memories
//!
//! Handles persisting conversation history as memories that can be viewed later

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A single conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,      // "user" or "model"
    pub content: String,   // Message text
    pub timestamp: f64,    // Unix timestamp
}

/// A saved memory/ conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub timestamp: f64,
    pub date: String,           // "04/12/25 10:49AM"
    pub image_cover: Option<String>,  // Base64 encoded image or file path
    pub messages: Vec<Message>,
}

/// Memory manager - handles saving/loading memories
pub struct MemoryManager {
    storage_dir: PathBuf,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(data_dir: PathBuf) -> Self {
        let storage_dir = data_dir.join("memories");
        Self { storage_dir }
    }

    /// Initialize storage directory
    pub fn init(&self) -> Result<(), String> {
        fs::create_dir_all(&self.storage_dir)
            .map_err(|e| format!("Failed to create memories directory: {}", e))
    }

    /// Get the storage directory path
    pub fn storage_dir(&self) -> &PathBuf {
        &self.storage_dir
    }

    /// Save a memory
    pub fn save_memory(&self, memory: &Memory) -> Result<(), String> {
        let filename = format!("{}.json", memory.id);
        let path = self.storage_dir.join(&filename);

        let json = serde_json::to_string_pretty(memory)
            .map_err(|e| format!("Failed to serialize memory: {}", e))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write memory file: {}", e))?;

        log::info!("Saved memory: {}", memory.id);
        Ok(())
    }

    /// Load a specific memory by ID
    pub fn load_memory(&self, id: &str) -> Result<Memory, String> {
        let filename = format!("{}.json", id);
        let path = self.storage_dir.join(&filename);

        let json = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read memory file: {}", e))?;

        let memory: Memory = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse memory file: {}", e))?;

        Ok(memory)
    }

    /// List all saved memories (returns IDs and basic info)
    pub fn list_memories(&self) -> Result<Vec<MemorySummary>, String> {
        let mut memories = Vec::new();

        if !self.storage_dir.exists() {
            return Ok(memories);
        }

        for entry in fs::read_dir(&self.storage_dir)
            .map_err(|e| format!("Failed to read memories directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(memory) = serde_json::from_str::<Memory>(&json) {
                        memories.push(MemorySummary {
                            id: memory.id.clone(),
                            title: memory.title.clone(),
                            summary: memory.summary.clone(),
                            date: memory.date.clone(),
                            timestamp: memory.timestamp,
                        });
                    }
                }
            }
        }

        // Sort by timestamp, newest first
        memories.sort_by(|a, b| b.timestamp.partial_cmp(&a.timestamp).unwrap_or(std::cmp::Ordering::Equal));

        Ok(memories)
    }

    /// Delete a memory by ID
    pub fn delete_memory(&self, id: &str) -> Result<(), String> {
        let filename = format!("{}.json", id);
        let path = self.storage_dir.join(&filename);

        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete memory: {}", e))?;

        log::info!("Deleted memory: {}", id);
        Ok(())
    }

    /// Get memories for a specific date
    pub fn get_memories_by_date(&self, date: &str) -> Result<Vec<MemorySummary>, String> {
        let all = self.list_memories()?;
        Ok(all.into_iter().filter(|m| m.date.starts_with(date)).collect())
    }

    /// Get all dates that have memories (for calendar view)
    pub fn get_memory_dates(&self) -> Result<Vec<String>, String> {
        let memories = self.list_memories()?;
        let mut dates: Vec<String> = memories
            .iter()
            .map(|m| {
                // Extract just the date portion (e.g., "04/12/25")
                m.date.split_whitespace().next().unwrap_or("").to_string()
            })
            .collect();
        dates.sort();
        dates.dedup();
        Ok(dates)
    }
}

/// Summary info for listing memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub date: String,
    pub timestamp: f64,
}

/// Helper to create a new memory from conversation messages
pub fn create_memory(
    messages: &[Message],
    image_cover: Option<String>,
) -> Memory {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    // Generate a simple ID
    let id = format!("memory_{}", now as u64);

    // Generate title from first user message or default
    let title = messages
        .iter()
        .find(|m| m.role == "user")
        .map(|m| {
            let text = m.content.chars().take(30).collect::<String>();
            if m.content.len() > 30 {
                format!("{}...", text)
            } else {
                text
            }
        })
        .unwrap_or_else(|| "Conversation".to_string());

    // Generate summary (simplified - just concatenate first few messages)
    let summary = messages
        .iter()
        .take(3)
        .map(|m| m.content.chars().take(50).collect::<String>())
        .collect::<Vec<_>>()
        .join(" | ");

    // Format date string
    let date = format_timestamp(now);

    Memory {
        id,
        title,
        summary,
        timestamp: now,
        date,
        image_cover,
        messages: messages.to_vec(),
    }
}

/// Format timestamp to display string
pub fn format_timestamp(timestamp: f64) -> String {
    // This is a simplified version - in production you'd use chrono or similar
    let total_seconds = timestamp as u64;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;

    let hour_12 = if hours == 0 {
        12
    } else if hours > 12 {
        hours - 12
    } else {
        hours
    };
    let am_pm = if hours >= 12 { "PM" } else { "AM" };

    format!("{:02}/{:02}/{:02} {:02}:{:02}{}",
        (total_seconds / 86400 / 30) % 12 + 1,
        (total_seconds / 86400) % 30 + 1,
        (total_seconds / 86400 / 365) % 100,
        hour_12,
        minutes,
        am_pm
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp() {
        let result = format_timestamp(0.0);
        assert!(result.contains("AM"));
    }
}