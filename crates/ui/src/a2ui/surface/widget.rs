//! A2uiSurface widget definition and core implementation

#[allow(unused_imports)]
use makepad_plot::plot::area::AreaChart;
#[allow(unused_imports)]
use makepad_plot::plot::financial::CandlestickChart;
use makepad_plot::*;
use makepad_widgets::*;

use crate::a2ui::{
    chart_bridge,
    data_model::DataModel,
    message::*,
    processor::{
        resolve_boolean_value_scoped, resolve_number_value_scoped, resolve_string_value_scoped,
        A2uiMessageProcessor, ProcessorEvent,
    },
};
use crate::widgets::{
    avatar_group::MpAvatarGroup,
    button::MpButton,
    calendar::MpCalendar,
    checkbox::{MpCheckbox, MpCheckboxAction},
    color_picker::{MpColorPicker, MpColorPickerAction},
    description_list::{MpDescriptionItem, MpDescriptionList},
    label::MpLabel,
    number_input::{MpNumberInput, MpNumberInputAction},
    searchable_list::{MpSearchableList, MpSearchableListAction},
    slider::{MpSlider, MpSliderAction},
    step_indicator::MpStepIndicator,
    tag::MpTag,
};

use super::draw_types::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.theme.*

    mod.widgets.A2uiSurfaceBase = #(A2uiSurface::register_widget(vm))

    // Dark-themed text input for A2UI forms (used by the widget pool)
    mod.widgets.A2uiTextInput = mod.widgets.TextInput{
        width: 200
        height: Fit
        padding: Inset{left: 12 right: 12 top: 8 bottom: 8}
        empty_text: ""

        draw_bg +: {
            hover: instance(0.0)
            focus: instance(0.0)

            border_radius: uniform(6.0)
            border_width: uniform(1.0)
            bg_color: uniform(#2a3a5a)
            border_color: uniform(#5588bb)
            border_color_focus: uniform(#3B82F6)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    self.border_radius
                )
                sdf.fill_keep(self.bg_color)
                let border = mix(self.border_color, self.border_color_focus, self.focus)
                sdf.stroke(border, self.border_width)
                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_regular{font_size: 14.0}
            get_color: fn() {
                return mix(#FFFFFF, #888888, self.empty)
            }
        }

        draw_cursor +: {
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 1.0)
                sdf.fill(mix(#0000, #3B82F6, self.focus * (1.0 - self.blink)))
                return sdf.result
            }
        }

        draw_selection +: {
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 2.0)
                sdf.fill(#3B82F620)
                return sdf.result
            }
        }

        animator: Animator{
            hover: {default: @off
                off: AnimatorState{from: {all: Forward{duration: 0.15}} apply: {draw_bg: {hover: 0.0}}}
                on: AnimatorState{from: {all: Forward{duration: 0.1}} apply: {draw_bg: {hover: 1.0}}}
            }
            focus: {default: @off
                off: AnimatorState{from: {all: Forward{duration: 0.2}} apply: {draw_bg: {focus: 0.0} draw_cursor: {focus: 0.0}}}
                on: AnimatorState{from: {all: Snap} apply: {draw_bg: {focus: 1.0} draw_cursor: {focus: 1.0}}}
            }
        }
    }

    mod.widgets.A2uiSurface = mod.std.set_type_default() do mod.widgets.A2uiSurfaceBase{
        width: Fill
        height: Fill
        flow: Down

        draw_bg +: {
            color: #1a1a2e
        }

        // Card background (DrawColor begin/end pattern for Card containers)
        draw_card +: {
            color: #2a3a5a
            border_color: uniform(#5588bb)
            border_radius: uniform(8.0)
            border_width: uniform(1.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    max(1.0, self.border_radius)
                )
                sdf.fill_keep(self.color)
                sdf.stroke(self.border_color, self.border_width)
                return sdf.result
            }
        }

        draw_image_placeholder +: {
            border_radius: uniform(4.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(1.0, 1.0, self.rect_size.x - 2.0, self.rect_size.y - 2.0, self.border_radius)
                let stripe_width = 8.0
                let pos = self.pos * self.rect_size
                let stripe = pos.x + pos.y - (stripe_width * 2.0) * floor((pos.x + pos.y) / (stripe_width * 2.0))
                let is_stripe = step(stripe_width, stripe)
                let color1 = vec4(0.25, 0.28, 0.35, 1.0)
                let color2 = vec4(0.30, 0.33, 0.40, 1.0)
                let bg_color = mix(color1, color2, is_stripe)
                sdf.fill(bg_color)
                return sdf.result
            }
        }

        draw_image_text +: {
            text_style: theme.font_regular{font_size: 11.0}
            color: #888888
        }

        draw_chart_text +: {
            text_style: theme.font_regular{font_size: 10.0}
            color: #AABBCC
        }

        // Divider draw
        draw_divider +: {
            color: #5588bb
        }

        // makepad-plot chart widgets
        plot_line := mod.widgets.LinePlot{}
        plot_bar := mod.widgets.BarPlot{}
        plot_scatter := mod.widgets.ScatterPlot{}
        plot_pie := mod.widgets.PieChart{}
        plot_area := mod.widgets.AreaChart{}
        plot_radar := mod.widgets.RadarChart{}
        plot_gauge := mod.widgets.GaugeChart{}
        plot_bubble := mod.widgets.BubbleChart{}
        plot_candlestick := mod.widgets.CandlestickChart{}
        plot_heatmap := mod.widgets.HeatmapChart{}
        plot_treemap := mod.widgets.Treemap{}
        plot_sankey := mod.widgets.SankeyDiagram{}
        plot_histogram := mod.widgets.HistogramChart{}
        plot_boxplot := mod.widgets.BoxPlotChart{}
        plot_donut := mod.widgets.DonutChart{}
        plot_stem := mod.widgets.StemPlot{}
        plot_violin := mod.widgets.ViolinPlot{}
        plot_polar := mod.widgets.PolarPlot{}
        plot_contour := mod.widgets.ContourPlot{}
        plot_waterfall := mod.widgets.WaterfallChart{}
        plot_funnel := mod.widgets.FunnelChart{}
        plot_step := mod.widgets.StepPlot{}
        plot_stackplot := mod.widgets.Stackplot{}
        plot_hexbin := mod.widgets.HexbinChart{}
        plot_streamgraph := mod.widgets.Streamgraph{}
        plot_surface3d := mod.widgets.Surface3D{}
        plot_scatter3d := mod.widgets.Scatter3D{}
        plot_line3d := mod.widgets.Line3D{}

        // Audio player button (draw_button/draw_button_text still used by audio player)
        draw_button +: {
            border_radius: uniform(6.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(1.0, 1.0, self.rect_size.x - 2.0, self.rect_size.y - 2.0, self.border_radius)
                sdf.fill(self.color)
                return sdf.result
            }
        }

        draw_button_text +: {
            text_style: theme.font_bold{font_size: 14.0 line_spacing: 1.4}
            color: #FFFFFF
        }

        draw_card_text +: {
            text_style: theme.font_regular{font_size: 14.0 line_spacing: 1.4}
            color: #FFFFFF
        }

        img_headphones: crate_resource("self:resources/headphones.jpg")
        img_mouse: crate_resource("self:resources/mouse.jpg")
        img_keyboard: crate_resource("self:resources/keyboard.jpg")
        img_alipay: crate_resource("self:resources/alipay.png")
        img_wechat: crate_resource("self:resources/wechat.png")
    }
}

// ============================================================================
// A2UI Surface Widget
// ============================================================================

/// The root container for rendering A2UI component trees.
#[derive(Script, ScriptHook, Widget)]
pub struct A2uiSurface {
    #[uid]
    uid: WidgetUid,

    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawColor,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    /// Draw card background (begin/end pattern for Card containers)
    #[redraw]
    #[live]
    draw_card: DrawColor,

    /// Draw image placeholder background
    #[redraw]
    #[live]
    draw_image_placeholder: DrawColor,

    /// Draw text for image placeholder
    #[live]
    draw_image_text: DrawText,

    /// Draw actual image
    #[redraw]
    #[live]
    draw_image: DrawA2uiImage,

    /// Draw chart line segment (chord chart)
    #[redraw]
    #[live]
    draw_chart_line: DrawA2uiChartLine,

    /// Draw chart arc (chord chart)
    #[redraw]
    #[live]
    draw_chart_arc: DrawA2uiArc,

    /// Draw chart text (chord chart labels)
    #[live]
    draw_chart_text: DrawText,

    /// Draw chart arbitrary quadrilateral (chord ribbons)
    #[redraw]
    #[live]
    draw_chart_quad: DrawA2uiQuad,

    /// Draw divider line
    #[redraw]
    #[live]
    draw_divider: DrawColor,

    // makepad-plot chart widget instances
    #[live]
    plot_line: LinePlot,
    #[live]
    plot_bar: BarPlot,
    #[live]
    plot_scatter: ScatterPlot,
    #[live]
    plot_pie: PieChart,
    #[live]
    plot_area: AreaChart,
    #[live]
    plot_radar: RadarChart,
    #[live]
    plot_gauge: GaugeChart,
    #[live]
    plot_bubble: BubbleChart,
    #[live]
    plot_candlestick: CandlestickChart,
    #[live]
    plot_heatmap: HeatmapChart,
    #[live]
    plot_treemap: Treemap,
    #[live]
    plot_sankey: SankeyDiagram,
    #[live]
    plot_histogram: HistogramChart,
    #[live]
    plot_boxplot: BoxPlotChart,
    #[live]
    plot_donut: DonutChart,
    #[live]
    plot_stem: StemPlot,
    #[live]
    plot_violin: ViolinPlot,
    #[live]
    plot_polar: PolarPlot,
    #[live]
    plot_contour: ContourPlot,
    #[live]
    plot_waterfall: WaterfallChart,
    #[live]
    plot_funnel: FunnelChart,
    #[live]
    plot_step: StepPlot,
    #[live]
    plot_stackplot: Stackplot,
    #[live]
    plot_hexbin: HexbinChart,
    #[live]
    plot_streamgraph: Streamgraph,
    #[live]
    plot_surface3d: Surface3D,
    #[live]
    plot_scatter3d: Scatter3D,
    #[live]
    plot_line3d: Line3D,

    /// Draw Aurora shader stage effect
    #[redraw]
    #[live]
    draw_aurora: DrawAurora,

    /// Draw Reef shader stage effect
    #[redraw]
    #[live]
    draw_reef: DrawReef,

    /// Draw Fractal Rainbow shader stage effect
    #[redraw]
    #[live]
    draw_fractal_rainbow: DrawFractalRainbow,

    /// Draw Glowing Lattice shader stage effect
    #[redraw]
    #[live]
    draw_glowing_lattice: DrawGlowingLattice,

    /// Draw Jellyfish shader stage effect
    #[redraw]
    #[live]
    draw_jellyfish: DrawJellyfish,

    /// Draw Turbulence Fire shader stage effect (Xor technique)
    #[redraw]
    #[live]
    draw_turbulence_fire: DrawTurbulenceFire,

    /// Draw audio bars visualization
    #[redraw]
    #[live]
    draw_audio_bars: DrawAudioBars,

    /// Draw Taiji (Yin-Yang) with liquid glass effect
    #[redraw]
    #[live]
    draw_taiji: DrawTaiji,

    /// Taiji rotation animation state (0.0–1.0, wraps)
    #[rust]
    taiji_anim: f32,

    // ============================================================================
    // Widget pool templates (used to clone new pool instances)
    // ============================================================================
    #[live]
    #[live]
    #[live]
    #[live]
    #[live]
    #[live]
    // ============================================================================
    // Widget pools
    // ============================================================================
    /// Pool of MpButton instances
    #[rust]
    mp_buttons: Vec<MpButton>,

    /// Pool of MpCheckbox instances
    #[rust]
    mp_checkboxes: Vec<MpCheckbox>,

    /// Pool of MpSlider instances
    #[rust]
    mp_sliders: Vec<MpSlider>,

    /// Pool of MpLabel instances
    #[rust]
    mp_labels: Vec<MpLabel>,

    /// Pool of Markdown instances (rendered from the MpMarkdown template)
    #[rust]
    mp_markdowns: Vec<Markdown>,

    /// Pool of TextInput instances
    #[rust]
    mp_text_inputs: Vec<TextInput>,

    /// Pool of extended-component instances (gpui parity batch)
    #[rust]
    mp_tags: Vec<MpTag>,
    #[rust]
    mp_step_indicators: Vec<MpStepIndicator>,
    #[rust]
    mp_number_inputs: Vec<MpNumberInput>,
    #[rust]
    mp_searchable_lists: Vec<MpSearchableList>,
    #[rust]
    mp_avatar_groups: Vec<MpAvatarGroup>,
    #[rust]
    mp_color_pickers: Vec<MpColorPicker>,
    #[rust]
    mp_description_lists: Vec<MpDescriptionList>,

    // ============================================================================
    // Pool metadata (maps pool index to A2UI component info)
    // ============================================================================
    /// Button metadata: (component_id, action_def, scope)
    #[rust]
    button_meta: Vec<(String, Option<ActionDefinition>, Option<String>)>,

    /// Checkbox metadata: (component_id, binding_path, checked_value)
    #[rust]
    checkbox_meta: Vec<(String, Option<String>, bool)>,

    /// Slider metadata: (component_id, binding_path, min, max, value)
    #[rust]
    slider_meta: Vec<(String, Option<String>, f64, f64, f64)>,

    /// TextInput metadata: (component_id, binding_path, value)
    #[rust]
    text_input_meta: Vec<(String, Option<String>, String)>,

    /// NumberInput metadata: (component_id, binding_path)
    #[rust]
    number_input_meta: Vec<(String, Option<String>)>,

    /// Frame counters for pools without metadata (reset each frame)
    #[rust]
    tag_count: usize,
    #[rust]
    step_indicator_count: usize,
    #[rust]
    searchable_list_count: usize,
    #[rust]
    avatar_group_count: usize,
    #[rust]
    description_list_count: usize,

    /// ColorPicker metadata: (component_id, binding_path, palette)
    #[rust]
    color_picker_meta: Vec<(String, Option<String>, Vec<Vec4f>)>,

    /// Frame counter for label pool (reset each frame, used as pool index)
    #[rust]
    label_count: usize,

    /// Whether currently rendering inside a Card (for audio player rendering)
    #[rust]
    inside_card: bool,

    // ============================================================================
    // Image sources (preloaded)
    // ============================================================================
    #[live]
    img_headphones: Option<ScriptHandleRef>,
    #[live]
    img_mouse: Option<ScriptHandleRef>,
    #[live]
    img_keyboard: Option<ScriptHandleRef>,
    #[live]
    img_alipay: Option<ScriptHandleRef>,
    #[live]
    img_wechat: Option<ScriptHandleRef>,

    /// Loaded textures for images
    #[rust]
    texture_headphones: Option<Texture>,
    #[rust]
    texture_mouse: Option<Texture>,
    #[rust]
    texture_keyboard: Option<Texture>,
    #[rust]
    texture_alipay: Option<Texture>,
    #[rust]
    texture_wechat: Option<Texture>,

    /// The message processor (manages surfaces and data models)
    #[rust]
    processor: Option<A2uiMessageProcessor>,

    #[rust]
    area: Area,

    /// Current template scope path for relative path resolution
    #[rust]
    current_scope: Option<String>,

    // ============================================================================
    // AudioPlayer state tracking (kept - no MpAudioPlayer widget exists)
    // ============================================================================

    // ============================================================================
    // Calendar widget (lazy-initialized)
    // ============================================================================
    #[rust]
    mp_calendar: Option<MpCalendar>,

    /// Real audio amplitude from native playback (0.0–1.0)
    #[rust]
    audio_amplitude: f32,

    /// NextFrame token for continuous animation (audio bars)
    #[rust]
    next_frame: NextFrame,

    /// AudioPlayer button areas for event detection (play buttons)
    #[rust]
    audio_player_areas: Vec<Area>,

    /// AudioPlayer metadata: (component_id, audio_url, title)
    #[rust]
    audio_player_data: Vec<(String, String, String)>,

    /// Currently hovered audio player index
    #[rust]
    hovered_audio_player_idx: Option<usize>,

    /// Currently playing audio component ID (for Play/Stop toggle)
    #[rust]
    playing_component_id: Option<String>,

    // ============================================================================
    // Audio player still uses draw_button for its play/stop button rendering
    // Keep a DrawColor + DrawText for the audio player button only
    // ============================================================================
    /// Draw button background for audio player only
    #[redraw]
    #[live]
    draw_button: DrawColor,

    /// Draw text for audio player button only
    #[live]
    draw_button_text: DrawText,

    /// Draw text for card content (audio player text inside cards)
    #[live]
    draw_card_text: DrawText,
}

impl A2uiSurface {
    /// Initialize the surface with a processor
    pub fn init_processor(&mut self) {
        if self.processor.is_none() {
            self.processor = Some(A2uiMessageProcessor::with_standard_catalog());
        }
    }

    /// Clear all surfaces and reset the processor
    pub fn clear(&mut self) {
        // Reset the processor to clear all surfaces and components
        self.processor = Some(A2uiMessageProcessor::with_standard_catalog());
    }

    /// Apply theme colors to all A2UI components
    pub fn set_theme_colors(&mut self, _cx: &mut Cx, colors: &A2uiThemeColors) {
        // Apply surface background
        self.draw_bg.color = colors.bg_surface;

        // Apply card colors
        self.draw_card.color = colors.bg_card;

        // Apply divider color
        self.draw_divider.color = colors.border_color;

        // Apply image placeholder text
        self.draw_image_text.color = colors.text_secondary;

        // Apply button color for audio player
        self.draw_button.color = colors.accent;

        self.draw_button_text.color = vec4(1.0, 1.0, 1.0, 1.0);

        self.draw_card_text.color = colors.text_primary;
    }

    /// Load image textures from crate_resource handles
    fn load_image_textures(&mut self, cx: &mut Cx) {
        use makepad_widgets::image_cache::ImageBuffer;

        fn load(cx: &mut Cx, src: &Option<ScriptHandleRef>, jpg: bool) -> Option<Texture> {
            let handle_ref = src.as_ref()?;
            let handle = handle_ref.as_handle();
            let data = if let Some(data) = cx.get_resource(handle) {
                data
            } else {
                cx.load_script_resource(handle);
                cx.get_resource(handle)?
            };
            let image = if jpg {
                ImageBuffer::from_jpg(&data).ok()?
            } else {
                ImageBuffer::from_png(&data).ok()?
            };
            Some(image.into_new_texture(cx))
        }

        if self.texture_headphones.is_none() {
            self.texture_headphones = load(cx, &self.img_headphones, true);
        }
        if self.texture_mouse.is_none() {
            self.texture_mouse = load(cx, &self.img_mouse, true);
        }
        if self.texture_keyboard.is_none() {
            self.texture_keyboard = load(cx, &self.img_keyboard, true);
        }
        if self.texture_alipay.is_none() {
            self.texture_alipay = load(cx, &self.img_alipay, false);
        }
        if self.texture_wechat.is_none() {
            self.texture_wechat = load(cx, &self.img_wechat, false);
        }
    }

    /// Get texture index for a given URL (0=headphones, 1=mouse, 2=keyboard, 3=alipay, 4=wechat, None=not found)
    fn get_texture_index_for_url(&self, url: &str) -> Option<usize> {
        if url.contains("headphones") && self.texture_headphones.is_some() {
            Some(0)
        } else if url.contains("mouse") && self.texture_mouse.is_some() {
            Some(1)
        } else if url.contains("keyboard") && self.texture_keyboard.is_some() {
            Some(2)
        } else if url.contains("alipay") && self.texture_alipay.is_some() {
            Some(3)
        } else if url.contains("wechat") && self.texture_wechat.is_some() {
            Some(4)
        } else {
            None
        }
    }

    /// Get the processor
    pub fn processor(&self) -> Option<&A2uiMessageProcessor> {
        self.processor.as_ref()
    }

    /// Get mutable processor
    pub fn processor_mut(&mut self) -> Option<&mut A2uiMessageProcessor> {
        self.processor.as_mut()
    }

    /// Set the currently playing audio component ID (for Play/Stop toggle display)
    pub fn set_playing_component(&mut self, component_id: Option<String>) {
        self.playing_component_id = component_id;
    }

    /// Get the currently playing audio component ID
    pub fn playing_component_id(&self) -> Option<&String> {
        self.playing_component_id.as_ref()
    }

    /// Set the real audio amplitude for visualization (0.0–1.0, from native audio playback)
    pub fn set_audio_amplitude(&mut self, amplitude: f32) {
        self.audio_amplitude = amplitude;
    }

    /// Collect all AudioPlayer URLs from the component tree.
    /// Returns (title, url) pairs for pre-downloading.
    pub fn collect_audio_urls(&self) -> Vec<(String, String)> {
        if let Some(processor) = &self.processor {
            processor.collect_audio_urls()
        } else {
            vec![]
        }
    }

    /// Process A2UI JSON messages
    pub fn process_json(&mut self, json: &str) -> Result<Vec<ProcessorEvent>, serde_json::Error> {
        self.init_processor();
        if let Some(processor) = self.processor.as_mut() {
            processor.process_json(json)
        } else {
            Ok(vec![])
        }
    }

    /// Process a single A2UI message
    pub fn process_message(&mut self, message: A2uiMessage) -> Vec<ProcessorEvent> {
        self.init_processor();
        if let Some(processor) = self.processor.as_mut() {
            processor.process_message(message)
        } else {
            vec![]
        }
    }

    /// Get the current surface ID
    fn get_surface_id(&self) -> String {
        // For now, use "main" as default
        "main".to_string()
    }

    /// Instantiate a widget of type T from its registered object in mod.widgets
    fn new_from_mod<T: ScriptNew>(cx: &mut Cx, id: LiveId) -> T {
        cx.with_vm(|vm| {
            let widgets = vm.module(id!(widgets));
            let value = vm.bx.heap.value(widgets, id.into(), NoTrap);
            T::script_from_value(vm, value)
        })
    }

    /// Get or lazily create the MpCalendar instance
    fn ensure_calendar(&mut self, cx: &mut Cx) -> &mut MpCalendar {
        if self.mp_calendar.is_none() {
            self.mp_calendar = Some(cx.with_vm(MpCalendar::script_new_with_default));
        }
        self.mp_calendar.as_mut().unwrap()
    }

    /// Get or grow a button from the pool
    fn pool_button(&mut self, cx: &mut Cx, idx: usize) -> &mut MpButton {
        while self.mp_buttons.len() <= idx {
            let new_btn = cx.with_vm(MpButton::script_new_with_default);
            self.mp_buttons.push(new_btn);
        }
        &mut self.mp_buttons[idx]
    }

    /// Get or grow a checkbox from the pool
    fn pool_checkbox(&mut self, cx: &mut Cx, idx: usize) -> &mut MpCheckbox {
        while self.mp_checkboxes.len() <= idx {
            let mut new_cb = cx.with_vm(MpCheckbox::script_new_with_default);
            // Override label color for dark bg
            script_apply_eval!(cx, new_cb, { draw_label +: { color: #E0E0E0 } });
            self.mp_checkboxes.push(new_cb);
        }
        &mut self.mp_checkboxes[idx]
    }

    /// Get or grow a slider from the pool
    fn pool_slider(&mut self, cx: &mut Cx, idx: usize) -> &mut MpSlider {
        while self.mp_sliders.len() <= idx {
            let mut new_sl = cx.with_vm(MpSlider::script_new_with_default);
            script_apply_eval!(cx, new_sl, { width: 200 });
            self.mp_sliders.push(new_sl);
        }
        &mut self.mp_sliders[idx]
    }

    /// Get or grow a label from the pool
    fn pool_label(&mut self, cx: &mut Cx, idx: usize) -> &mut MpLabel {
        while self.mp_labels.len() <= idx {
            let mut new_lb = cx.with_vm(MpLabel::script_new_with_default);
            script_apply_eval!(cx, new_lb, { draw_text +: { color: #E0E0E0 } });
            self.mp_labels.push(new_lb);
        }
        &mut self.mp_labels[idx]
    }

    /// Get or grow a markdown renderer from the pool (MpMarkdown template)
    fn pool_markdown(&mut self, cx: &mut Cx, idx: usize) -> &mut Markdown {
        while self.mp_markdowns.len() <= idx {
            let new_md = Self::new_from_mod::<Markdown>(cx, id!(MpMarkdown));
            self.mp_markdowns.push(new_md);
        }
        &mut self.mp_markdowns[idx]
    }

    /// Get or grow a text input from the pool
    fn pool_text_input(&mut self, cx: &mut Cx, idx: usize) -> &mut TextInput {
        while self.mp_text_inputs.len() <= idx {
            let new_ti = Self::new_from_mod::<TextInput>(cx, id!(A2uiTextInput));
            self.mp_text_inputs.push(new_ti);
        }
        &mut self.mp_text_inputs[idx]
    }

    fn pool_tag(&mut self, cx: &mut Cx, idx: usize) -> &mut MpTag {
        while self.mp_tags.len() <= idx {
            let new_tag = cx.with_vm(MpTag::script_new_with_default);
            self.mp_tags.push(new_tag);
        }
        &mut self.mp_tags[idx]
    }

    fn pool_step_indicator(&mut self, cx: &mut Cx, idx: usize) -> &mut MpStepIndicator {
        while self.mp_step_indicators.len() <= idx {
            let new_si = cx.with_vm(MpStepIndicator::script_new_with_default);
            self.mp_step_indicators.push(new_si);
        }
        &mut self.mp_step_indicators[idx]
    }

    fn pool_number_input(&mut self, cx: &mut Cx, idx: usize) -> &mut MpNumberInput {
        while self.mp_number_inputs.len() <= idx {
            let new_ni = cx.with_vm(MpNumberInput::script_new_with_default);
            self.mp_number_inputs.push(new_ni);
        }
        &mut self.mp_number_inputs[idx]
    }

    fn pool_searchable_list(&mut self, cx: &mut Cx, idx: usize) -> &mut MpSearchableList {
        while self.mp_searchable_lists.len() <= idx {
            let new_sl = cx.with_vm(MpSearchableList::script_new_with_default);
            self.mp_searchable_lists.push(new_sl);
        }
        &mut self.mp_searchable_lists[idx]
    }

    fn pool_avatar_group(&mut self, cx: &mut Cx, idx: usize) -> &mut MpAvatarGroup {
        while self.mp_avatar_groups.len() <= idx {
            let new_ag = cx.with_vm(MpAvatarGroup::script_new_with_default);
            self.mp_avatar_groups.push(new_ag);
        }
        &mut self.mp_avatar_groups[idx]
    }

    fn pool_color_picker(&mut self, cx: &mut Cx, idx: usize) -> &mut MpColorPicker {
        while self.mp_color_pickers.len() <= idx {
            let new_cp = cx.with_vm(MpColorPicker::script_new_with_default);
            self.mp_color_pickers.push(new_cp);
        }
        &mut self.mp_color_pickers[idx]
    }

    fn pool_description_list(&mut self, cx: &mut Cx, idx: usize) -> &mut MpDescriptionList {
        while self.mp_description_lists.len() <= idx {
            let new_dl = cx.with_vm(MpDescriptionList::script_new_with_default);
            self.mp_description_lists.push(new_dl);
        }
        &mut self.mp_description_lists[idx]
    }
}

// Widget trait implementation (handle_event + draw_walk)
include!("events_impl.rs");

// Render methods - layout and basic components
include!("render_impl.rs");

// Render methods - charts, chord, audio player
include!("render_charts_impl.rs");

// Render methods - calendar grid
include!("render_calendar_impl.rs");

// Render methods - shader stage effects
include!("render_shader_stage_impl.rs");

impl A2uiSurfaceRef {
    /// Process A2UI JSON messages
    pub fn process_json(&self, json: &str) -> Result<Vec<ProcessorEvent>, serde_json::Error> {
        if let Some(mut inner) = self.borrow_mut() {
            inner.process_json(json)
        } else {
            Ok(vec![])
        }
    }

    /// Process a single A2UI message
    pub fn process_message(&self, message: A2uiMessage) -> Vec<ProcessorEvent> {
        if let Some(mut inner) = self.borrow_mut() {
            inner.process_message(message)
        } else {
            vec![]
        }
    }

    /// Check if any user action was triggered
    /// Returns the UserAction if one was triggered
    pub fn user_action(&self, actions: &Actions) -> Option<UserAction> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let A2uiSurfaceAction::UserAction(user_action) =
                    action.cast::<A2uiSurfaceAction>()
                {
                    return Some(user_action);
                }
            }
        }
        None
    }

    /// Check if a specific action was triggered by name
    /// Returns the context HashMap if the action matches
    pub fn action_by_name(
        &self,
        actions: &Actions,
        action_name: &str,
    ) -> Option<std::collections::HashMap<String, serde_json::Value>> {
        if let Some(user_action) = self.user_action(actions) {
            if user_action.action.name == action_name {
                return Some(user_action.action.context);
            }
        }
        None
    }

    /// Check if an audio play action was triggered
    /// Returns (component_id, url, title) if PlayAudio was triggered
    pub fn play_audio(&self, actions: &Actions) -> Option<(String, String, String)> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let A2uiSurfaceAction::PlayAudio {
                    component_id,
                    url,
                    title,
                } = action.cast::<A2uiSurfaceAction>()
                {
                    return Some((component_id, url, title));
                }
            }
        }
        None
    }

    /// Set the currently playing audio component ID (for Play/Stop toggle display)
    pub fn set_playing_component(&self, component_id: Option<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_playing_component(component_id);
        }
    }

    /// Set the real audio amplitude for visualization (0.0–1.0)
    pub fn set_audio_amplitude(&self, amplitude: f32) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_audio_amplitude(amplitude);
        }
    }

    /// Collect all AudioPlayer URLs from the component tree for pre-downloading.
    pub fn collect_audio_urls(&self) -> Vec<(String, String)> {
        if let Some(inner) = self.borrow() {
            inner.collect_audio_urls()
        } else {
            vec![]
        }
    }
}
