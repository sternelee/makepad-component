// Render methods for shadcn UI components exposed via the Splash A2UI script layer.
//
// Components: Badge, Alert, Avatar, Progress, Spinner, Switch, Accordion,
// Notification, Skeleton.

// ============================================================================
// Color helpers
// ============================================================================

fn badge_color(variant: Option<BadgeVariant>) -> Vec4 {
    match variant.unwrap_or_default() {
        BadgeVariant::Default     => vec4(0.231, 0.510, 0.965, 1.0), // #3B82F6 blue
        BadgeVariant::Secondary   => vec4(0.420, 0.447, 0.502, 1.0), // #6B7280 gray
        BadgeVariant::Destructive => vec4(0.937, 0.267, 0.267, 1.0), // #EF4444 red
        BadgeVariant::Outline     => vec4(0.150, 0.230, 0.360, 1.0), // dark border bg
        BadgeVariant::Success     => vec4(0.133, 0.773, 0.369, 1.0), // #22C55E green
        BadgeVariant::Warning     => vec4(0.969, 0.620, 0.067, 1.0), // #F59E0B amber
        BadgeVariant::Info        => vec4(0.231, 0.510, 0.965, 1.0), // same as default
    }
}

fn alert_bg_color(variant: Option<AlertVariant>) -> Vec4 {
    match variant.unwrap_or_default() {
        AlertVariant::Default     => vec4(0.110, 0.168, 0.259, 1.0), // #1C2B42
        AlertVariant::Destructive => vec4(0.290, 0.098, 0.098, 1.0), // #4A1919
        AlertVariant::Warning     => vec4(0.290, 0.220, 0.050, 1.0), // #4A380D
        AlertVariant::Success     => vec4(0.075, 0.259, 0.133, 1.0), // #132240
        AlertVariant::Info        => vec4(0.075, 0.157, 0.290, 1.0), // #132649
    }
}

fn alert_accent_color(variant: Option<AlertVariant>) -> Vec4 {
    match variant.unwrap_or_default() {
        AlertVariant::Default     => vec4(0.231, 0.510, 0.965, 1.0),
        AlertVariant::Destructive => vec4(0.937, 0.267, 0.267, 1.0),
        AlertVariant::Warning     => vec4(0.969, 0.620, 0.067, 1.0),
        AlertVariant::Success     => vec4(0.133, 0.773, 0.369, 1.0),
        AlertVariant::Info        => vec4(0.231, 0.510, 0.965, 1.0),
    }
}

fn avatar_color(idx: usize) -> Vec4 {
    const COLORS: &[(f32, f32, f32)] = &[
        (0.388, 0.400, 0.945), // indigo  #6366F1
        (0.220, 0.686, 0.506), // teal    #38AF81
        (0.941, 0.204, 0.545), // pink    #F0348B
        (0.969, 0.620, 0.067), // amber   #F59E0B
        (0.133, 0.773, 0.369), // green   #22C55E
        (0.580, 0.247, 0.902), // purple  #943CE6
    ];
    let (r, g, b) = COLORS[idx % COLORS.len()];
    Vec4 { x: r, y: g, z: b, w: 1.0 }
}

// ============================================================================
// Badge, Alert, Avatar, Progress, Spinner, Switch, Accordion, Notification, Skeleton
// ============================================================================

impl A2uiSurface {
    // -------------------------------------------------------------------------
    // Badge
    // -------------------------------------------------------------------------

    pub(super) fn render_badge(
        &mut self,
        cx: &mut Cx2d,
        badge: &BadgeComponent,
        data_model: &DataModel,
    ) {
        let text = resolve_string_value_scoped(
            &badge.text,
            data_model,
            self.current_scope.as_deref(),
        );
        let bg_color = badge_color(badge.variant);

        let walk = Walk {
            width: Size::fit(),
            height: Size::Fixed(22.0),
            margin: Margin { left: 0.0, right: 4.0, top: 2.0, bottom: 2.0 },
            ..Walk::default()
        };
        let layout = Layout {
            flow: Flow::right(),
            align: Align { x: 0.5, y: 0.5 },
            padding: Padding { left: 8.0, right: 8.0, top: 3.0, bottom: 3.0 },
            ..Layout::default()
        };

        self.draw_badge.apply_over(cx, live! { color: (bg_color) border_radius: 11.0 });
        self.draw_badge.begin(cx, walk, layout);
        self.draw_comp_text.apply_over(cx, live! {
            text_style: { font_size: 11.0 }
            color: #FFFFFF
        });
        self.draw_comp_text.draw_walk(cx, Walk::fit(), Align::default(), &text);
        self.draw_badge.end(cx);
    }

