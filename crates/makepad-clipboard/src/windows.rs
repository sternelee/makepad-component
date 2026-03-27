//! Windows clipboard implementation

use crate::{ClipboardContent, ClipboardEntry, ClipboardMonitor};

#[cfg(target_os = "windows")]
pub fn read_clipboard(monitor: &ClipboardMonitor) -> Option<ClipboardEntry> {
    use std::ptr::null_mut;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::System::Rendering::Win32::Graphics::Gdi::{
        CreateBitmap, DeleteObject, GetDIBits, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS,
    };

    unsafe {
        // Open clipboard
        if OpenClipboard(HWND(null_mut())).is_err() {
            return None;
        }

        // Get clipboard sequence number to detect changes
        // Note: Windows doesn't have a direct sequence number like macOS
        // We'll use CF_TEXT to check for changes

        let mut formats = Vec::new();
        let mut content = ClipboardContent::Unknown;

        // Check for various formats
        let cf_text = 1i32; // CF_TEXT
        let cf_unicode = 13i32; // CF_UNICODETEXT
        let cf_bitmap = 2i32; // CF_BITMAP
        let cf_dib = 8i32; // CF_DIB

        // Try to get text (Unicode first, then ANSI)
        let text_data = GetClipboardData(windows::Win32::System::DataExchange::CLIPFORMAT(cf_unicode));
        if !text_data.is_null() {
            formats.push("text/plain;charset=utf-16".to_string());
            let text = {
                let ptr = GlobalLock(text_data as *mut _);
                if !ptr.is_null() {
                    let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
                    let slice = std::slice::from_raw_parts(ptr as *const u16, len);
                    String::from_utf16_lossy(slice)
                } else {
                    GlobalUnlock(text_data);
                    String::new()
                }
            };
            if !text.is_empty() {
                content = ClipboardContent::Text(text);
            }
        }

        // Try CF_TEXT as fallback
        if matches!(content, ClipboardContent::Unknown) {
            let text_data = GetClipboardData(windows::Win32::System::DataExchange::CLIPFORMAT(cf_text));
            if !text_data.is_null() {
                formats.push("text/plain".to_string());
                let text = {
                    let ptr = GlobalLock(text_data as *mut _);
                    if !ptr.is_null() {
                        let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
                        String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).to_string()
                    } else {
                        String::new()
                    }
                };
                if !text.is_empty() {
                    content = ClipboardContent::Text(text);
                }
            }
        }

        // Check for bitmap image
        if matches!(content, ClipboardContent::Unknown) {
            let bitmap_data = GetClipboardData(windows::Win32::System::DataExchange::CLIPFORMAT(cf_bitmap));
            if !bitmap_data.is_null() {
                formats.push("image/bmp".to_string());
                // For now, just indicate it's an image without extracting pixels
                content = ClipboardContent::Image {
                    width: 0,
                    height: 0,
                    data: vec![],
                };
            }
        }

        // Check for DIB (device independent bitmap)
        if matches!(content, ClipboardContent::Unknown) {
            let dib_data = GetClipboardData(windows::Win32::System::DataExchange::CLIPFORMAT(cf_dib));
            if !dib_data.is_null() {
                formats.push("image/dib".to_string());
                content = ClipboardContent::Image {
                    width: 0,
                    height: 0,
                    data: vec![],
                };
            }
        }

        let _ = CloseClipboard();

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
