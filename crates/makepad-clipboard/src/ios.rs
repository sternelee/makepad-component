//! iOS clipboard implementation

use objc2::msg_send_id;
use objc2_ui_kit::UIPasteboard;

use crate::{ClipboardContent, ClipboardEntry, ClipboardMonitor};

/// Read clipboard on iOS
pub fn read_clipboard(monitor: &ClipboardMonitor) -> Option<ClipboardEntry> {
    unsafe {
        let pasteboard = UIPasteboard::generalPasteboard();
        let change_count: i64 = msg_send_id![&pasteboard, changeCount];

        if change_count == monitor.get_change_count() {
            return None;
        }

        monitor.set_change_count(change_count);

        let mut formats = Vec::new();
        let mut content = ClipboardContent::Unknown;

        // Check for text
        if pasteboard.hasStrings() {
            formats.push("public.utf8-plain-text".to_string());
            if let Some(text) = pasteboard.string() {
                content = ClipboardContent::Text(text.to_string());
            }
        }

        // Check for URLs
        if pasteboard.hasURLs() {
            formats.push("public.url".to_string());
        }

        // Check for images
        if pasteboard.hasImages() {
            formats.push("public.png".to_string());
            formats.push("public.tiff".to_string());
        }

        if formats.is_empty() {
            return None;
        }

        // Get timestamp
        let timestamp = {
            use std::time::SystemTime;
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            let secs = now.as_secs();
            let hours = (secs / 3600) % 24;
            let minutes = (secs / 60) % 60;
            format!("{:02}:{:02}", hours, minutes)
        };

        Some(ClipboardEntry {
            content,
            timestamp,
            formats,
        })
    }
}