    // -------------------------------------------------------------------------
    // Alert
    // -------------------------------------------------------------------------

    pub(super) fn render_alert(
        &mut self,
        cx: &mut Cx2d,
        _scope: &mut Scope,
        _surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        alert: &AlertComponent,
    ) {
        let description = resolve_string_value_scoped(
            &alert.description,
            data_model,
            self.current_scope.as_deref(),
        );
        let title = alert.title.as_ref().map(|t| {
            resolve_string_value_scoped(t, data_model, self.current_scope.as_deref())
        });

        let bg_color = alert_bg_color(alert.variant);
        let accent = alert_accent_color(alert.variant);

        let walk = Walk {
            width: Size::fill(),
            height: Size::fit(),
            margin: Margin { left: 0.0, right: 0.0, top: 6.0, bottom: 6.0 },
            ..Walk::default()
        };
        let layout = Layout {
            flow: Flow::Down,
            padding: Padding { left: 12.0, right: 12.0, top: 10.0, bottom: 10.0 },
            spacing: 4.0,
            ..Layout::default()
        };

        self.draw_badge.apply_over(cx, live! { color: (bg_color) border_radius: 8.0 });
        self.draw_badge.begin(cx, walk, layout);

        if let Some(ref title_text) = title {
            // Row: [accent dot] [title]
            let row_walk = Walk {
                width: Size::fill(),
                height: Size::fit(),
                ..Walk::default()
            };
            let row_layout = Layout {
                flow: Flow::right(),
                align: Align { x: 0.0, y: 0.5 },
                spacing: 6.0,
                ..Layout::default()
            };
            cx.begin_turtle(row_walk, row_layout);

            // Accent dot
            let dot_walk = Walk {
                width: Size::Fixed(6.0),
                height: Size::Fixed(6.0),
                margin: Margin { top: 0.0, bottom: 0.0, left: 0.0, right: 0.0 },
                ..Walk::default()
            };
            self.draw_badge.apply_over(cx, live! { color: (accent) border_radius: 4.0 });
            self.draw_badge.draw_walk(cx, dot_walk);

            // Title text
            self.draw_comp_title.apply_over(cx, live! {
                text_style: { font_size: 13.0 }
                color: #FFFFFF
            });
            self.draw_comp_title.draw_walk(cx, Walk::fit(), Align::default(), title_text);
            cx.end_turtle();
        }

        // Description
        self.draw_comp_text.apply_over(cx, live! {
            text_style: { font_size: 12.0 }
            color: #CCCCCC
        });
        self.draw_comp_text.draw_walk(cx, Walk::fill_fit(), Align::default(), &description);

        self.draw_badge.apply_over(cx, live! { color: (bg_color) border_radius: 8.0 });
        self.draw_badge.end(cx);
    }

    // -------------------------------------------------------------------------
    // Avatar
    // -------------------------------------------------------------------------

