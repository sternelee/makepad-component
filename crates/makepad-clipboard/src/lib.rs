//! Makepad Clipboard - Cross-platform clipboard implementation
//! Based on super_native_extensions approach

#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Clipboard content types
#[derive(Clone, Debug, PartialEq)]
pub enum ClipboardContent {
    Text(String),
    Image { width: u32, height: u32, data: Vec<u8> },
    Html(String),
    Files(Vec<String>),
    Unknown,
}

/// A clipboard entry with content and metadata
#[derive(Clone, Debug)]
pub struct ClipboardEntry {
    pub content: ClipboardContent,
    pub timestamp: String,
    pub formats: Vec<String>,
}

/// Clipboard monitor for watching clipboard changes
pub struct ClipboardMonitor {
    entries: Arc<Mutex<VecDeque<ClipboardEntry>>>,
    is_running: Arc<Mutex<bool>>,
    last_change_count: Arc<Mutex<i64>>,
}

impl ClipboardMonitor {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
            is_running: Arc::new(Mutex::new(false)),
            last_change_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn start_monitoring(&self) {
        if let Ok(mut running) = self.is_running.lock() {
            *running = true;
        }
    }

    pub fn stop_monitoring(&self) {
        if let Ok(mut running) = self.is_running.lock() {
            *running = false;
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.lock().map(|r| *r).unwrap_or(false)
    }

    pub fn get_entries(&self) -> Vec<ClipboardEntry> {
        self.entries.lock().map(|e| e.iter().cloned().collect()).unwrap_or_default()
    }

    pub fn add_entry(&self, entry: ClipboardEntry) {
        if let Ok(mut entries) = self.entries.lock() {
            // Don't add duplicate entries (same content as last one)
            if let Some(last) = entries.front() {
                if last.content == entry.content {
                    return;
                }
            }
            entries.push_front(entry);
            while entries.len() > 100 {
                entries.pop_back();
            }
        }
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }

    pub fn get_change_count(&self) -> i64 {
        self.last_change_count.lock().map(|c| *c).unwrap_or(0)
    }

    pub fn set_change_count(&self, count: i64) {
        if let Ok(mut c) = self.last_change_count.lock() {
            *c = count;
        }
    }

    fn get_timestamp() -> String {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs();
        let hours = (secs / 3600) % 24;
        let minutes = (secs / 60) % 60;
        format!("{:02}:{:02}", hours, minutes)
    }

    /// Poll for clipboard changes - platform specific implementation
    pub fn poll_clipboard(&self) -> Option<ClipboardEntry> {
        #[cfg(target_os = "macos")]
        {
            self.poll_macos()
        }

        #[cfg(target_os = "ios")]
        {
            self.poll_ios()
        }

        #[cfg(target_os = "android")]
        {
            self.poll_android()
        }

        #[cfg(target_os = "linux")]
        {
            self.poll_linux()
        }

        #[cfg(target_os = "windows")]
        {
            self.poll_windows()
        }

        #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "android", target_os = "linux", target_os = "windows")))]
        {
            None
        }
    }

    #[cfg(target_os = "macos")]
    fn poll_macos(&self) -> Option<ClipboardEntry> {
        use crate::macos::read_clipboard;
        read_clipboard(self)
    }

    #[cfg(target_os = "ios")]
    fn poll_ios(&self) -> Option<ClipboardEntry> {
        use crate::ios::read_clipboard;
        read_clipboard(self)
    }

    #[cfg(target_os = "android")]
    fn poll_android(&self) -> Option<ClipboardEntry> {
        None
    }

    #[cfg(target_os = "linux")]
    fn poll_linux(&self) -> Option<ClipboardEntry> {
        use crate::linux::read_clipboard;
        read_clipboard(self)
    }

    #[cfg(target_os = "windows")]
    fn poll_windows(&self) -> Option<ClipboardEntry> {
        use crate::windows::read_clipboard;
        read_clipboard(self)
    }
}

impl Default for ClipboardMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// Platform-specific implementations
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// Initialize the clipboard system
pub fn init() {
    #[cfg(target_os = "macos")]
    {
        log::info!("Clipboard system initialized (macOS)");
    }
    #[cfg(target_os = "ios")]
    {
        log::info!("Clipboard system initialized (iOS)");
    }
    #[cfg(target_os = "linux")]
    {
        log::info!("Clipboard system initialized (Linux)");
    }
    #[cfg(target_os = "windows")]
    {
        log::info!("Clipboard system initialized (Windows)");
    }
}
