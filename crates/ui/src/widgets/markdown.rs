//! `MpMarkdown` — a theme-aligned markdown renderer.
//!
//! Reuses makepad-widgets' built-in `Markdown` widget (pulldown-cmark parser +
//! `TextFlow` layout), which already handles headings, bold/italic/strikethrough,
//! ordered/unordered lists, fenced code blocks, inline code, blockquotes, tables,
//! horizontal rules, and links. This module only re-skins it onto the
//! `mpc_theme` design tokens so it follows appearance switching instead of the
//! baked-in makepad theme colors.
//!
//! It is a pure `script_mod!` variant (no Rust struct), so it is addressed
//! through the underlying `Markdown` widget ref:
//!
//! ```ignore
//! self.ui.markdown(cx, ids!(md)).set_text(cx, "# Title\n\n- a\n- b");
//! ```

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Body metrics — bezel's TEXT_SIZE 14 / LINE_HEIGHT 22 (≈1.57 line-height).
    let MD_FONT_SIZE = 14.0
    let MD_LINE_HEIGHT = 1.6
    let MD_HEADING_SCALE = 1.8
    let MD_PARAGRAPH_SPACING = 14.0
    let MD_PRE_CODE_SPACING = 10.0
    let MD_FIXED_FONT_SIZE = 13.0

    mod.widgets.MpMarkdown = mod.widgets.Markdown{
        width: Fill
        height: Fit
        flow: Flow.Right{wrap: true}

        font_color: TEXT
        font_size: MD_FONT_SIZE
        paragraph_spacing: MD_PARAGRAPH_SPACING
        pre_code_spacing: MD_PRE_CODE_SPACING
        heading_base_scale: MD_HEADING_SCALE

        draw_text +: {
            color: TEXT
        }

        text_style_normal: theme.font_regular{font_size: MD_FONT_SIZE}
        text_style_italic: theme.font_italic{font_size: MD_FONT_SIZE}
        text_style_bold: theme.font_bold{font_size: MD_FONT_SIZE}
        text_style_bold_italic: theme.font_bold_italic{font_size: MD_FONT_SIZE}
        text_style_fixed: theme.font_code{font_size: MD_FIXED_FONT_SIZE}

        // Block chrome: quote border/background, horizontal rule, inline-code
        // wash, text selection, table header fill and row borders.
        draw_block +: {
            line_color: TEXT_MUTED
            sep_color: DIVIDER
            quote_bg_color: CODE_WASH
            quote_fg_color: TEXT_MUTED
            code_color: CODE_WASH
            selection_color: SELECTION
            table_header_bg_color: SURFACE
            table_border_color: DIVIDER
        }

        // Links: accent text. (The link underline shader keeps makepad's own
        // theme colors; overriding its pixel fn would risk a shader-compile
        // mismatch, so only the resting text color is re-tokenized here.)
        link +: {
            draw_text +: {
                color: ACCENT
            }
        }
    }
}