    pub(super) fn render_avatar(
        &mut self,
        cx: &mut Cx2d,
        avatar: &AvatarComponent,
        data_model: &DataModel,
    ) {
        let fallback = resolve_string_value_scoped(
            &avatar.fallback,
            data_model,
            self.current_scope.as_deref(),
        );

        let size = match avatar.size.unwrap_or_default() {
            AvatarSize::XSmall => 24.0_f64,
            AvatarSize::Small  => 32.0,
            AvatarSize::Medium => 40.0,
            AvatarSize::Large  => 56.0,
            AvatarSize::XLarge => 80.0,
        };

        let color_idx = fallback.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize));
        let bg_color = avatar_color(color_idx);

        let walk = Walk {
            width: Size::Fixed(size),
            height: Size::Fixed(size),
            margin: Margin { left: 2.0, right: 2.0, top: 2.0, bottom: 2.0 },
            ..Walk::default()
        };
        let layout = Layout {
            align: Align { x: 0.5, y: 0.5 },
            ..Layout::default()
        };
        let font_size = (size * 0.35).max(10.0);

        self.draw_circle.apply_over(cx, live! { color: (bg_color) });
        self.draw_circle.begin(cx, walk, layout);
        self.draw_comp_text.apply_over(cx, live! {
            text_style: { font_size: (font_size) }
            color: #FFFFFF
        });
        let display: String = fallback.chars().take(2).collect();
        self.draw_comp_text.draw_walk(cx, Walk::fit(), Align::default(), &display);
        self.draw_circle.end(cx);
    }

    // -------------------------------------------------------------------------
    // Progress
    // -------------------------------------------------------------------------

    pub(super) fn render_progress(
        &mut self,
        cx: &mut Cx2d,
        progress: &ProgressComponent,
        data_model: &DataModel,
    ) {
        let raw_value = resolve_number_value_scoped(
            &progress.value,
            data_model,
            self.current_scope.as_deref(),
        );
        let normalized = (raw_value.clamp(0.0, 100.0) / 100.0) as f32;

        let fill_color = badge_color(progress.variant);
        let track_color = vec4(0.216, 0.255, 0.318, 1.0); // #374151

        let walk = Walk {
            width: Size::fill(),
            height: Size::Fixed(8.0),
            margin: Margin { left: 0.0, right: 0.0, top: 4.0, bottom: 4.0 },
            ..Walk::default()
        };

        self.draw_progress.apply_over(cx, live! {
            color: (fill_color)
            track_color: (track_color)
            progress: (normalized)
        });
        self.draw_progress.draw_walk(cx, walk);
    }

    // -------------------------------------------------------------------------
    // Spinner
    // -------------------------------------------------------------------------

    pub(super) fn render_spinner(
        &mut self,
        cx: &mut Cx2d,
        spinner: &SpinnerComponent,
    ) {
        // MVP: render spinner as a unicode rotation glyph via the label pool.
        // Full spinning animation requires a dedicated MpSpinner widget pool entry.
        let font_size = spinner.size.map(|s| s * 0.6).unwrap_or(14.0);
        let label_idx = self.label_count;
        self.label_count += 1;
        let label = self.pool_label(cx, label_idx);
        label.set_text("\u{27F3}"); // ⟳ CLOCKWISE GAPLESS OPEN-CENTRE ARROW
        label.apply_over(cx, live! {
            draw_text: { text_style: { font_size: (font_size) } }
        });
        let _ = label.draw_walk(cx, &mut Scope::empty(), Walk::fit());
    }

    // -------------------------------------------------------------------------
    // Switch
    // -------------------------------------------------------------------------

    pub(super) fn render_switch(
        &mut self,
        cx: &mut Cx2d,
        switch: &SwitchComponent,
        data_model: &DataModel,
        component_id: &str,
    ) {
        let switch_idx = self.switch_meta.len();

        let is_on =
            resolve_boolean_value_scoped(&switch.value, data_model, self.current_scope.as_deref());

        let binding_path = switch.binding.clone().or_else(|| {
            switch.value.as_path().map(|p| {
                if let Some(scope) = &self.current_scope {
                    format!("{}/{}", scope, p.trim_start_matches('/'))
                } else {
                    p.to_string()
                }
            })
        });

        let sw = self.pool_switch(cx, switch_idx);
        sw.set_on(cx, is_on);
        let _ = sw.draw_walk(cx, &mut Scope::empty(), Walk::fit());

        if let Some(ref label_val) = switch.label {
            let label_text = resolve_string_value_scoped(
                label_val,
                data_model,
                self.current_scope.as_deref(),
            );
            let label_idx = self.label_count;
            self.label_count += 1;
            let label = self.pool_label(cx, label_idx);
            label.set_text(&label_text);
            label.apply_over(cx, live! {
                draw_text: { text_style: { font_size: 13.0 } }
            });
            let _ = label.draw_walk(cx, &mut Scope::empty(), Walk::fit());
        }

        self.switch_meta.push((
            component_id.to_string(),
            binding_path,
            is_on,
            switch.action.clone(),
        ));
    }

    // -------------------------------------------------------------------------
    // Accordion
    // -------------------------------------------------------------------------

    pub(super) fn render_accordion(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        accordion: &AccordionComponent,
    ) {
        let items = accordion.items.clone();

        let outer_walk = Walk {
            width: Size::fill(),
            height: Size::fit(),
            margin: Margin { left: 0.0, right: 0.0, top: 4.0, bottom: 4.0 },
            ..Walk::default()
        };
        let outer_layout = Layout {
            flow: Flow::Down,
            spacing: 2.0,
            ..Layout::default()
        };

        let header_color = vec4(0.153, 0.200, 0.290, 1.0);
        let content_color = vec4(0.110, 0.145, 0.220, 1.0);

        cx.begin_turtle(outer_walk, outer_layout);

        for item in &items {
            let title_text = resolve_string_value_scoped(
                &item.title,
                data_model,
                self.current_scope.as_deref(),
            );

            // Section header row
            let header_walk = Walk {
                width: Size::fill(),
                height: Size::fit(),
                ..Walk::default()
            };
            let header_layout = Layout {
                flow: Flow::right(),
                align: Align { x: 0.0, y: 0.5 },
                padding: Padding { left: 12.0, right: 12.0, top: 10.0, bottom: 10.0 },
                ..Layout::default()
            };
            self.draw_badge.apply_over(cx, live! { color: (header_color) border_radius: 6.0 });
            self.draw_badge.begin(cx, header_walk, header_layout);
            self.draw_comp_title.apply_over(cx, live! {
                text_style: { font_size: 13.0 }
                color: #E0E0E0
            });
            self.draw_comp_title.draw_walk(cx, Walk::fit(), Align::default(), &title_text);
            self.draw_badge.end(cx);

            // Content area
            let content_walk = Walk {
                width: Size::fill(),
                height: Size::fit(),
                ..Walk::default()
            };
            let content_layout = Layout {
                flow: Flow::Down,
                padding: Padding { left: 12.0, right: 12.0, top: 8.0, bottom: 8.0 },
                ..Layout::default()
            };
            self.draw_badge.apply_over(cx, live! { color: (content_color) border_radius: 6.0 });
            self.draw_badge.begin(cx, content_walk, content_layout);
            self.render_component(cx, scope, surface, data_model, &item.content);
            self.draw_badge.apply_over(cx, live! { color: (content_color) border_radius: 6.0 });
            self.draw_badge.end(cx);
        }

        cx.end_turtle();
    }

    // -------------------------------------------------------------------------
    // Notification / Toast
    // -------------------------------------------------------------------------

    pub(super) fn render_notification(
        &mut self,
        cx: &mut Cx2d,
        _scope: &mut Scope,
        _surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        notification: &NotificationComponent,
    ) {
        let title_text = resolve_string_value_scoped(
            &notification.title,
            data_model,
            self.current_scope.as_deref(),
        );
        let desc_text = resolve_string_value_scoped(
            &notification.description,
            data_model,
            self.current_scope.as_deref(),
        );

        let bg_color = alert_bg_color(notification.variant);
        let accent = alert_accent_color(notification.variant);

        // Outer row: [4px accent bar] [body]
        let outer_walk = Walk {
            width: Size::fill(),
            height: Size::fit(),
            margin: Margin { left: 0.0, right: 0.0, top: 6.0, bottom: 6.0 },
            ..Walk::default()
        };
        let outer_layout = Layout {
            flow: Flow::right(),
            ..Layout::default()
        };
        cx.begin_turtle(outer_walk, outer_layout);

        // Left accent bar
        self.draw_badge.apply_over(cx, live! { color: (accent) border_radius: 0.0 });
        self.draw_badge.draw_walk(cx, Walk::new(Size::Fixed(4.0), Size::fill()));

        // Body
        let body_walk = Walk {
            width: Size::fill(),
            height: Size::fit(),
            ..Walk::default()
        };
        let body_layout = Layout {
            flow: Flow::Down,
            padding: Padding { left: 14.0, right: 14.0, top: 10.0, bottom: 10.0 },
            spacing: 3.0,
            ..Layout::default()
        };
        self.draw_badge.apply_over(cx, live! { color: (bg_color) border_radius: 0.0 });
        self.draw_badge.begin(cx, body_walk, body_layout);

        self.draw_comp_title.apply_over(cx, live! {
            text_style: { font_size: 13.0 }
            color: #FFFFFF
        });
        self.draw_comp_title.draw_walk(cx, Walk::fill_fit(), Align::default(), &title_text);

        self.draw_comp_text.apply_over(cx, live! {
            text_style: { font_size: 12.0 }
            color: #BBBBBB
        });
        self.draw_comp_text.draw_walk(cx, Walk::fill_fit(), Align::default(), &desc_text);

        self.draw_badge.end(cx);
        cx.end_turtle();
    }

    // -------------------------------------------------------------------------
    // Skeleton
    // -------------------------------------------------------------------------

    pub(super) fn render_skeleton(
        &mut self,
        cx: &mut Cx2d,
        skeleton: &SkeletonComponent,
    ) {
        let width = skeleton.width.map(Size::Fixed).unwrap_or_else(Size::fill);
        let height = skeleton.height.unwrap_or(16.0);
        let border_radius = skeleton.border_radius.unwrap_or(4.0);

        let walk = Walk {
            width,
            height: Size::Fixed(height),
            margin: Margin { left: 0.0, right: 0.0, top: 4.0, bottom: 4.0 },
            ..Walk::default()
        };
        let skeleton_color = vec4(0.216, 0.255, 0.318, 1.0);
        self.draw_badge.apply_over(cx, live! { color: (skeleton_color) border_radius: (border_radius) });
        self.draw_badge.draw_walk(cx, walk);
    }
}

