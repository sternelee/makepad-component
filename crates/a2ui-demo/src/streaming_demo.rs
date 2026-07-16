//! A2UI Streaming Demo
//!
//! Demonstrates connecting to an A2A server and receiving streaming UI updates.
//! Run the mock server first: python3 debug/mock_a2a_server.py

use makepad_component::a2ui::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.theme.*

    startup() do #(StreamingApp::script_component(vm)){
        ui: Root{
            main_window := Window{
                show_bg: true
                width: Fill
                height: Fill
                pass +: { clear_color: #1a1a2e }

                body := mod.widgets.SolidView{
                    width: Fill
                    height: Fill
                    flow: Down
                    padding: Inset{left: 20.0 right: 20.0 top: 20.0 bottom: 20.0}
                    spacing: 16.0
                    draw_bg.color: #1a1a2e

                    // Title
                    Label{
                        text: "A2UI Streaming Demo"
                        draw_text +: {
                            text_style: theme.font_bold{font_size: 24.0}
                            color: #FFFFFF
                        }
                    }

                    // Connection controls
                    View{
                        width: Fill
                        height: Fit
                        flow: Right
                        spacing: 10.0
                        align: Align{y: 0.5}

                        Label{
                            text: "Server:"
                            draw_text +: { color: #888888 }
                        }

                        server_url := Label{
                            text: "http://localhost:8080/rpc"
                            draw_text +: { color: #FFFFFF }
                        }

                        connect_btn := Button{
                            text: "Connect"
                        }
                    }

                    // Status
                    status_label := Label{
                        text: "Not connected"
                        draw_text +: { color: #888888 }
                    }

                    // A2UI Surface with scroll
                    ScrollYView{
                        width: Fill
                        height: Fill
                        show_bg: true
                        draw_bg +: {
                            color: instance(#222244)
                            pixel: fn() {
                                return self.color
                            }
                        }

                        View{
                            width: Fill
                            height: Fit
                            padding: Inset{left: 16.0 right: 16.0 top: 16.0 bottom: 16.0}

                            a2ui_surface := mod.widgets.A2uiSurface{
                                width: Fill
                                height: Fit
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct StreamingApp {
    #[live]
    ui: WidgetRef,

    #[rust]
    host: Option<A2uiHost>,

    #[rust]
    is_connecting: bool,
}

impl StreamingApp {
    fn connect(&mut self, cx: &mut Cx) {
        if self.host.is_some() {
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "Already connected");
            return;
        }

        let config = A2uiHostConfig {
            url: "http://localhost:8080/rpc".to_string(),
            auth_token: None,
        };

        let mut host = A2uiHost::new(config);

        match host.connect("Hello, show me a greeting UI") {
            Ok(()) => {
                self.ui
                    .label(cx, ids!(status_label))
                    .set_text(cx, "Connecting...");
                self.host = Some(host);
                self.is_connecting = true;
            }
            Err(e) => {
                self.ui
                    .label(cx, ids!(status_label))
                    .set_text(cx, &format!("Connection failed: {}", e));
            }
        }

        self.ui.redraw(cx);
    }

    fn poll_host(&mut self, cx: &mut Cx) {
        let Some(host) = &mut self.host else {
            return;
        };

        let events = host.poll_all();
        if events.is_empty() {
            return;
        }

        // Get surface widget
        let surface_ref = self.ui.widget(cx, ids!(a2ui_surface));

        for event in events {
            match event {
                A2uiHostEvent::Connected => {
                    self.ui
                        .label(cx, ids!(status_label))
                        .set_text(cx, "Connected! Receiving UI...");
                    self.is_connecting = false;
                }
                A2uiHostEvent::Message(msg) => {
                    // Process the A2UI message
                    if let Some(mut surface) = surface_ref.borrow_mut::<A2uiSurface>() {
                        let events = surface.process_message(msg);
                        log!("Processed message, {} events", events.len());
                    }
                    self.ui
                        .label(cx, ids!(status_label))
                        .set_text(cx, "Receiving UI updates...");
                }
                A2uiHostEvent::TaskStatus { task_id, state } => {
                    self.ui
                        .label(cx, ids!(status_label))
                        .set_text(cx, &format!("Task {}: {}", task_id, state));
                }
                A2uiHostEvent::Error(e) => {
                    self.ui
                        .label(cx, ids!(status_label))
                        .set_text(cx, &format!("Error: {}", e));
                }
                A2uiHostEvent::Disconnected => {
                    self.ui
                        .label(cx, ids!(status_label))
                        .set_text(cx, "Disconnected");
                    self.host = None;
                }
            }
        }

        self.ui.redraw(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Handle connect button
        if self.ui.button(cx, ids!(connect_btn)).clicked(actions) {
            self.connect(cx);
        }

        // Handle A2UI surface actions
        let surface_ref = self.ui.widget(cx, ids!(a2ui_surface));
        if let Some(item) = actions.find_widget_action(surface_ref.widget_uid()) {
            match item.cast::<A2uiSurfaceAction>() {
                A2uiSurfaceAction::UserAction(user_action) => {
                    // Forward action to server
                    if let Some(host) = &mut self.host {
                        if let Err(e) = host.send_action(&user_action) {
                            log!("Failed to send action: {}", e);
                        }
                    }
                    self.ui
                        .label(cx, ids!(status_label))
                        .set_text(cx, &format!("Action: {}", user_action.action.name));
                    self.ui.redraw(cx);
                }
                A2uiSurfaceAction::DataModelChanged {
                    surface_id,
                    path,
                    value,
                } => {
                    // Update local data model
                    if let Some(mut surface) = surface_ref.borrow_mut::<A2uiSurface>() {
                        if let Some(processor) = surface.processor_mut() {
                            if let Some(data_model) = processor.get_data_model_mut(&surface_id) {
                                // Implement radio button behavior for payment methods
                                // When one is selected, deselect all others
                                let payment_methods = [
                                    "/payment/creditCard",
                                    "/payment/paypal",
                                    "/payment/alipay",
                                    "/payment/wechat",
                                ];

                                if payment_methods.contains(&path.as_str()) {
                                    // If setting to true, deselect all others first
                                    if value == serde_json::Value::Bool(true) {
                                        for method in &payment_methods {
                                            if *method != path {
                                                data_model
                                                    .set(method, serde_json::Value::Bool(false));
                                            }
                                        }
                                    }
                                }

                                data_model.set(&path, value);
                            }
                        }
                    }
                    self.ui.redraw(cx);
                }
                _ => {}
            }
        }
    }
}

impl AppMain for StreamingApp {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);
        makepad_component::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // Poll host for new messages (on every frame if connecting)
        if self.host.is_some() {
            self.poll_host(cx);
        }

        // Handle UI events
        let actions = cx.capture_actions(|cx| {
            self.ui.handle_event(cx, event, &mut Scope::empty());
        });

        self.handle_actions(cx, &actions);
    }
}

// Module-level app_main! macro call - generates public app_main() function
app_main!(StreamingApp);

// Re-export as streaming_app_main for clarity
pub use app_main as streaming_app_main;
