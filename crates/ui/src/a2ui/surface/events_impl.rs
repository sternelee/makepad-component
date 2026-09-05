impl Widget for A2uiSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Handle NextFrame for continuous animation (audio bars breathing)
        if self.next_frame.is_event(event).is_some() {
            // Request another frame to keep animation loop running
            self.next_frame = cx.new_next_frame();
            self.redraw(cx);
            return;
        }

        // Forward events to 3D chart widgets FIRST for interactive rotation/zoom
        self.plot_surface3d.handle_event(cx, event, scope);
        self.plot_scatter3d.handle_event(cx, event, scope);
        self.plot_line3d.handle_event(cx, event, scope);

        let surface_id = self.get_surface_id();

        // Forward events to all widget pools and capture their actions
        let actions = cx.capture_actions(|cx| {
            for btn in self.mp_buttons.iter_mut() {
                btn.handle_event(cx, event, scope);
            }
            for cb in self.mp_checkboxes.iter_mut() {
                cb.handle_event(cx, event, scope);
            }
            for sl in self.mp_sliders.iter_mut() {
                sl.handle_event(cx, event, scope);
            }
            for ti in self.mp_text_inputs.iter_mut() {
                ti.handle_event(cx, event, scope);
            }
            for ni in self.mp_number_inputs.iter_mut() {
                ni.handle_event(cx, event, scope);
            }
            for sl in self.mp_searchable_lists.iter_mut() {
                sl.handle_event(cx, event, scope);
            }
            for cp in self.mp_color_pickers.iter_mut() {
                cp.handle_event(cx, event, scope);
            }
        });

        let mut needs_redraw = false;

        // Check button actions
        for (idx, btn) in self.mp_buttons.iter().enumerate() {
            if btn.clicked(&actions) {
                if let Some((component_id, action_def, btn_scope)) = self.button_meta.get(idx) {
                    if let Some(action_def) = action_def {
                        if let Some(processor) = &self.processor {
                            let user_action = processor.create_action(
                                &surface_id,
                                component_id,
                                action_def,
                                btn_scope.as_deref(),
                            );
                            cx.widget_action(self.widget_uid(), A2uiSurfaceAction::UserAction(user_action),
                            );
                        }
                    }
                }
            }
        }

        // Check checkbox actions
        for (idx, cb) in self.mp_checkboxes.iter().enumerate() {
            if let Some(action) = actions.find_widget_action(cb.widget_uid()) {
                if let MpCheckboxAction::Changed(new_value) = action.cast::<MpCheckboxAction>() {
                    if let Some((_, binding_path, _)) = self.checkbox_meta.get(idx) {
                        if let Some(path) = binding_path {
                            cx.widget_action(self.widget_uid(), A2uiSurfaceAction::DataModelChanged {
                                    surface_id: surface_id.clone(),
                                    path: path.clone(),
                                    value: serde_json::Value::Bool(new_value),
                                },
                            );
                            needs_redraw = true;
                        }
                    }
                }
            }
        }

        // Check slider actions
        for (idx, sl) in self.mp_sliders.iter().enumerate() {
            if let Some(action) = actions.find_widget_action(sl.widget_uid()) {
                if let MpSliderAction::Changed(slider_value) = action.cast::<MpSliderAction>() {
                    if let Some((_, binding_path, _, _, _)) = self.slider_meta.get(idx) {
                        if let Some(path) = binding_path {
                            let value = match slider_value {
                                crate::widgets::slider::SliderValue::Single(v) => serde_json::json!(v),
                                crate::widgets::slider::SliderValue::Range(start, end) => {
                                    serde_json::json!({"start": start, "end": end})
                                }
                            };
                            cx.widget_action(self.widget_uid(), A2uiSurfaceAction::DataModelChanged {
                                    surface_id: surface_id.clone(),
                                    path: path.clone(),
                                    value,
                                },
                            );
                            needs_redraw = true;
                        }
                    }
                }
            }
        }

        // Check number input actions (two-way binding)
        for (idx, ni) in self.mp_number_inputs.iter().enumerate() {
            if let Some(action) = actions.find_widget_action(ni.widget_uid()) {
                if let MpNumberInputAction::Changed(v) = action.cast::<MpNumberInputAction>() {
                    if let Some((_, binding_path)) = self.number_input_meta.get(idx) {
                        if let Some(path) = binding_path {
                            cx.widget_action(self.widget_uid(), A2uiSurfaceAction::DataModelChanged {
                                    surface_id: surface_id.clone(),
                                    path: path.clone(),
                                    value: serde_json::json!(v),
                                },
                            );
                            needs_redraw = true;
                        }
                    }
                }
            }
        }

        // Check color picker actions (write hex string back to the binding)
        for (idx, cp) in self.mp_color_pickers.iter().enumerate() {
            if let Some(action) = actions.find_widget_action(cp.widget_uid()) {
                if let MpColorPickerAction::Picked(c) = action.cast::<MpColorPickerAction>() {
                    if let Some((_, binding_path, _)) = self.color_picker_meta.get(idx) {
                        if let Some(path) = binding_path {
                            let hex = format!(
                                "#{:02X}{:02X}{:02X}",
                                (c.x.clamp(0.0, 1.0) * 255.0).round() as u8,
                                (c.y.clamp(0.0, 1.0) * 255.0).round() as u8,
                                (c.z.clamp(0.0, 1.0) * 255.0).round() as u8,
                            );
                            cx.widget_action(self.widget_uid(), A2uiSurfaceAction::DataModelChanged {
                                    surface_id: surface_id.clone(),
                                    path: path.clone(),
                                    value: serde_json::Value::String(hex),
                                },
                            );
                            needs_redraw = true;
                        }
                    }
                }
            }
        }

        // Check text input actions
        for (idx, ti) in self.mp_text_inputs.iter().enumerate() {
            if let Some(action) = actions.find_widget_action(ti.widget_uid()) {
                if let TextInputAction::Changed(new_text) = action.cast::<TextInputAction>() {
                    if let Some((_, binding_path, _)) = self.text_input_meta.get(idx) {
                        if let Some(path) = binding_path {
                            cx.widget_action(self.widget_uid(), A2uiSurfaceAction::DataModelChanged {
                                    surface_id: surface_id.clone(),
                                    path: path.clone(),
                                    value: serde_json::Value::String(new_text),
                                },
                            );
                            needs_redraw = true;
                        }
                    }
                }
            }
        }

        // Handle calendar events via MpCalendar widget
        if let Some(cal) = self.mp_calendar.as_mut() {
            let cal_actions = cx.capture_actions(|cx| {
                cal.handle_event(cx, event, scope);
            });
            if let Some((row, col)) = cal.cell_clicked(&cal_actions) {
                let user_action = crate::a2ui::message::UserAction {
                    surface_id: surface_id.clone(),
                    action: crate::a2ui::message::UserActionPayload {
                        name: "calendarCellClick".to_string(),
                        context: {
                            let mut ctx = std::collections::HashMap::new();
                            ctx.insert("row".to_string(), serde_json::json!(row));
                            ctx.insert("col".to_string(), serde_json::json!(col));
                            ctx
                        },
                    },
                    component_id: Some("calendar-view".to_string()),
                };
                cx.widget_action(self.widget_uid(), A2uiSurfaceAction::UserAction(user_action),
                );
                needs_redraw = true;
            }
        }

        // Handle audio player events (still uses manual Area hit testing)
        for (idx, area) in self.audio_player_areas.iter().enumerate() {
            match event.hits(cx, *area) {
                Hit::FingerHoverIn(_) => {
                    if self.hovered_audio_player_idx != Some(idx) {
                        self.hovered_audio_player_idx = Some(idx);
                        cx.set_cursor(MouseCursor::Hand);
                        needs_redraw = true;
                    }
                }
                Hit::FingerHoverOut(_) => {
                    if self.hovered_audio_player_idx == Some(idx) {
                        self.hovered_audio_player_idx = None;
                        cx.set_cursor(MouseCursor::Default);
                        needs_redraw = true;
                    }
                }
                Hit::FingerDown(_) => {
                    self.hovered_audio_player_idx = Some(idx);
                    if let Some((component_id, url, title)) = self.audio_player_data.get(idx).cloned() {
                        cx.widget_action(self.widget_uid(), A2uiSurfaceAction::PlayAudio {
                                component_id,
                                url,
                                title,
                            },
                        );
                    }
                    needs_redraw = true;
                }
                _ => {}
            }
        }

        if needs_redraw {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Load image textures if not loaded yet
        self.load_image_textures(cx);

        // Clear metadata from previous frame (pool instances are kept for animation state)
        self.button_meta.clear();
        self.checkbox_meta.clear();
        self.slider_meta.clear();
        self.text_input_meta.clear();
        self.audio_player_data.clear();
        self.label_count = 0;
        self.number_input_meta.clear();
        self.color_picker_meta.clear();
        self.tag_count = 0;
        self.step_indicator_count = 0;
        self.searchable_list_count = 0;
        self.avatar_group_count = 0;
        self.description_list_count = 0;
        self.inside_card = false;

        self.draw_bg.begin(cx, walk, self.layout);

        // Get surface and data model - clone to avoid borrow issues
        let surface_id = self.get_surface_id();
        let render_data = if let Some(processor) = &self.processor {
            let surface_opt = processor.get_surface(&surface_id);
            let data_model_opt = processor.get_data_model(&surface_id);

            if surface_opt.is_none() {
                log!("[draw_walk] No surface found for id: {}", surface_id);
            }
            if data_model_opt.is_none() {
                log!("[draw_walk] No data model found for id: {}", surface_id);
            }

            if let (Some(surface), Some(data_model)) = (surface_opt, data_model_opt) {
                Some((surface.clone(), data_model.clone()))
            } else {
                None
            }
        } else {
            log!("[draw_walk] No processor!");
            None
        };

        // Render the component tree
        if let Some((surface, data_model)) = render_data {
            let root_id = surface.root.clone();
            if !root_id.is_empty() {
                self.render_component(cx, scope, &surface, &data_model, &root_id);
            }
        }

        // Trim widget pools to match this frame's usage
        let button_count = self.button_meta.len();
        self.mp_buttons.truncate(button_count);

        let checkbox_count = self.checkbox_meta.len();
        self.mp_checkboxes.truncate(checkbox_count);

        let slider_count = self.slider_meta.len();
        self.mp_sliders.truncate(slider_count);

        self.mp_labels.truncate(self.label_count);

        let text_input_count = self.text_input_meta.len();
        self.mp_text_inputs.truncate(text_input_count);

        let audio_player_count = self.audio_player_data.len();
        if audio_player_count < self.audio_player_areas.len() {
            self.audio_player_areas.truncate(audio_player_count);
        }

        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();

        DrawStep::done()
    }
}
