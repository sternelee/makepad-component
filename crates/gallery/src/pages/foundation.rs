//! The foundation pages: the palette, the type ladder, the layout metrics.
//!
//! These are the three things every other page is drawn with, so they come
//! first — a reader who wants to know what `surface_card` is looks at a
//! swatch, not at the source of the theme crate.


use makepad_widgets::*;
script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    // ---- shared furniture ----

    let Page = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20
    }

    let Section = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 8
    }

    let Heading = Label{
        width: Fit
        height: Fit
        draw_text +: {
            text_style: title2
            color: text
        }
    }

    let Note = Label{
        width: Fill
        height: Fit
        draw_text +: {
            text_style: footnote
            color: text_muted
        }
    }

    // A swatch: the colour, its token name, and the value it resolved to.
    //
    // The rounded plate is `SurfaceSunken` rather than a bare rect, because a
    // swatch of `bg` on a `bg` page would otherwise be invisible — the plate
    // gives every swatch the same neutral ground so the colour under test is
    // the only thing being read.
    let Swatch = mod.mp.SurfaceSunken{
        width: 116
        height: 60
        padding: Inset{left: 8, right: 8, top: 6, bottom: 6}
        spacing: 4

        plate := mod.mp.Surface{
            width: Fill
            height: Fill
            show_bg: true
            draw_bg +: {
                color: instance(text)
                border_size: instance(0.0)
                border_radius: instance(4.0)
            }
        }

        name := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: caption
                color: text_muted
            }
            text: "token"
        }
    }

    let Row = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 8
        align: Align{y: 0.5}
    }

    // ---- Palette ----
    //
    // Six groups, because the token list is not flat in use: the surface ladder
    // is read together (a card sits on a shell on a page), the ink ladder is
    // read together, and the plates are read against their own labels.
    mod.gallery.pages.palette = Page{
        Heading{ text: "Palette" }
        Note{
            text: "Every token in the appearance in force. Switch appearance with the button in the rail and every swatch moves — the tokens are generated from the theme in Rust, not written per appearance."
        }

        Section{
            Heading{ text: "Surface ladder" }
            Row{
                Swatch{ name +: {text: "bg"} plate +: {draw_bg +: {color: instance(bg)}} }
                Swatch{ name +: {text: "surface"} plate +: {draw_bg +: {color: instance(surface)}} }
                Swatch{ name +: {text: "surface_raised"} plate +: {draw_bg +: {color: instance(surface_raised)}} }
                Swatch{ name +: {text: "surface_card"} plate +: {draw_bg +: {color: instance(surface_card)}} }
            }
            Row{
                Swatch{ name +: {text: "surface_dialog"} plate +: {draw_bg +: {color: instance(surface_dialog)}} }
                Swatch{ name +: {text: "surface_overlay"} plate +: {draw_bg +: {color: instance(surface_overlay)}} }
                Swatch{ name +: {text: "element_hover"} plate +: {draw_bg +: {color: instance(element_hover)}} }
                Swatch{ name +: {text: "element_active"} plate +: {draw_bg +: {color: instance(element_active)}} }
            }
        }

        Section{
            Heading{ text: "Ink ladder" }
            Row{
                Swatch{ name +: {text: "text"} plate +: {draw_bg +: {color: instance(text)}} }
                Swatch{ name +: {text: "text_muted"} plate +: {draw_bg +: {color: instance(text_muted)}} }
                Swatch{ name +: {text: "text_faint"} plate +: {draw_bg +: {color: instance(text_faint)}} }
                Swatch{ name +: {text: "text_dim"} plate +: {draw_bg +: {color: instance(text_dim)}} }
            }
        }

        Section{
            Heading{ text: "Hairlines" }
            Row{
                Swatch{ name +: {text: "border"} plate +: {draw_bg +: {color: instance(border)}} }
                Swatch{ name +: {text: "border_strong"} plate +: {draw_bg +: {color: instance(border_strong)}} }
                Swatch{ name +: {text: "divider"} plate +: {draw_bg +: {color: instance(divider)}} }
                Swatch{ name +: {text: "ring"} plate +: {draw_bg +: {color: instance(ring)}} }
            }
        }

        Section{
            Heading{ text: "Plates" }
            Row{
                Swatch{ name +: {text: "solid"} plate +: {draw_bg +: {color: instance(solid)}} }
                Swatch{ name +: {text: "accent"} plate +: {draw_bg +: {color: instance(accent)}} }
                Swatch{ name +: {text: "accent_strong"} plate +: {draw_bg +: {color: instance(accent_strong)}} }
                Swatch{ name +: {text: "caret"} plate +: {draw_bg +: {color: instance(caret)}} }
            }
        }

        Section{
            Heading{ text: "Status" }
            Row{
                Swatch{ name +: {text: "danger"} plate +: {draw_bg +: {color: instance(danger)}} }
                Swatch{ name +: {text: "warning"} plate +: {draw_bg +: {color: instance(warning)}} }
                Swatch{ name +: {text: "success"} plate +: {draw_bg +: {color: instance(success)}} }
                Swatch{ name +: {text: "busy"} plate +: {draw_bg +: {color: instance(busy)}} }
            }
        }
    }

    // ---- Type ----
    //
    // The ladder is shown at its measured size on its measured line box, so a
    // reader can see the two things that are easy to get wrong: the leading is
    // not one ratio, and `Headline` and `Body` share a size and differ only in
    // weight.
    mod.gallery.pages.typography = Page{
        Heading{ text: "Type" }
        Note{
            text: "Eleven roles measured from NSFont.preferredFont(forTextStyle:), each with its own line height. Leading runs 1.18 at Title to 1.33 at Title3 and does not move with the size, which is why it is a table and not a multiplier."
        }

        Section{
            large_title_row := Row{ Label{ draw_text +: {text_style: large_title, color: text} text: "LargeTitle 26/32"} }
            title_row := Row{ Label{ draw_text +: {text_style: title, color: text} text: "Title 22/26"} }
            title2_row := Row{ Label{ draw_text +: {text_style: title2, color: text} text: "Title2 17/22"} }
            title3_row := Row{ Label{ draw_text +: {text_style: title3, color: text} text: "Title3 15/20 — the ratio here is 1.33, the loosest rung"} }
            headline_row := Row{ Label{ draw_text +: {text_style: headline, color: text} text: "Headline 13/16, bold — same size as Body"} }
            subheadline_row := Row{ Label{ draw_text +: {text_style: subheadline, color: text} text: "Subheadline 11/14"} }
            body_row := Row{ Label{ draw_text +: {text_style: body, color: text} text: "Body 13/16 — the size every other role is a ratio of"} }
            callout_row := Row{ Label{ draw_text +: {text_style: callout, color: text} text: "Callout 12/15 — the small control's role"} }
            footnote_row := Row{ Label{ draw_text +: {text_style: footnote, color: text} text: "Footnote 10/13"} }
            caption_row := Row{ Label{ draw_text +: {text_style: caption, color: text} text: "Caption 10/13"} }
            caption2_row := Row{ Label{ draw_text +: {text_style: caption2, color: text} text: "Caption2 10/13, medium"} }
        }
    }

    // ---- Metrics ----
    //
    // Each control is drawn at its real height with its real padding, so the
    // page is the measurement rather than a description of it.
    mod.gallery.pages.metrics = Page{
        Heading{ text: "Metrics" }
        Note{
            text: "The numbers the platform named, plus the ratios derived from them. Heights are measured from NSButton, NSTextField and NSPopUpButton: 20 at .small and 24 at .regular, which is Body's 16pt line box with 4 above and below."
        }

        Section{
            Heading{ text: "Control sizes" }
            control_small := View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                mod.mp.MpButtonSmall{ text: "Small — 20pt" }
                Label{ draw_text +: {text_style: footnote, color: text_muted} text: "Callout 12pt on the control radius" }
            }
            control_regular := View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                mod.mp.MpButton{ text: "Regular — 24pt" }
                Label{ draw_text +: {text_style: footnote, color: text_muted} text: "Body 13pt on the button radius" }
            }
            control_large := View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                mod.mp.MpButtonLarge{ text: "Large — 28pt" }
                Label{ draw_text +: {text_style: footnote, color: text_muted} text: "Title3 15pt — derived, not measured" }
            }
        }

        Section{
            Heading{ text: "Radius" }
            Note{
                text: "Every corner is a ratio of one base (8pt). Branding the radius moves all five together, which is why a component never writes a corner of its own."
            }
            radius_row := View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                mod.mp.SurfaceRaised{ width: 64, height: 44, draw_bg +: {border_radius: instance(6.0)}, Label{ text: "control" } }
                mod.mp.SurfaceRaised{ width: 64, height: 44, draw_bg +: {border_radius: instance(8.0)}, Label{ text: "button" } }
                mod.mp.SurfaceRaised{ width: 64, height: 44, draw_bg +: {border_radius: instance(10.0)}, Label{ text: "panel" } }
                mod.mp.SurfaceRaised{ width: 64, height: 44, draw_bg +: {border_radius: instance(12.0)}, Label{ text: "surface" } }
                mod.mp.SurfaceRaised{ width: 64, height: 44, draw_bg +: {border_radius: instance(16.0)}, Label{ text: "bubble" } }
            }
        }

        Section{
            Heading{ text: "The sibling gap" }
            Note{
                text: "8pt: NSStackView().spacing, the visual format's '-', and constraint(equalToSystemSpacingAfter:) all report it. ui::stack's row and column carry it, so a call site that wants the standard gap writes no number."
            }
            gap_row := View{
                width: Fill, height: Fit, flow: Right, spacing: 8
                mod.mp.SurfaceRaised{ width: 60, height: 32 }
                mod.mp.SurfaceRaised{ width: 60, height: 32 }
                mod.mp.SurfaceRaised{ width: 60, height: 32 }
            }
        }
    }
}
