//! Icons: the glyph face, the ladder, and ink that follows its row.
//!
//! The page's own check is that a glyph set on the control ladder sits on the
//! optical centre of the control beside it — which is the thing a call site
//! writing `Label{text_style: theme.font_icons}` gets wrong.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    let Caption = Label{
        width: Fit, height: Fit
        draw_text +: {text_style: caption, color: text_muted}
    }

    let Section = View{
        width: Fill, height: Fit, flow: Down, spacing: 6
    }

    let Row = View{
        width: Fill, height: Fit, flow: Right, spacing: 10, align: Align{y: 0.5}
    }

    mod.gallery.pages.icon = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Icon"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A glyph from Makepad's bundled FontAwesome, sized from the same three-rung ladder every control uses, in `text_muted` because an icon is decoration. The alternatives a call site writes by hand — a raw Label with a font override — sit low in their row, because the icon face has no descender to take up the line box's slack."
        }

        Section{
            Caption{ text: "The ladder — an icon inside a control is the right size without anyone comparing them" }
            Row{
                mod.mp.MpIconSmall{ glyph: "\u{f111}" }
                mod.mp.MpIcon{ glyph: "\u{f111}" }
                mod.mp.MpIconLarge{ glyph: "\u{f111}" }
            }
        }

        Section{
            Caption{ text: "Beside text at each rung — the glyph and the label share a baseline, which is the whole job" }
            Row{
                mod.mp.MpIconSmall{ glyph: "\u{f00c}" }
                Label{ draw_text +: {text_style: callout, color: text} text: "Small" }
                mod.mp.MpIcon{ glyph: "\u{f00c}" }
                Label{ draw_text +: {text_style: body, color: text} text: "Regular" }
                mod.mp.MpIconLarge{ glyph: "\u{f00c}" }
                Label{ draw_text +: {text_style: title3, color: text} text: "Large" }
            }
        }

        Section{
            Caption{ text: "In a button — a `glyph` property rather than a child `MpIcon`, because a button is not a container: a child is never drawn and never errors. This page shipped for a while with icons that rendered as plain buttons and logged nothing. Leading by default; `glyph_trailing: true` puts it after the label, which is what a select's chevron is" }
            Row{
                mod.mp.MpButton{ glyph: "\u{f067}", text: "New" }
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Prominent, glyph: "\u{f0c7}", text: "Save" }
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Ghost, glyph: "\u{f00d}", text: "Close" }
                mod.mp.MpButton{ glyph: "\u{f0d7}", glyph_trailing: true, text: "Options" }
            }
        }

        Section{
            Caption{ text: "A toolbar — one rung, eight glyphs, and they line up because they all take their size from the same place" }
            Row{
                mod.mp.MpIcon{ glyph: "\u{f0c9}" }
                mod.mp.MpIcon{ glyph: "\u{f002}" }
                mod.mp.MpIcon{ glyph: "\u{f013}" }
                mod.mp.MpIcon{ glyph: "\u{f0f3}" }
                mod.mp.MpIcon{ glyph: "\u{f1de}" }
                mod.mp.MpIcon{ glyph: "\u{f1f8}" }
                mod.mp.MpIcon{ glyph: "\u{f023}" }
                mod.mp.MpIcon{ glyph: "\u{f0e0}" }
                mod.mp.MpIcon{ glyph: "\u{f2f5}" }
                mod.mp.MpIcon{ glyph: "\u{f0a9}" }
            }
        }

        Section{
            Caption{ text: "A row of list glyphs — same components, different characters. A name-to-glyph map is the caller's, because the font has 1834 and a map here would be a second place to keep it wrong" }
            Row{
                mod.mp.MpIcon{ glyph: "\u{f07b}" }
                Label{ draw_text +: {text_style: body, color: text} text: "Documents" }
                mod.mp.MpIcon{ glyph: "\u{f0d6}" }
                Label{ draw_text +: {text_style: body, color: text} text: "Receipts" }
                mod.mp.MpIcon{ glyph: "\u{f1c0}" }
                Label{ draw_text +: {text_style: body, color: text} text: "Archive" }
                mod.mp.MpIcon{ glyph: "\u{f1f8}" }
                Label{ draw_text +: {text_style: body, color: text} text: "Trash" }
            }
        }
    }
}
