//! macOS clipboard implementation with text and image support

use objc2::{msg_send_id, rc::Id};
use objc2_app_kit::NSPasteboard;
use objc2_foundation::{NSArray, NSString};

use crate::{ClipboardContent, ClipboardEntry, ClipboardMonitor};

/// Read clipboard on macOS
pub fn read_clipboard(monitor: &ClipboardMonitor) -> Option<ClipboardEntry> {
    unsafe {
        let pasteboard: Id<NSPasteboard> = NSPasteboard::generalPasteboard();
        let change_count: isize = pasteboard.changeCount();

        if change_count as i64 == monitor.get_change_count() {
            return None;
        }

        monitor.set_change_count(change_count as i64);

        // Get available types
        let types: Id<NSArray<NSString>> = msg_send_id![&pasteboard, types];

        // Convert NSArray to Vec<String>
        let count = types.count();
        let mut formats = Vec::new();
        for i in 0..count {
            let ty: Id<NSString> = msg_send_id![&types, objectAtIndex: i];
            formats.push(ty.to_string());
        }

        if formats.is_empty() {
            return None;
        }

        let mut content = ClipboardContent::Unknown;

        // Check for image types
        let image_types = [
            "public.image",
            "public.png",
            "public.tiff",
            "public.jpeg",
            "public.gif",
            "public.bmp",
            "com.apple.pict",
            "public.heic",
        ];

        let has_image_type = formats.iter().any(|f| {
            image_types.iter().any(|t| f.contains(t) || f == *t)
        });

        if has_image_type {
            // Get image data and dimensions
            let (width, height, data) = get_image_data(&pasteboard);
            let final_width = if width > 0 { width } else { 800 };
            let final_height = if height > 0 { height } else { 600 };
            content = ClipboardContent::Image {
                width: final_width,
                height: final_height,
                data,
            };
        }

        // Try to get text if no image
        if matches!(content, ClipboardContent::Unknown) {
            let text_type = NSString::from_str("public.utf8-plain-text");
            if let Some(text) = pasteboard.stringForType(&text_type) {
                content = ClipboardContent::Text(text.to_string());
            } else {
                // Try plain text fallback
                let plain_type = NSString::from_str("public.plain-text");
                if let Some(text) = pasteboard.stringForType(&plain_type) {
                    content = ClipboardContent::Text(text.to_string());
                }
            }
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

/// Get image data from pasteboard
#[cfg(target_os = "macos")]
unsafe fn get_image_data(pasteboard: &NSPasteboard) -> (u32, u32, Vec<u8>) {
    // Try PNG first
    let png_type = NSString::from_str("public.png");
    if let Some(data) = pasteboard.dataForType(&png_type) {
        let bytes = data.bytes();
        let data_vec = bytes.to_vec();

        if data_vec.len() >= 24 {
            let ptr = bytes.as_ptr() as *const u8;
            if *ptr.offset(0) == 0x89 && *ptr.offset(1) == 0x50 &&
               *ptr.offset(2) == 0x4E && *ptr.offset(3) == 0x47 {
                let width = ((*ptr.offset(16) as u32) << 24) |
                           ((*ptr.offset(17) as u32) << 16) |
                           ((*ptr.offset(18) as u32) << 8) |
                           (*ptr.offset(19) as u32);
                let height = ((*ptr.offset(20) as u32) << 24) |
                            ((*ptr.offset(21) as u32) << 16) |
                            ((*ptr.offset(22) as u32) << 8) |
                            (*ptr.offset(23) as u32);
                if width > 0 && height > 0 && width < 32768 && height < 32768 {
                    return (width, height, data_vec);
                }
            }
        }
    }

    // Try TIFF
    let tiff_type = NSString::from_str("public.tiff");
    if let Some(data) = pasteboard.dataForType(&tiff_type) {
        let bytes = data.bytes();
        let data_vec = bytes.to_vec();

        if data_vec.len() > 100 {
            let ptr = bytes.as_ptr() as *const u8;
            // Little endian
            if *ptr == 0x49 && *ptr.offset(1) == 0x49 {
                let width = (*ptr.offset(42) as u32) | ((*ptr.offset(43) as u32) << 8);
                let height = (*ptr.offset(38) as u32) | ((*ptr.offset(39) as u32) << 8);
                if width > 0 && height > 0 && width < 32768 && height < 32768 {
                    return (width, height, data_vec);
                }
            }
            // Big endian
            if *ptr == 0x4D && *ptr.offset(1) == 0x4D {
                let width = ((*ptr.offset(42) as u32) << 8) | *ptr.offset(43) as u32;
                let height = ((*ptr.offset(38) as u32) << 8) | *ptr.offset(39) as u32;
                if width > 0 && height > 0 && width < 32768 && height < 32768 {
                    return (width, height, data_vec);
                }
            }
        }
    }

    // Try JPEG
    let jpeg_type = NSString::from_str("public.jpeg");
    if let Some(data) = pasteboard.dataForType(&jpeg_type) {
        let bytes = data.bytes();
        let data_vec = bytes.to_vec();

        if bytes.len() > 2 {
            let ptr = bytes.as_ptr() as *const u8;
            let len = bytes.len();
            for i in 0..len.saturating_sub(8) {
                if *ptr.offset(i as isize) == 0xFF {
                    let marker = *ptr.offset(i as isize + 1);
                    if marker >= 0xC0 && marker <= 0xCF && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
                        let height = (*ptr.offset(i as isize + 5) as u32) << 8 | (*ptr.offset(i as isize + 6) as u32);
                        let width = (*ptr.offset(i as isize + 7) as u32) << 8 | (*ptr.offset(i as isize + 8) as u32);
                        if width > 0 && height > 0 && width < 32768 && height < 32768 {
                            return (width, height, data_vec);
                        }
                    }
                }
            }
        }
    }

    // Try BMP
    let bmp_type = NSString::from_str("com.microsoft.bmp");
    if let Some(data) = pasteboard.dataForType(&bmp_type) {
        let bytes = data.bytes();
        let data_vec = bytes.to_vec();

        if data_vec.len() >= 26 {
            let ptr = bytes.as_ptr() as *const u8;
            let width = (*ptr.offset(18) as u32) | ((*ptr.offset(19) as u32) << 8) |
                       ((*ptr.offset(20) as u32) << 16) | ((*ptr.offset(21) as u32) << 24);
            let height = (*ptr.offset(22) as u32) | ((*ptr.offset(23) as u32) << 8) |
                        ((*ptr.offset(24) as u32) << 16) | ((*ptr.offset(25) as u32) << 24);
            if width > 0 && height > 0 && width < 32768 && height < 32768 {
                return (width, height, data_vec);
            }
        }
    }

    (0, 0, vec![])
}
