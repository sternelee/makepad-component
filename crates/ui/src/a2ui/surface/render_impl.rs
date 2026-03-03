impl A2uiSurface {
    /// Render a component and its children recursively
    fn render_component(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        component_id: &str,
    ) {
        let Some(component_def) = surface.get_component(component_id) else {
            return;
        };

        // Clone component data to avoid borrow issues
        let component = component_def.component.clone();

        match &component {
            ComponentType::Column(col) => {
                self.render_column(cx, scope, surface, data_model, col);
            }
            ComponentType::Row(row) => {
                self.render_row(cx, scope, surface, data_model, row);
            }
            ComponentType::Text(text) => {
                self.render_text(cx, text, data_model);
            }
            ComponentType::Card(card) => {
                self.render_card(cx, scope, surface, data_model, card);
            }
            ComponentType::Button(btn) => {
                self.render_button(cx, scope, surface, data_model, btn, component_id);
            }
            ComponentType::Image(img) => {
                self.render_image(cx, img, data_model);
            }
            ComponentType::TextField(text_field) => {
                self.render_text_field(cx, text_field, data_model, component_id);
            }
            ComponentType::CheckBox(checkbox) => {
                self.render_checkbox(cx, checkbox, data_model, component_id);
            }
            ComponentType::Slider(slider) => {
                self.render_slider(cx, slider, data_model, component_id);
            }
            ComponentType::List(list) => {
                self.render_list(cx, scope, surface, data_model, list);
            }
            ComponentType::Chart(chart) => {
                self.render_chart(cx, scope, chart, data_model, component_id);
            }
            ComponentType::Calendar(calendar) => {
                self.render_calendar(cx, scope, calendar, data_model);
            }
            ComponentType::AudioPlayer(audio_player) => {
                self.render_audio_player(cx, audio_player, data_model, component_id);
            }
            ComponentType::ShaderStage(stage) => {
                self.render_shader_stage(cx, stage, data_model, component_id);
            }
            ComponentType::Divider(_) => {
                self.render_divider(cx);
            }
            // Raycast-style containers
            ComponentType::Detail(detail) => {
                self.render_detail(cx, scope, surface, data_model, detail);
            }
            ComponentType::Form(form) => {
                self.render_form(cx, scope, surface, data_model, form);
            }
            ComponentType::ActionPanel(ap) => {
                self.render_action_panel(cx, scope, surface, data_model, ap);
            }
            ComponentType::Grid(grid) => {
                self.render_grid(cx, scope, surface, data_model, grid);
            }
            // Raycast-style form components (using existing renderers where applicable)
            ComponentType::PasswordField(pf) => {
                self.render_password_field(cx, pf, data_model, component_id);
            }
            ComponentType::TextArea(ta) => {
                self.render_text_area(cx, ta, data_model, component_id);
            }
            ComponentType::DatePicker(dp) => {
                self.render_date_picker(cx, dp, data_model, component_id);
            }
            ComponentType::Dropdown(dd) => {
                self.render_dropdown(cx, scope, surface, data_model, dd);
            }
            ComponentType::TagPicker(tp) => {
                self.render_tag_picker(cx, scope, surface, data_model, tp);
            }
            ComponentType::FilePicker(fp) => {
                self.render_file_picker(cx, fp, data_model, component_id);
            }
            ComponentType::ListItem(li) => {
                self.render_list_item(cx, scope, surface, data_model, li);
            }
            ComponentType::DropdownItem(_) | ComponentType::DropdownSection(_) | ComponentType::TagPickerItem(_) => {
                // These are rendered as children of their parent components
            }
            _ => {
                // Unsupported component - skip for now
            }
        }
    }

    fn render_column(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        col: &ColumnComponent,
    ) {
        // Start a vertical layout
        let walk = Walk::fill_fit();
        let layout = Layout {
            flow: Flow::Down,
            spacing: 8.0,
            ..Layout::default()
        };

        cx.begin_turtle(walk, layout);

        // Render children
        let children = col.children.clone();
        self.render_children(cx, scope, surface, data_model, &children);

        cx.end_turtle();
    }

    fn render_row(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        row: &RowComponent,
    ) {
        // Start a horizontal layout - Fill width to allow spacer pattern
        let walk = Walk::fill_fit();
        let layout = Layout {
            flow: Flow::right(),
            spacing: 16.0,
            align: Align { x: 0.0, y: 0.5 },
            ..Layout::default()
        };

        cx.begin_turtle(walk, layout);

        // Render children with special handling for Row context
        let children = row.children.clone();
        self.render_row_children(cx, scope, surface, data_model, &children);

        cx.end_turtle();
    }

    /// Render children specifically for Row context (horizontal layout)
    /// If last child is a Button, it's placed in a Fill-width container with right alignment
    fn render_row_children(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        children: &ChildrenRef,
    ) {
        match children {
            ChildrenRef::ExplicitList(ids) => {
                let len = ids.len();

                // Check if last child is a Button for right-alignment
                let last_is_button = if len > 0 {
                    if let Some(comp) = surface.get_component(&ids[len - 1]) {
                        matches!(comp.component, ComponentType::Button(_))
                    } else {
                        false
                    }
                } else {
                    false
                };

                if last_is_button && len > 1 {
                    // Render non-button children with fixed min-width for alignment
                    // 280px is enough for longest product name
                    for child_id in ids.iter().take(len - 1) {
                        self.render_row_child_with_min_width(cx, scope, surface, data_model, child_id, 280.0);
                    }

                    // Render button
                    self.render_row_child(cx, scope, surface, data_model, &ids[len - 1]);
                } else {
                    // Render all children normally
                    for child_id in ids.iter() {
                        self.render_row_child(cx, scope, surface, data_model, child_id);
                    }
                }
            }
            ChildrenRef::Template { .. } => {
                // For templates in Row, use regular rendering
                self.render_children(cx, scope, surface, data_model, children);
            }
        }
    }

    /// Render a single child in Row context
    fn render_row_child(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        component_id: &str,
    ) {
        self.render_row_child_with_min_width(cx, scope, surface, data_model, component_id, 0.0);
    }

    /// Render a single child in Row context with minimum width for Column alignment
    fn render_row_child_with_min_width(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        component_id: &str,
        min_width: f64,
    ) {
        let Some(component_def) = surface.get_component(component_id) else {
            return;
        };

        let component = component_def.component.clone();

        match &component {
            ComponentType::Column(col) => {
                // Column with fixed width ensures buttons align
                // Height is Fit to adapt to content
                let walk = if min_width > 0.0 {
                    // Fixed width, Fit height using Walk::new()
                    Walk::new(Size::Fixed(min_width), Size::fit())
                } else {
                    Walk::fit()
                };
                let layout = Layout {
                    flow: Flow::Down,
                    spacing: 4.0,
                    ..Layout::default()
                };

                cx.begin_turtle(walk, layout);

                // Render Column children
                if let ChildrenRef::ExplicitList(ids) = &col.children {
                    for child_id in ids {
                        self.render_component(cx, scope, surface, data_model, child_id);
                    }
                }

                cx.end_turtle();
            }
            _ => {
                // Other components render normally
                self.render_component(cx, scope, surface, data_model, component_id);
            }
        }
    }

    fn render_children(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        children: &ChildrenRef,
    ) {
        match children {
            ChildrenRef::ExplicitList(ids) => {
                let ids_clone = ids.clone();
                for child_id in ids_clone {
                    self.render_component(cx, scope, surface, data_model, &child_id);
                }
            }
            ChildrenRef::Template {
                component_id,
                data_binding,
            } => {
                // Get array data from data model
                if let Some(array) = data_model.get_array(data_binding) {
                    let component_id = component_id.clone();
                    let data_binding = data_binding.clone();
                    for (index, _item) in array.iter().enumerate() {
                        let item_path = format!("{}/{}", data_binding, index);
                        self.render_template_item(
                            cx,
                            scope,
                            surface,
                            data_model,
                            &component_id,
                            &item_path,
                        );
                    }
                }
            }
        }
    }

    fn render_template_item(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        component_id: &str,
        item_path: &str,
    ) {
        // Set up scoped data model for template items
        let previous_scope = self.current_scope.take();
        self.current_scope = Some(item_path.to_string());

        // Render the component with scoped path resolution
        self.render_component(cx, scope, surface, data_model, component_id);

        // Restore previous scope
        self.current_scope = previous_scope;
    }

    // ============================================================================
    // Text -> MpLabel pool
    // ============================================================================

    fn render_text(&mut self, cx: &mut Cx2d, text: &TextComponent, data_model: &DataModel) {
        let text_value = resolve_string_value_scoped(
            &text.text,
            data_model,
            self.current_scope.as_deref(),
        );

        // Determine font size based on usage hint
        let font_size = match text.usage_hint {
            Some(TextUsageHint::H1) => 28.0,
            Some(TextUsageHint::H2) => 22.0,
            Some(TextUsageHint::H3) => 18.0,
            Some(TextUsageHint::H4) => 16.0,
            Some(TextUsageHint::H5) => 14.0,
            Some(TextUsageHint::Caption) => 12.0,
            Some(TextUsageHint::Code) => 13.0,
            _ => 14.0, // Body default
        };

        if self.inside_card {
            // Inside card containers: use DrawText directly (avoids draw ordering
            // issues with MpLabel's #[redraw] scope inside DrawQuad.begin/end)
            self.draw_card_text.text_style.font_size = font_size;
            self.draw_card_text.draw_walk(cx, Walk::fit(), Align::default(), &text_value);
        } else {
            // Outside cards: use MpLabel pool for full widget features
            let label_idx = self.label_count;
            self.label_count += 1;
            let label = self.pool_label(cx, label_idx);
            label.set_text(&text_value);
            label.apply_over(cx, live! {
                draw_text: { text_style: { font_size: (font_size) } }
            });
            let _ = label.draw_walk(cx, &mut Scope::empty(), Walk::fit());
        }
    }

    fn render_image(&mut self, cx: &mut Cx2d, img: &ImageComponent, data_model: &DataModel) {
        // Use scoped resolution for template rendering
        let url = resolve_string_value_scoped(
            &img.url,
            data_model,
            self.current_scope.as_deref(),
        );

        // Determine size based on usage hint
        let (width, height) = match img.usage_hint {
            Some(ImageUsageHint::Icon) => (24.0, 24.0),
            Some(ImageUsageHint::Avatar) => (48.0, 48.0),
            Some(ImageUsageHint::SmallFeature) => (64.0, 64.0),
            Some(ImageUsageHint::MediumFeature) => (120.0, 80.0),
            Some(ImageUsageHint::LargeFeature) => (200.0, 150.0),
            Some(ImageUsageHint::Header) => (300.0, 100.0),
            _ => (80.0, 80.0), // Default size
        };

        let walk = Walk::new(Size::Fixed(width), Size::Fixed(height));

        // Get texture index (avoid borrow conflict)
        let texture_idx = self.get_texture_index_for_url(&url);

        // Try to render actual image if texture is available
        if let Some(idx) = texture_idx {
            // Get texture reference by index
            let texture = match idx {
                0 => self.texture_headphones.as_ref(),
                1 => self.texture_mouse.as_ref(),
                2 => self.texture_keyboard.as_ref(),
                3 => self.texture_alipay.as_ref(),
                4 => self.texture_wechat.as_ref(),
                _ => None,
            };

            if let Some(tex) = texture {
                // Draw actual image with texture
                self.draw_image.draw_vars.set_texture(0, tex);
                self.draw_image.draw_walk(cx, walk);
                return;
            }
        }

        // Fallback to placeholder
        let layout = Layout {
            padding: Padding {
                left: 4.0,
                right: 4.0,
                top: 4.0,
                bottom: 4.0,
            },
            align: Align { x: 0.5, y: 0.5 },
            ..Layout::default()
        };

        self.draw_image_placeholder.begin(cx, walk, layout);
        self.draw_image_text.draw_walk(cx, Walk::fit(), Align::default(), "IMG");
        self.draw_image_placeholder.end(cx);
    }

    // ============================================================================
    // Card rendering (still uses draw_card begin/end for container background)
    // ============================================================================

    fn render_card(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        card: &CardComponent,
    ) {
        let walk = Walk {
            margin: Margin { left: 0.0, right: 0.0, top: 8.0, bottom: 8.0 },
            ..Walk::fill_fit()
        };
        let layout = Layout {
            flow: Flow::Down,
            padding: Padding {
                left: 16.0,
                right: 16.0,
                top: 12.0,
                bottom: 12.0,
            },
            ..Layout::default()
        };

        // Begin card background
        self.draw_card.begin(cx, walk, layout);
        self.inside_card = true;

        // Render child content
        let child = card.child.clone();
        self.render_component(cx, scope, surface, data_model, &child);

        // End card
        self.inside_card = false;
        self.draw_card.end(cx);
    }

    // ============================================================================
    // Button -> MpButton pool
    // ============================================================================

    fn render_button(
        &mut self,
        cx: &mut Cx2d,
        _scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        btn: &ButtonComponent,
        component_id: &str,
    ) {
        let button_idx = self.button_meta.len();

        // Resolve button text from child component
        let button_text = self.resolve_button_text(surface, data_model, &btn.child);

        // Get or grow button from pool
        let button = self.pool_button(cx, button_idx);

        // Set button text
        button.set_text(&button_text);

        // Draw the button widget
        let _ = button.draw_walk(cx, &mut Scope::empty(), Walk::fit());

        // Store metadata
        self.button_meta.push((
            component_id.to_string(),
            btn.action.clone(),
            self.current_scope.clone(),
        ));
    }

    /// Resolve button text by looking at child component (usually a Text component)
    fn resolve_button_text(
        &self,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        child_id: &str,
    ) -> String {
        if let Some(component_def) = surface.get_component(child_id) {
            if let ComponentType::Text(text) = &component_def.component {
                return resolve_string_value_scoped(
                    &text.text,
                    data_model,
                    self.current_scope.as_deref(),
                );
            }
        }
        String::new()
    }

    // ============================================================================
    // TextField -> TextInput pool
    // ============================================================================

    fn render_text_field(
        &mut self,
        cx: &mut Cx2d,
        text_field: &TextFieldComponent,
        data_model: &DataModel,
        component_id: &str,
    ) {
        let text_input_idx = self.text_input_meta.len();

        // Get current value from data model
        let current_value = resolve_string_value_scoped(
            &text_field.text,
            data_model,
            self.current_scope.as_deref(),
        );

        // Get placeholder text
        let placeholder = text_field
            .placeholder
            .as_ref()
            .map(|p| resolve_string_value_scoped(p, data_model, self.current_scope.as_deref()))
            .unwrap_or_default();

        // Get binding path for two-way binding
        let binding_path = text_field.text.as_path().map(|p| {
            if let Some(scope) = &self.current_scope {
                format!("{}/{}", scope, p.trim_start_matches('/'))
            } else {
                p.to_string()
            }
        });

        // Get or grow text input from pool
        let text_input = self.pool_text_input(cx, text_input_idx);

        // Set text and placeholder
        text_input.set_text(cx, &current_value);
        if !placeholder.is_empty() {
            text_input.set_empty_text(cx, placeholder.clone());
        }

        // Draw the text input widget
        let _ = text_input.draw_walk(cx, &mut Scope::empty(), Walk::new(Size::Fixed(300.0), Size::fit()));

        // Store metadata
        self.text_input_meta.push((
            component_id.to_string(),
            binding_path,
            current_value,
        ));
    }

    // ============================================================================
    // CheckBox -> MpCheckbox pool
    // ============================================================================

    fn render_checkbox(
        &mut self,
        cx: &mut Cx2d,
        checkbox: &CheckBoxComponent,
        data_model: &DataModel,
        component_id: &str,
    ) {
        let checkbox_idx = self.checkbox_meta.len();

        // Get current checked state
        let is_checked =
            resolve_boolean_value_scoped(&checkbox.value, data_model, self.current_scope.as_deref());

        // Get label text
        let label = checkbox
            .label
            .as_ref()
            .map(|l| resolve_string_value_scoped(l, data_model, self.current_scope.as_deref()))
            .unwrap_or_default();

        // Get binding path
        let binding_path = checkbox.value.as_path().map(|p| {
            if let Some(scope) = &self.current_scope {
                format!("{}/{}", scope, p.trim_start_matches('/'))
            } else {
                p.to_string()
            }
        });

        // Get or grow checkbox from pool
        let cb = self.pool_checkbox(cx, checkbox_idx);

        // Set state
        cb.set_checked(cx, is_checked);
        if !label.is_empty() {
            cb.set_text(&label);
        }

        // Draw the checkbox widget
        let _ = cb.draw_walk(cx, &mut Scope::empty(), Walk::fit());

        // Store metadata
        self.checkbox_meta
            .push((component_id.to_string(), binding_path, is_checked));
    }

    // ============================================================================
    // Slider -> MpSlider pool
    // ============================================================================

    fn render_slider(
        &mut self,
        cx: &mut Cx2d,
        slider: &SliderComponent,
        data_model: &DataModel,
        component_id: &str,
    ) {
        let slider_idx = self.slider_meta.len();

        // Get binding path: prefer explicit `binding` field, fall back to value path
        let binding_path = slider.binding.clone().or_else(|| {
            slider.value.as_path().map(|p| {
                if let Some(scope) = &self.current_scope {
                    format!("{}/{}", scope, p.trim_start_matches('/'))
                } else {
                    p.to_string()
                }
            })
        });

        // Get current value: read from data model via binding path, fall back to literal value
        let current_value = if let Some(ref path) = binding_path {
            data_model.get_number(path).unwrap_or_else(|| {
                resolve_number_value_scoped(&slider.value, data_model, self.current_scope.as_deref())
            })
        } else {
            resolve_number_value_scoped(&slider.value, data_model, self.current_scope.as_deref())
        };
        let min = slider.min.unwrap_or(0.0);
        let max = slider.max.unwrap_or(100.0);

        // Get or grow slider from pool
        let sl = self.pool_slider(cx, slider_idx);

        // Set range and value (but NOT during active drag - let user control position)
        sl.set_range(min, max);
        if !sl.is_dragging() {
            sl.set_single_value(cx, current_value);
        }

        // Draw the slider widget
        let _ = sl.draw_walk(cx, &mut Scope::empty(), Walk::new(Size::Fixed(200.0), Size::Fixed(24.0)));

        // Store metadata
        self.slider_meta.push((
            component_id.to_string(),
            binding_path,
            min,
            max,
            current_value,
        ));
    }

    // ============================================================================
    // Divider rendering
    // ============================================================================

    fn render_divider(&mut self, cx: &mut Cx2d) {
        let walk = Walk {
            width: Size::fill(),
            height: Size::Fixed(1.0),
            margin: Margin { top: 8.0, bottom: 8.0, left: 0.0, right: 0.0 },
            ..Walk::default()
        };
        self.draw_divider.draw_walk(cx, walk);
    }

    // ============================================================================
    // List Rendering
    // ============================================================================

    fn render_list(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        list: &ListComponent,
    ) {
        let walk = Walk::fill_fit();
        let layout = Layout {
            flow: Flow::Down,
            spacing: 8.0,
            ..Layout::default()
        };

        cx.begin_turtle(walk, layout);

        // Render children (supports template binding)
        let children = list.children.clone();
        self.render_children(cx, scope, surface, data_model, &children);

        cx.end_turtle();
    }

    // ============================================================================
    // Chart Rendering
    // ============================================================================

    /// Default color palette for charts
    fn chart_palette(index: usize) -> Vec4 {
        const COLORS: &[(f32, f32, f32)] = &[
            (0.231, 0.510, 0.965),  // #3B82F6 blue
            (0.161, 0.714, 0.467),  // #28B677 green
            (0.937, 0.333, 0.314),  // #EF5550 red
            (0.969, 0.643, 0.176),  // #F7A42D orange
            (0.545, 0.361, 0.886),  // #8B5CE2 purple
            (0.071, 0.741, 0.812),  // #12BDD0 teal
            (0.957, 0.486, 0.667),  // #F47CAA pink
            (0.400, 0.553, 0.200),  // #668D33 olive
        ];
        let (r, g, b) = COLORS[index % COLORS.len()];
        Vec4 { x: r, y: g, z: b, w: 1.0 }
    }

    /// Parse a hex color string to Vec4
    fn parse_hex_color(hex: &str) -> Option<Vec4> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 { return None; }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(Vec4 { x: r, y: g, z: b, w: 1.0 })
    }

    fn get_chart_color(&self, chart: &ChartComponent, index: usize) -> Vec4 {
        if index < chart.colors.len() {
            if let Some(color) = Self::parse_hex_color(&chart.colors[index]) {
                return color;
            }
        }
        Self::chart_palette(index)
    }

    /// Estimate text width in pixels for chart layout
    fn estimate_text_width(text: &str, font_size: f64) -> f64 {
        let avg_char_width = font_size * 0.55;
        text.len() as f64 * avg_char_width
    }

    // ============================================================================
    // Detail Component (Raycast-style)
    // ============================================================================

    fn render_detail(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        detail: &DetailComponent,
    ) {
        let walk = Walk::fill_fit();
        let layout = Layout {
            flow: Flow::Down,
            spacing: 12.0,
            padding: Padding {
                left: 16.0,
                right: 16.0,
                top: 12.0,
                bottom: 12.0,
            },
            ..Layout::default()
        };

        cx.begin_turtle(walk, layout);

        // Render markdown content if present
        if let Some(markdown) = &detail.markdown {
            let content = resolve_string_value_scoped(
                markdown,
                data_model,
                self.current_scope.as_deref(),
            );
            // For now, render as plain text (markdown rendering would need additional implementation)
            self.draw_card_text.draw_walk(cx, Walk::fit(), Align::default(), &content);
        }

        // Render metadata if present
        if let Some(metadata) = &detail.metadata {
            for item in metadata {
                let key = resolve_string_value_scoped(
                    &item.key,
                    data_model,
                    self.current_scope.as_deref(),
                );
                let value = resolve_string_value_scoped(
                    &item.value,
                    data_model,
                    self.current_scope.as_deref(),
                );
                // Render key-value pair
                let label_idx = self.label_count;
                self.label_count += 1;
                let label = self.pool_label(cx, label_idx);
                label.set_text(&format!("{}: {}", key, value));
                let _ = label.draw_walk(cx, &mut Scope::empty(), Walk::fit());
            }
        }

        // Render actions if present
        if let Some(actions_id) = &detail.actions {
            self.render_component(cx, scope, surface, data_model, actions_id);
        }

        cx.end_turtle();
    }

    // ============================================================================
    // Form Component (Raycast-style)
    // ============================================================================

    fn render_form(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        form: &FormComponent,
    ) {
        let walk = Walk::fill_fit();
        let layout = Layout {
            flow: Flow::Down,
            spacing: 16.0,
            padding: Padding {
                left: 16.0,
                right: 16.0,
                top: 12.0,
                bottom: 12.0,
            },
            ..Layout::default()
        };

        cx.begin_turtle(walk, layout);

        // Render form children (form items)
        let children = form.children.clone();
        self.render_children(cx, scope, surface, data_model, &children);

        // Render actions at the bottom
        if let Some(actions_id) = &form.actions {
            // Add some spacing before actions
            let spacer_walk = Walk::new(Size::fill(), Size::Fixed(16.0));
            self.draw_divider.draw_walk(cx, spacer_walk);

            self.render_component(cx, scope, surface, data_model, actions_id);
        }

        cx.end_turtle();
    }

    // ============================================================================
    // ActionPanel Component (Raycast-style)
    // ============================================================================

    fn render_action_panel(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        ap: &ActionPanelComponent,
    ) {
        let walk = Walk::fill_fit();
        let layout = Layout {
            flow: Flow::Down,
            spacing: 4.0,
            padding: Padding {
                left: 8.0,
                right: 8.0,
                top: 4.0,
                bottom: 4.0,
            },
            ..Layout::default()
        };

        cx.begin_turtle(walk, layout);

        // Render action children (buttons)
        let children = ap.children.clone();
        self.render_children(cx, scope, surface, data_model, &children);

        cx.end_turtle();
    }

    // ============================================================================
    // Grid Component (Raycast-style)
    // ============================================================================

    fn render_grid(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        grid: &GridComponent,
    ) {
        let columns = grid.columns.unwrap_or(3);

        let walk = Walk::fill_fit();
        let layout = Layout {
            flow: Flow::right(),
            spacing: 12.0,
            align: Align { x: 0.0, y: 0.0 },
            ..Layout::default()
        };

        cx.begin_turtle(walk, layout);

        // Render children (grid items)
        let children = grid.children.clone();
        self.render_children(cx, scope, surface, data_model, &children);

        cx.end_turtle();
    }

    // ============================================================================
    // PasswordField Component (Raycast-style)
    // ============================================================================

    fn render_password_field(
        &mut self,
        cx: &mut Cx2d,
        pf: &PasswordFieldComponent,
        data_model: &DataModel,
        component_id: &str,
    ) {
        // Similar to text field but with hidden input
        let text_input_idx = self.text_input_meta.len();

        let current_value = pf
            .value
            .as_ref()
            .map(|v| resolve_string_value_scoped(v, data_model, self.current_scope.as_deref()))
            .unwrap_or_default();

        let placeholder = pf
            .placeholder
            .as_ref()
            .map(|p| resolve_string_value_scoped(p, data_model, self.current_scope.as_deref()))
            .unwrap_or_default();

        let binding_path = pf.value.as_ref().and_then(|v| v.as_path().map(|p| {
            if let Some(scope) = &self.current_scope {
                format!("{}/{}", scope, p.trim_start_matches('/'))
            } else {
                p.to_string()
            }
        }));

        let text_input = self.pool_text_input(cx, text_input_idx);
        text_input.set_text(cx, &current_value);
        if !placeholder.is_empty() {
            text_input.set_empty_text(cx, placeholder);
        }

        // Password field - in production, would set a flag to hide text
        let _ = text_input.draw_walk(cx, &mut Scope::empty(), Walk::new(Size::Fixed(200.0), Size::fit()));

        self.text_input_meta.push((
            component_id.to_string(),
            binding_path,
            current_value,
        ));
    }

    // ============================================================================
    // TextArea Component (Raycast-style)
    // ============================================================================

    fn render_text_area(
        &mut self,
        cx: &mut Cx2d,
        ta: &TextAreaComponent,
        data_model: &DataModel,
        component_id: &str,
    ) {
        // For now, reuse text field rendering with larger size
        // In production, this would render a multi-line text input
        let text_input_idx = self.text_input_meta.len();

        let current_value = ta
            .value
            .as_ref()
            .map(|v| resolve_string_value_scoped(v, data_model, self.current_scope.as_deref()))
            .unwrap_or_default();

        let placeholder = ta
            .placeholder
            .as_ref()
            .map(|p| resolve_string_value_scoped(p, data_model, self.current_scope.as_deref()))
            .unwrap_or_default();

        let binding_path = ta.value.as_ref().and_then(|v| v.as_path().map(|p| {
            if let Some(scope) = &self.current_scope {
                format!("{}/{}", scope, p.trim_start_matches('/'))
            } else {
                p.to_string()
            }
        }));

        let text_input = self.pool_text_input(cx, text_input_idx);
        text_input.set_text(cx, &current_value);
        if !placeholder.is_empty() {
            text_input.set_empty_text(cx, placeholder);
        }

        // Render with larger height for textarea
        let _ = text_input.draw_walk(cx, &mut Scope::empty(), Walk::new(Size::Fixed(300.0), Size::Fixed(100.0)));

        self.text_input_meta.push((
            component_id.to_string(),
            binding_path,
            current_value,
        ));
    }

    // ============================================================================
    // DatePicker Component (Raycast-style)
    // ============================================================================

    fn render_date_picker(
        &mut self,
        cx: &mut Cx2d,
        dp: &DatePickerComponent,
        data_model: &DataModel,
        component_id: &str,
    ) {
        // Render as a text input with date picker trigger
        let text_input_idx = self.text_input_meta.len();

        let current_value = dp
            .value
            .as_ref()
            .map(|v| resolve_string_value_scoped(v, data_model, self.current_scope.as_deref()))
            .unwrap_or_default();

        let placeholder = match dp.date_type {
            Some(DatePickerType::Date) => "Select date...",
            Some(DatePickerType::DateTime) => "Select date and time...",
            Some(DatePickerType::Time) => "Select time...",
            None => "Select date...",
        };

        let binding_path = dp.value.as_ref().and_then(|v| v.as_path().map(|p| {
            if let Some(scope) = &self.current_scope {
                format!("{}/{}", scope, p.trim_start_matches('/'))
            } else {
                p.to_string()
            }
        }));

        let text_input = self.pool_text_input(cx, text_input_idx);
        text_input.set_text(cx, &current_value);
        text_input.set_empty_text(cx, placeholder.to_string());

        let _ = text_input.draw_walk(cx, &mut Scope::empty(), Walk::new(Size::Fixed(200.0), Size::fit()));

        self.text_input_meta.push((
            component_id.to_string(),
            binding_path,
            current_value,
        ));
    }

    // ============================================================================
    // Dropdown Component (Raycast-style)
    // ============================================================================

    fn render_dropdown(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        dd: &DropdownComponent,
    ) {
        // Render as a button that opens a popover with options
        // For now, render current value as text
        let text_input_idx = self.text_input_meta.len();

        let current_value = dd
            .value
            .as_ref()
            .map(|v| resolve_string_value_scoped(v, data_model, self.current_scope.as_deref()))
            .unwrap_or_else(|| {
                dd.placeholder
                    .as_ref()
                    .map(|p| resolve_string_value_scoped(p, data_model, self.current_scope.as_deref()))
                    .unwrap_or_default()
            });

        let placeholder = dd
            .placeholder
            .as_ref()
            .map(|p| resolve_string_value_scoped(p, data_model, self.current_scope.as_deref()))
            .unwrap_or_else(|| "Select...".to_string());

        let binding_path = dd.value.as_ref().and_then(|v| v.as_path().map(|p| {
            if let Some(scope) = &self.current_scope {
                format!("{}/{}", scope, p.trim_start_matches('/'))
            } else {
                p.to_string()
            }
        }));

        let text_input = self.pool_text_input(cx, text_input_idx);
        text_input.set_text(cx, &current_value);
        if current_value.is_empty() {
            text_input.set_empty_text(cx, placeholder);
        }

        let _ = text_input.draw_walk(cx, &mut Scope::empty(), Walk::new(Size::Fixed(200.0), Size::fit()));

        self.text_input_meta.push((
            format!("{}_dropdown", dd.id),
            binding_path,
            current_value,
        ));
    }

    // ============================================================================
    // TagPicker Component (Raycast-style)
    // ============================================================================

    fn render_tag_picker(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        tp: &TagPickerComponent,
    ) {
        // Render selected tags as chips + add button
        let text_input_idx = self.text_input_meta.len();

        // Get selected values
        let selected_values: Vec<String> = tp
            .value
            .as_ref()
            .map(|values| {
                values
                    .iter()
                    .map(|v| resolve_string_value_scoped(v, data_model, self.current_scope.as_deref()))
                    .collect()
            })
            .unwrap_or_default();

        let display_text = if selected_values.is_empty() {
            tp.placeholder
                .as_ref()
                .map(|p| resolve_string_value_scoped(p, data_model, self.current_scope.as_deref()))
                .unwrap_or_else(|| "Select tags...".to_string())
        } else {
            selected_values.join(", ")
        };

        let text_input = self.pool_text_input(cx, text_input_idx);
        text_input.set_text(cx, &display_text);

        let _ = text_input.draw_walk(cx, &mut Scope::empty(), Walk::new(Size::Fixed(300.0), Size::fit()));

        // Store metadata (simplified - full implementation would handle tag array)
        self.text_input_meta.push((
            format!("{}_tagpicker", tp.id),
            None,
            display_text,
        ));
    }

    // ============================================================================
    // FilePicker Component (Raycast-style)
    // ============================================================================

    fn render_file_picker(
        &mut self,
        cx: &mut Cx2d,
        fp: &FilePickerComponent,
        data_model: &DataModel,
        component_id: &str,
    ) {
        // Render as text input showing selected file path + browse button
        let text_input_idx = self.text_input_meta.len();

        let current_value = fp
            .value
            .as_ref()
            .map(|values| {
                values
                    .iter()
                    .map(|v| resolve_string_value_scoped(v, data_model, self.current_scope.as_deref()))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        let placeholder = "Select file...".to_string();

        let text_input = self.pool_text_input(cx, text_input_idx);
        text_input.set_text(cx, &current_value);
        if current_value.is_empty() {
            text_input.set_empty_text(cx, placeholder);
        }

        let _ = text_input.draw_walk(cx, &mut Scope::empty(), Walk::new(Size::Fixed(300.0), Size::fit()));

        self.text_input_meta.push((
            component_id.to_string(),
            None,
            current_value,
        ));
    }

    // ============================================================================
    // ListItem Component (Raycast-style)
    // ============================================================================

    fn render_list_item(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        surface: &crate::a2ui::processor::Surface,
        data_model: &DataModel,
        item: &ListItemComponent,
    ) {
        let walk = Walk::fill_fit();
        let layout = Layout {
            flow: Flow::right(),
            spacing: 12.0,
            align: Align { x: 0.0, y: 0.5 },
            padding: Padding {
                left: 12.0,
                right: 12.0,
                top: 8.0,
                bottom: 8.0,
            },
            ..Layout::default()
        };

        cx.begin_turtle(walk, layout);

        // Render icon if present
        if let Some(icon) = &item.icon {
            let icon_name = resolve_string_value_scoped(
                icon,
                data_model,
                self.current_scope.as_deref(),
            );
            // For now, render as text placeholder
            let label_idx = self.label_count;
            self.label_count += 1;
            let label = self.pool_label(cx, label_idx);
            label.set_text(&format!("[{}]", icon_name));
            let _ = label.draw_walk(cx, &mut Scope::empty(), Walk::fit());
        }

        // Render child content
        let child = item.child.clone();
        self.render_component(cx, scope, surface, data_model, &child);

        // Render accessory if present
        if let Some(accessory_id) = &item.accessory {
            self.render_component(cx, scope, surface, data_model, accessory_id);
        }

        cx.end_turtle();
    }
}
