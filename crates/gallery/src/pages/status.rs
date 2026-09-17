//! Badges and tags: the six tones, and the difference between reporting and
//! classifying.
//!
//! The page exists to check two things a test cannot: that a tag is visually
//! quieter than a badge at the same tone (a tag that shouts reads as a button),
//! and that a status badge's label is legible on its own pale plate.

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
        width: Fill, height: Fit, flow: Down, spacing: 8
    }

    let Row = View{
        width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
    }

    mod.gallery.pages.status = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Status"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A badge reports something about its subject — a count, a state, a build number — so it carries its own fill and reads as an object placed on the row. A tag classifies its subject — a category, a filter that is on — so it is a wash with a hairline and does not compete with the text beside it. Same shader, one closed set of six tones, two assembled looks."
        }

        Section{
            Caption{ text: "The six tones, as badges" }
            Row{
                mod.mp.MpBadge{ tone: mod.mp.StatusTone.Neutral, text: "Neutral" }
                mod.mp.MpBadge{ tone: mod.mp.StatusTone.Solid, text: "Solid" }
                mod.mp.MpBadge{ tone: mod.mp.StatusTone.Accent, text: "Accent" }
                mod.mp.MpBadge{ tone: mod.mp.StatusTone.Success, text: "Success" }
                mod.mp.MpBadge{ tone: mod.mp.StatusTone.Warning, text: "Warning" }
                mod.mp.MpBadge{ tone: mod.mp.StatusTone.Danger, text: "Danger" }
                mod.mp.MpBadge{ tone: mod.mp.StatusTone.Busy, text: "Busy" }
            }
        }

        Section{
            Caption{ text: "The same six as tags — a tag with a dot is a state, one without is a category" }
            Row{
                mod.mp.MpTag{ tone: mod.mp.StatusTone.Neutral, text: "Neutral", dot: true }
                mod.mp.MpTag{ tone: mod.mp.StatusTone.Solid, text: "Solid", dot: true }
                mod.mp.MpTag{ tone: mod.mp.StatusTone.Accent, text: "Accent", dot: true }
                mod.mp.MpTag{ tone: mod.mp.StatusTone.Success, text: "Passing", dot: true }
                mod.mp.MpTag{ tone: mod.mp.StatusTone.Warning, text: "Flaky", dot: true }
                mod.mp.MpTag{ tone: mod.mp.StatusTone.Danger, text: "Failed", dot: true }
                mod.mp.MpTag{ tone: mod.mp.StatusTone.Busy, text: "Queued", dot: true }
            }
            Row{
                mod.mp.MpTagNoDot{ tone: mod.mp.StatusTone.Neutral, text: "rust" }
                mod.mp.MpTagNoDot{ tone: mod.mp.StatusTone.Neutral, text: "makepad" }
                mod.mp.MpTagNoDot{ tone: mod.mp.StatusTone.Neutral, text: "gpui" }
                mod.mp.MpTagSmall{ tone: mod.mp.StatusTone.Accent, text: "small" }
            }
        }

        Section{
            Caption{ text: "In a row of content — which is where the difference matters. A badge sits on the line; a tag belongs to it" }
            View{
                width: Fill, height: Fit, flow: Down, spacing: 10
                Row{
                    Label{ width: 160, height: Fit, draw_text +: {text_style: body, color: text} text: "Build 4120" }
                    mod.mp.MpBadge{ tone: mod.mp.StatusTone.Success, text: "passed" }
                    mod.mp.MpBadgeSmall{ tone: mod.mp.StatusTone.Neutral, text: "2m 14s" }
                }
                Row{
                    Label{ width: 160, height: Fit, draw_text +: {text_style: body, color: text} text: "agent-workbench" }
                    mod.mp.MpTagNoDot{ tone: mod.mp.StatusTone.Neutral, text: "terminal" }
                    mod.mp.MpTagNoDot{ tone: mod.mp.StatusTone.Neutral, text: "cef" }
                    mod.mp.MpBadge{ tone: mod.mp.StatusTone.Busy, text: "3 running" }
                }
                Row{
                    Label{ width: 160, height: Fit, draw_text +: {text_style: body, color: text} text: "nightly-sync" }
                    mod.mp.MpTagNoDot{ tone: mod.mp.StatusTone.Neutral, text: "cron" }
                    mod.mp.MpBadge{ tone: mod.mp.StatusTone.Danger, text: "failed" }
                }
            }
        }

        Section{
            Caption{ text: "Counts and dots — a badge with no text is a dot, which is how a bare count becomes a mark" }
            Row{
                mod.mp.MpBadge{ tone: mod.mp.StatusTone.Accent, text: "12" }
                mod.mp.MpBadge{ tone: mod.mp.StatusTone.Danger, text: "99+" }
                mod.mp.MpBadgeDot{ tone: mod.mp.StatusTone.Danger }
                mod.mp.MpBadgeDot{ tone: mod.mp.StatusTone.Success }
                mod.mp.MpBadgeDot{ tone: mod.mp.StatusTone.Neutral }
            }
        }

        Section{
            Caption{ text: "Small — the chip rung, for a dense list" }
            Row{
                mod.mp.MpBadgeSmall{ tone: mod.mp.StatusTone.Neutral, text: "1.4.2" }
                mod.mp.MpBadgeSmall{ tone: mod.mp.StatusTone.Success, text: "ok" }
                mod.mp.MpTagSmall{ tone: mod.mp.StatusTone.Warning, text: "deprecated", dot: true }
                mod.mp.MpTagSmall{ tone: mod.mp.StatusTone.Danger, text: "removed", dot: true }
            }
        }
    }
}
