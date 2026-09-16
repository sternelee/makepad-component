//! The chrome bars: a titlebar, a control bar and a menubar.
//!
//! Shown stacked inside one window mock, because that is the only way a bar can be
//! checked: a strip's correctness is *alignment* — that the title clears the window
//! buttons, that the control bar's edges line up with the titlebar's, that the
//! menubar's triggers sit on one baseline. A bar rendered alone on a page would
//! show that its fill drew and nothing about whether it works.
//!
//! Two of the three have no height of their own and take `Fit`; only the titlebar
//! reads a number, and it reads the theme's, which this port already carried from
//! the reference. That is the decision worth reading here: a control bar is 32pt in
//! one app and 44pt in another, so a library that picks one is a library that will
//! be overridden.

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
    let Trigger = mod.mp.MpButtonSmall{
        style: mod.mp.ButtonStyle.Ghost
        text: "File"
    }

    mod.gallery.pages.bars = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Bars"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Three horizontal strips that hold other things. A strip's only real decisions are how tall it is, how much air it has at the sides, and where its content sits vertically — so they are three prototypes in one module rather than three modules. Only the titlebar reads a height, because a titlebar is one of the few pieces of chrome with a platform-defined shape and this port's layout tokens already carried titlebar_height, titlebar_top_pad and traffic_light_inset from the reference. The other two take Fit, because a control bar is 32pt in one app and 44pt in another, and a library that picks one is a library that will be overridden."
        }

        Section{
            Caption{ text: "Stacked, because a bar's correctness is alignment — the title clears the window buttons, the control bar's edges line up with the titlebar's, the menubar's triggers sit on one baseline. The leading gap in the titlebar is traffic_light_inset, a spacer rather than padding so the bar's own background runs under the window buttons" }
            View{
                width: 620
                height: Fit
                flow: Down
                spacing: 0

                mod.mp.MpTitlebar{
                    titlebar_title := Label{
                        text: "agent-workbench — terminal.rs"
                    }
                    titlebar_actions := mod.mp.Row{
                        // `text: ""` is load-bearing: `MpButton` ships a
                        // placeholder label, so setting only `glyph` gives an
                        // icon *and* the word "Button". Found by reading the
                        // rendered window, not the log — the two buttons in the
                        // titlebar said "Button" and nothing errored.
                        mod.mp.MpButtonSmall{style: mod.mp.ButtonStyle.Ghost, text: "", glyph: "\u{f067}"}
                        mod.mp.MpButtonSmall{style: mod.mp.ButtonStyle.Ghost, text: "", glyph: "\u{f00c}"}
                    }
                }
                mod.mp.MpControlBar{
                    controlbar_leading := mod.mp.Row{
                        Trigger{ text: "File" }
                        Trigger{ text: "Edit" }
                        Trigger{ text: "View" }
                        Trigger{ text: "Run" }
                    }
                }
                mod.mp.MpControlBar{
                    draw_bg +: {color: surface}
                    controlbar_leading := mod.mp.Row{
                        // This was three ghost buttons with the active one promoted, because the library had no
                        // segmented control and the page said so. `mp/segmented.rs` is that interim replaced.
                        view_mode := mod.mp.MpSegmentedSmall{}
                    }
                    controlbar_trailing := mod.mp.Row{
                        mod.mp.MpTextInputSearch{
                            width: 200
                            empty_text: "Filter files"
                        }
                    }
                }
                mod.mp.SurfaceSunken{
                    width: Fill
                    height: 150
                    draw_bg +: {border_radius: instance(0.0)}
                    align: Align{x: 0.5, y: 0.5}
                    Label{
                        width: Fit, height: Fit
                        draw_text +: {text_style: footnote, color: text_faint}
                        text: "window body"
                    }
                }
            }
        }

        Section{
            Caption{ text: "A menubar on its own — File Edit View Window Help. Its triggers are MpButton ghosts rather than labels, so they get the press, focus and hover behaviour every other control already has instead of a second implementation of the same three states" }
            mod.mp.MpMenubar{
                width: 420
                menubar_items := mod.mp.Row{
                    Trigger{ text: "File" }
                    Trigger{ text: "Edit" }
                    Trigger{ text: "View" }
                    Trigger{ text: "Window" }
                    Trigger{ text: "Help" }
                }
            }
        }
    }
}
