use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpMessage - chat message row with avatar/name header, body and
    // footer meta slots (gpui Message port). End alignment right-aligns
    // the rows for outgoing messages.
    // ============================================================

    mod.widgets.MpMessageAlignment = #(MpMessageAlignment::script_api(vm))

    mod.widgets.MpMessageBase = #(MpMessage::register_widget(vm))
    mod.widgets.MpMessage = set_type_default() do mod.widgets.MpMessageBase{
        width: Fill
        height: Fit
        flow: Down
        spacing: 4.0

        header := View{
            width: Fill, height: Fit
            flow: Right
            spacing: 8.0
            align: Align{y: 0.5}

            avatar := View{
                width: 28, height: 28
                align: Align{x: 0.5, y: 0.5}
                show_bg: true
                draw_bg +: {
                    color: ACCENT
                    border_radius: 14.0
                }

                avatar_label := Label{
                    width: Fit, height: Fit
                    draw_text +: {
                        text_style: theme.font_bold{font_size: 12.0}
                        color: ON_ACCENT
                    }
                    text: ""
                }
            }

            name := Label{
                width: Fit, height: Fit
                draw_text +: {
                    text_style: theme.font_bold{font_size: 13.0}
                    color: TEXT
                }
                text: ""
            }
        }

        body := View{
            width: Fill, height: Fit
            flow: Down

            body_label := Label{
                width: Fill, height: Fit
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: TEXT
                }
                text: ""
            }
        }

        footer := View{
            width: Fill, height: Fit
            flow: Right

            footer_label := Label{
                width: Fit, height: Fit
                draw_text +: {
                    text_style: theme.font_regular{font_size: 11.0}
                    color: TEXT_MUTED
                }
                text: ""
            }
        }
    }

    // Outgoing messages: rows right-aligned.
    mod.widgets.MpMessageEnd = mod.widgets.MpMessage{
        alignment: mod.widgets.MpMessageAlignment.End
    }
}

/// Message actions
#[derive(Clone, Debug, Default)]
pub enum MpMessageAction {
    #[default]
    None,
}

/// Leading or trailing alignment of the message rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Script, ScriptHook)]
pub enum MpMessageAlignment {
    /// Incoming messages (left-aligned).
    #[default]
    Start,
    /// Outgoing messages (right-aligned).
    End,
}

/// Chat message row: avatar + name header, body text and footer meta.
#[derive(Script, ScriptHook, Widget)]
pub struct MpMessage {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Leading (incoming) or trailing (outgoing) alignment.
    #[live]
    alignment: MpMessageAlignment,

    /// Hide the header row entirely (continuation messages in a group).
    #[live(true)]
    show_header: bool,

    /// Hide the footer row entirely.
    #[live]
    show_footer: bool,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[rust]
    applied_alignment: Option<MpMessageAlignment>,
    #[rust]
    applied_header: Option<bool>,
    #[rust]
    applied_footer: Option<bool>,
}

impl Widget for MpMessage {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Apply alignment / row visibility once per change (children request
        // redraws on their own setters, so guard here).
        if self.applied_alignment != Some(self.alignment) {
            self.applied_alignment = Some(self.alignment);
            self.layout.align.x = match self.alignment {
                MpMessageAlignment::Start => 0.0,
                MpMessageAlignment::End => 1.0,
            };
            self.redraw(cx);
        }
        if self.applied_header != Some(self.show_header) {
            self.applied_header = Some(self.show_header);
            self.view.widget(cx, ids!(header)).set_visible(cx, self.show_header);
        }
        if self.applied_footer != Some(self.show_footer) {
            self.applied_footer = Some(self.show_footer);
            self.view.widget(cx, ids!(footer)).set_visible(cx, self.show_footer);
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpMessage {
    fn initials_of(name: &str) -> String {
        let mut out = String::new();
        for word in name.split_whitespace() {
            if let Some(c) = word.chars().next() {
                out.push(c);
            }
            if out.chars().count() >= 2 {
                break;
            }
        }
        if out.is_empty() {
            if let Some(c) = name.chars().next() {
                out.push(c);
            }
        }
        out.to_uppercase()
    }

    /// Set the sender name; the avatar shows up to two leading initials.
    pub fn set_name(&mut self, cx: &mut Cx, name: &str) {
        self.view.label(cx, ids!(name)).set_text(cx, name);
        self.view
            .label(cx, ids!(avatar_label))
            .set_text(cx, &Self::initials_of(name));
    }

    /// Set the message body text.
    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.view.label(cx, ids!(body_label)).set_text(cx, text);
    }

    /// Set the footer meta text (timestamp, status, ...). Empty hides the row.
    pub fn set_footer(&mut self, cx: &mut Cx, footer: &str) {
        self.view
            .label(cx, ids!(footer_label))
            .set_text(cx, footer);
        self.show_footer = !footer.is_empty();
        self.applied_footer = None;
        self.redraw(cx);
    }

    pub fn set_alignment(&mut self, cx: &mut Cx, alignment: MpMessageAlignment) {
        if self.alignment != alignment {
            self.alignment = alignment;
            self.applied_alignment = None;
            self.redraw(cx);
        }
    }
}

impl MpMessageRef {
    pub fn set_name(&self, cx: &mut Cx, name: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_name(cx, name);
        }
    }

    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    pub fn set_footer(&self, cx: &mut Cx, footer: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_footer(cx, footer);
        }
    }

    pub fn set_alignment(&self, cx: &mut Cx, alignment: MpMessageAlignment) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_alignment(cx, alignment);
        }
    }
}
