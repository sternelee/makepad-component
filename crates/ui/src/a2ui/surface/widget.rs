//! A2uiSurface widget definition and core implementation

use makepad_widgets::*;
use makepad_plot::*;

use crate::a2ui::{
    chart_bridge,
    data_model::DataModel,
    message::*,
    processor::{
        resolve_boolean_value_scoped, resolve_number_value_scoped,
        resolve_string_value_scoped, A2uiMessageProcessor, ProcessorEvent,
    },
};
use crate::widgets::{
    button::MpButton,
    calendar::MpCalendar,
    checkbox::{MpCheckbox, MpCheckboxAction},
    slider::{MpSlider, MpSliderAction},
    switch::{MpSwitch, MpSwitchAction},
    label::MpLabel,
};

use super::draw_types::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use makepad_plot::plot::line::LinePlot;
    use makepad_plot::plot::bar::BarPlot;
    use makepad_plot::plot::scatter::ScatterPlot;
    use makepad_plot::plot::pie::PieChart;
    use makepad_plot::plot::area::AreaChart;
    use makepad_plot::plot::polar::RadarChart;
    use makepad_plot::plot::gauge::GaugeChart;
    use makepad_plot::plot::bubble::BubbleChart;
    use makepad_plot::plot::financial::CandlestickChart;
    use makepad_plot::plot::heatmap::HeatmapChart;
    use makepad_plot::plot::treemap::Treemap;
    use makepad_plot::plot::hexbin::SankeyDiagram;
    use makepad_plot::plot::histogram::HistogramChart;
    use makepad_plot::plot::histogram::BoxPlotChart;
    use makepad_plot::plot::pie::DonutChart;
    use makepad_plot::plot::stem::StemPlot;
    use makepad_plot::plot::stem::ViolinPlot;
    use makepad_plot::plot::polar::PolarPlot;
    use makepad_plot::plot::contour::ContourPlot;
    use makepad_plot::plot::financial::WaterfallChart;
    use makepad_plot::plot::gauge::FunnelChart;
    use makepad_plot::plot::area::StepPlot;
    use makepad_plot::plot::stack::Stackplot;
    use makepad_plot::plot::hexbin::HexbinChart;
    use makepad_plot::plot::stack::Streamgraph;
    use makepad_plot::plot::surface3d::Surface3D;
    use makepad_plot::plot::scatter3d::Scatter3D;
    use makepad_plot::plot::scatter3d::Line3D;

    use crate::theme::colors::*;

    use crate::a2ui::surface::draw_types::DrawA2uiImage;
    use crate::a2ui::surface::draw_types::DrawA2uiChartLine;
    use crate::a2ui::surface::draw_types::DrawA2uiArc;
    use crate::a2ui::surface::draw_types::DrawA2uiQuad;
    use crate::a2ui::surface::draw_types::DrawAudioBars;
    use crate::a2ui::surface::draw_types::DrawTaiji;
    use crate::a2ui::surface::draw_types::DrawAurora;
    use crate::a2ui::surface::draw_types::DrawReef;
    use crate::a2ui::surface::draw_types::DrawFractalRainbow;
    use crate::a2ui::surface::draw_types::DrawGlowingLattice;
    use crate::a2ui::surface::draw_types::DrawJellyfish;
    use crate::a2ui::surface::draw_types::DrawTurbulenceFire;

    use crate::widgets::calendar::MpCalendar;

    // Widget templates for pool cloning
    use crate::widgets::button::MpButton;
    use crate::widgets::checkbox::MpCheckbox;
    use crate::widgets::slider::MpSlider;
    use crate::widgets::switch::MpSwitch;
    use crate::widgets::label::MpLabel;

    pub A2uiSurface = {{A2uiSurface}} {
        width: Fill
        height: Fill
        flow: Down

        draw_bg: {
            instance bg_color: #1a1a2e

            fn pixel(self) -> vec4 {
                return self.bg_color;
            }
        }

        // Card background (DrawColor begin/end pattern for Card containers)
        draw_card: {
            color: #2a3a5a
            instance border_color: #5588bb
            instance border_radius: 8.0
            instance border_width: 1.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    max(1.0, self.border_radius)
                );
                sdf.fill_keep(self.color);
                sdf.stroke(self.border_color, self.border_width);
                return sdf.result;
            }
        }

        draw_image_placeholder: {
            instance border_radius: 4.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(1.0, 1.0, self.rect_size.x - 2.0, self.rect_size.y - 2.0, self.border_radius);
                let stripe_width = 8.0;
                let pos = self.pos * self.rect_size;
                let stripe = mod(pos.x + pos.y, stripe_width * 2.0);
                let is_stripe = step(stripe_width, stripe);
                let color1 = vec4(0.25, 0.28, 0.35, 1.0);
                let color2 = vec4(0.30, 0.33, 0.40, 1.0);
                let bg_color = mix(color1, color2, is_stripe);
                sdf.fill(bg_color);
                return sdf.result;
            }
        }

        draw_image_text: {
            text_style: <THEME_FONT_REGULAR> {
                font_size: 11.0
            }
            color: #888888
        }

        draw_image: <DrawA2uiImage> {}

        draw_chart_line: <DrawA2uiChartLine> {}
        draw_chart_arc: <DrawA2uiArc> {}
        draw_chart_text: {
            text_style: <THEME_FONT_REGULAR> {
                font_size: 10.0
            }
            color: #AABBCC
        }
        draw_chart_quad: <DrawA2uiQuad> {}

        // Divider draw
        draw_divider: {
            color: #5588bb

            fn pixel(self) -> vec4 {
                return self.color;
            }
        }

        // Calendar widget template
        tpl_calendar: <MpCalendar> {}

        plot_line: <LinePlot> {}
        plot_bar: <BarPlot> {}
        plot_scatter: <ScatterPlot> {}
        plot_pie: <PieChart> {}
        plot_area: <AreaChart> {}
        plot_radar: <RadarChart> {}
        plot_gauge: <GaugeChart> {}
        plot_bubble: <BubbleChart> {}
        plot_candlestick: <CandlestickChart> {}
        plot_heatmap: <HeatmapChart> {}
        plot_treemap: <Treemap> {}
        plot_sankey: <SankeyDiagram> {}
        plot_histogram: <HistogramChart> {}
        plot_boxplot: <BoxPlotChart> {}
        plot_donut: <DonutChart> {}
        plot_stem: <StemPlot> {}
        plot_violin: <ViolinPlot> {}
        plot_polar: <PolarPlot> {}
        plot_contour: <ContourPlot> {}
        plot_waterfall: <WaterfallChart> {}
        plot_funnel: <FunnelChart> {}
        plot_step: <StepPlot> {}
        plot_stackplot: <Stackplot> {}
        plot_hexbin: <HexbinChart> {}
        plot_streamgraph: <Streamgraph> {}
        plot_surface3d: <Surface3D> {}
        plot_scatter3d: <Scatter3D> {}
        plot_line3d: <Line3D> {}
        draw_aurora: <DrawAurora> {}
        draw_reef: <DrawReef> {}
        draw_fractal_rainbow: <DrawFractalRainbow> {}
        draw_glowing_lattice: <DrawGlowingLattice> {}
        draw_jellyfish: <DrawJellyfish> {}
        draw_turbulence_fire: <DrawTurbulenceFire> {}

        draw_audio_bars: <DrawAudioBars> {}
        draw_taiji: <DrawTaiji> {}

        // Audio player button (draw_button/draw_button_text still used by audio player)
        draw_button: {
            instance border_radius: 6.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(1.0, 1.0, self.rect_size.x - 2.0, self.rect_size.y - 2.0, self.border_radius);
                sdf.fill(self.color);
                return sdf.result;
            }
        }

        draw_button_text: {
            text_style: <THEME_FONT_BOLD> {
                font_size: 14.0
                line_spacing: 1.4
            }
            color: #FFFFFF
        }

        draw_card_text: {
            text_style: <THEME_FONT_REGULAR> {
                font_size: 14.0
                line_spacing: 1.4
            }
            color: #FFFFFF
        }

        // Widget templates for pool cloning
        // Override text colors for dark A2UI background (#1a1a2e / #2a3a5a)
        tpl_button: <MpButton> {}
        tpl_checkbox: <MpCheckbox> {
            // Override label color for dark bg
            draw_label: { color: #E0E0E0 }
        }
        tpl_slider: <MpSlider> { width: 200 }
        tpl_label: <MpLabel> {
            draw_text: {
                color: #E0E0E0
            }
        }
        tpl_text_input: <TextInput> {
            width: 200
            height: Fit
            padding: { left: 12, right: 12, top: 8, bottom: 8 }
            empty_text: ""

            draw_bg: {
                instance hover: 0.0
                instance focus: 0.0

                uniform border_radius: 6.0
                uniform border_width: 1.0
                uniform bg_color: #2a3a5a
                uniform border_color: #5588bb
                uniform border_color_focus: #3B82F6

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(
                        self.border_width,
                        self.border_width,
                        self.rect_size.x - self.border_width * 2.0,
                        self.rect_size.y - self.border_width * 2.0,
                        self.border_radius
                    );
                    sdf.fill_keep(self.bg_color);
                    let border = mix(self.border_color, self.border_color_focus, self.focus);
                    sdf.stroke(border, self.border_width);
                    return sdf.result;
                }
            }

            draw_text: {
                text_style: <THEME_FONT_REGULAR> { font_size: 14.0 }
                fn get_color(self) -> vec4 {
                    return mix(#FFFFFF, #888888, self.empty);
                }
            }

            draw_cursor: {
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 1.0);
                    sdf.fill(mix(#0000, #3B82F6, self.focus * (1.0 - self.blink)));
                    return sdf.result;
                }
            }

            draw_selection: {
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 2.0);
                    sdf.fill(#3B82F620);
                    return sdf.result;
                }
            }

            animator: {
                hover = {
                    default: off,
                    off = {
                        from: {all: Forward {duration: 0.15}}
                        apply: { draw_bg: {hover: 0.0} }
                    }
                    on = {
                        from: {all: Forward {duration: 0.1}}
                        apply: { draw_bg: {hover: 1.0} }
                    }
                }
                focus = {
                    default: off,
                    off = {
                        from: {all: Forward {duration: 0.2}}
                        apply: { draw_bg: {focus: 0.0}, draw_cursor: {focus: 0.0} }
                    }
                    on = {
                        from: {all: Snap}
                        apply: { draw_bg: {focus: 1.0}, draw_cursor: {focus: 1.0} }
                    }
                }
            }
        }

        img_headphones: dep("crate://self/resources/headphones.jpg")
        img_mouse: dep("crate://self/resources/mouse.jpg")
        img_keyboard: dep("crate://self/resources/keyboard.jpg")
        img_alipay: dep("crate://self/resources/alipay.png")
        img_wechat: dep("crate://self/resources/wechat.png")

        // ============================================================
        // shadcn UI draw primitives (Splash script layer)
        // ============================================================

        // Rounded-rect background used for Badge, Alert, Notification, Skeleton.
        // The default color and border_radius are always overridden at runtime
        // via apply_over() before each draw call, so these defaults are never
        // directly visible — they exist only to satisfy the Live type system.
        draw_badge: {
            color: #3b82f6
            instance border_radius: 12.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(
                    0.0, 0.0,
                    self.rect_size.x, self.rect_size.y,
                    max(1.0, self.border_radius)
                );
                sdf.fill(self.color);
                return sdf.result;
            }
        }

        // Filled circle used for Avatar background
        draw_circle: {
            color: #6366f1

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let c = self.rect_size * 0.5;
                let r = min(c.x, c.y) - 1.0;
                sdf.circle(c.x, c.y, r);
                sdf.fill(self.color);
                return sdf.result;
            }
        }

        // Progress bar capsule (track + fill driven by `progress` instance uniform)
        draw_progress: {
            color: #3b82f6
            instance track_color: #374151
            instance progress: 0.5

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let sz = self.rect_size;
                let r = sz.y * 0.5;

                // Track capsule
                sdf.circle(r, r, r);
                sdf.rect(r, 0.0, sz.x - sz.y, sz.y);
                sdf.circle(sz.x - r, r, r);
                sdf.fill(self.track_color);

                // Fill capsule (overlay)
                let fill_end = sz.x * self.progress;
                let px = self.pos.x * sz.x;
                let in_fill = step(px, fill_end);

                let sdf2 = Sdf2d::viewport(self.pos * self.rect_size);
                sdf2.circle(r, r, r);
                sdf2.rect(r, 0.0, sz.x - sz.y, sz.y);
                sdf2.circle(sz.x - r, r, r);
                sdf2.fill(self.color);

                return mix(sdf.result, sdf2.result, in_fill * sdf2.result.w);
            }
        }

        // Small text for badges, alert descriptions, etc.
        draw_comp_text: {
            text_style: <THEME_FONT_REGULAR> { font_size: 12.0 }
            color: #FFFFFF
        }

        // Bold text for alert titles, section headings
        draw_comp_title: {
            text_style: <THEME_FONT_BOLD> { font_size: 13.0 }
            color: #FFFFFF
        }

        // Switch template for interactive toggle pool
        tpl_switch: <MpSwitch> {}
    }
}

// ============================================================================
// A2UI Surface Widget
// ============================================================================

/// The root container for rendering A2UI component trees.
#[derive(Live, LiveHook, Widget)]
pub struct A2uiSurface {
    #[redraw]
    #[live]
    draw_bg: DrawQuad,

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
    #[live] plot_line: LinePlot,
    #[live] plot_bar: BarPlot,
    #[live] plot_scatter: ScatterPlot,
    #[live] plot_pie: PieChart,
    #[live] plot_area: AreaChart,
    #[live] plot_radar: RadarChart,
    #[live] plot_gauge: GaugeChart,
    #[live] plot_bubble: BubbleChart,
    #[live] plot_candlestick: CandlestickChart,
    #[live] plot_heatmap: HeatmapChart,
    #[live] plot_treemap: Treemap,
    #[live] plot_sankey: SankeyDiagram,
    #[live] plot_histogram: HistogramChart,
    #[live] plot_boxplot: BoxPlotChart,
    #[live] plot_donut: DonutChart,
    #[live] plot_stem: StemPlot,
    #[live] plot_violin: ViolinPlot,
    #[live] plot_polar: PolarPlot,
    #[live] plot_contour: ContourPlot,
    #[live] plot_waterfall: WaterfallChart,
    #[live] plot_funnel: FunnelChart,
    #[live] plot_step: StepPlot,
    #[live] plot_stackplot: Stackplot,
    #[live] plot_hexbin: HexbinChart,
    #[live] plot_streamgraph: Streamgraph,
    #[live] plot_surface3d: Surface3D,
    #[live] plot_scatter3d: Scatter3D,
    #[live] plot_line3d: Line3D,

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

    #[live] tpl_button: Option<LivePtr>,
    #[live] tpl_checkbox: Option<LivePtr>,
    #[live] tpl_slider: Option<LivePtr>,
    #[live] tpl_label: Option<LivePtr>,
    #[live] tpl_text_input: Option<LivePtr>,
    #[live] tpl_calendar: Option<LivePtr>,
    #[live] tpl_switch: Option<LivePtr>,

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

    /// Pool of TextInput instances
    #[rust]
    mp_text_inputs: Vec<TextInput>,

    /// Pool of MpSwitch instances (shadcn Switch component)
    #[rust]
    mp_switches: Vec<MpSwitch>,

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

    /// Switch metadata: (component_id, binding_path, current_value, action_def)
    #[rust]
    switch_meta: Vec<(String, Option<String>, bool, Option<ActionDefinition>)>,

    /// Frame counter for label pool (reset each frame, used as pool index)
    #[rust]
    label_count: usize,

    /// Whether currently rendering inside a Card (for audio player rendering)
    #[rust]
    inside_card: bool,

    // ============================================================================
    // shadcn UI draw primitives (DrawColor with custom shaders)
    // ============================================================================

    /// Rounded-rect background for Badge, Alert, Notification, Skeleton, Accordion header
    #[redraw]
    #[live]
    draw_badge: DrawColor,

    /// Filled circle for Avatar background
    #[redraw]
    #[live]
    draw_circle: DrawColor,

    /// Capsule progress bar (track + fill via `progress` instance uniform)
    #[redraw]
    #[live]
    draw_progress: DrawColor,

    /// Small text for shadcn component labels
    #[live]
    draw_comp_text: DrawText,

    /// Bold text for shadcn component titles / section headers
    #[live]
    draw_comp_title: DrawText,

    // ============================================================================
    // Image sources (preloaded)
    // ============================================================================

    #[live]
    img_headphones: LiveDependency,
    #[live]
    img_mouse: LiveDependency,
    #[live]
    img_keyboard: LiveDependency,
    #[live]
    img_alipay: LiveDependency,
    #[live]
    img_wechat: LiveDependency,

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

    /// Surface ID
    #[live]
    surface_id: LiveValue,

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
    pub fn set_theme_colors(&mut self, cx: &mut Cx, colors: &A2uiThemeColors) {
        // Apply surface background
        self.draw_bg.apply_over(cx, live! {
            bg_color: (colors.bg_surface)
        });

        // Apply card colors
        self.draw_card.apply_over(cx, live! {
            color: (colors.bg_card)
            border_color: (colors.border_color)
        });

        // Apply divider color
        self.draw_divider.apply_over(cx, live! {
            color: (colors.border_color)
        });

        // Apply image placeholder text
        self.draw_image_text.apply_over(cx, live! {
            color: (colors.text_secondary)
        });

        // Apply button color for audio player
        self.draw_button.apply_over(cx, live! {
            color: (colors.accent)
        });

        self.draw_button_text.apply_over(cx, live! {
            color: (vec4(1.0, 1.0, 1.0, 1.0))
        });

        self.draw_card_text.apply_over(cx, live! {
            color: (colors.text_primary)
        });
    }

    /// Load image textures from LiveDependency resources
    fn load_image_textures(&mut self, cx: &mut Cx) {
        use makepad_widgets::image_cache::ImageBuffer;

        // Load headphones image (JPG)
        if self.texture_headphones.is_none() {
            let path = self.img_headphones.as_str();
            if !path.is_empty() {
                if let Ok(data) = cx.get_dependency(path) {
                    if let Ok(image) = ImageBuffer::from_jpg(&data) {
                        self.texture_headphones = Some(image.into_new_texture(cx));
                    }
                }
            }
        }

        // Load mouse image (JPG)
        if self.texture_mouse.is_none() {
            let path = self.img_mouse.as_str();
            if !path.is_empty() {
                if let Ok(data) = cx.get_dependency(path) {
                    if let Ok(image) = ImageBuffer::from_jpg(&data) {
                        self.texture_mouse = Some(image.into_new_texture(cx));
                    }
                }
            }
        }

        // Load keyboard image (JPG)
        if self.texture_keyboard.is_none() {
            let path = self.img_keyboard.as_str();
            if !path.is_empty() {
                if let Ok(data) = cx.get_dependency(path) {
                    if let Ok(image) = ImageBuffer::from_jpg(&data) {
                        self.texture_keyboard = Some(image.into_new_texture(cx));
                    }
                }
            }
        }

        // Load Alipay icon (PNG)
        if self.texture_alipay.is_none() {
            let path = self.img_alipay.as_str();
            if !path.is_empty() {
                if let Ok(data) = cx.get_dependency(path) {
                    if let Ok(image) = ImageBuffer::from_png(&data) {
                        self.texture_alipay = Some(image.into_new_texture(cx));
                    }
                }
            }
        }

        // Load WeChat icon (PNG)
        if self.texture_wechat.is_none() {
            let path = self.img_wechat.as_str();
            if !path.is_empty() {
                if let Ok(data) = cx.get_dependency(path) {
                    if let Ok(image) = ImageBuffer::from_png(&data) {
                        self.texture_wechat = Some(image.into_new_texture(cx));
                    }
                }
            }
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

    /// Get or lazily create the MpCalendar instance
    fn ensure_calendar(&mut self, cx: &mut Cx) -> &mut MpCalendar {
        if self.mp_calendar.is_none() {
            self.mp_calendar = Some(MpCalendar::new_from_ptr(cx, self.tpl_calendar));
        }
        self.mp_calendar.as_mut().unwrap()
    }

    /// Get or grow a button from the pool
    fn pool_button(&mut self, cx: &mut Cx, idx: usize) -> &mut MpButton {
        while self.mp_buttons.len() <= idx {
            let new_btn = MpButton::new_from_ptr(cx, self.tpl_button);
            self.mp_buttons.push(new_btn);
        }
        &mut self.mp_buttons[idx]
    }

    /// Get or grow a checkbox from the pool
    fn pool_checkbox(&mut self, cx: &mut Cx, idx: usize) -> &mut MpCheckbox {
        while self.mp_checkboxes.len() <= idx {
            let new_cb = MpCheckbox::new_from_ptr(cx, self.tpl_checkbox);
            self.mp_checkboxes.push(new_cb);
        }
        &mut self.mp_checkboxes[idx]
    }

    /// Get or grow a slider from the pool
    fn pool_slider(&mut self, cx: &mut Cx, idx: usize) -> &mut MpSlider {
        while self.mp_sliders.len() <= idx {
            let new_sl = MpSlider::new_from_ptr(cx, self.tpl_slider);
            self.mp_sliders.push(new_sl);
        }
        &mut self.mp_sliders[idx]
    }

    /// Get or grow a label from the pool
    fn pool_label(&mut self, cx: &mut Cx, idx: usize) -> &mut MpLabel {
        while self.mp_labels.len() <= idx {
            let new_lb = MpLabel::new_from_ptr(cx, self.tpl_label);
            self.mp_labels.push(new_lb);
        }
        &mut self.mp_labels[idx]
    }

    /// Get or grow a text input from the pool
    fn pool_text_input(&mut self, cx: &mut Cx, idx: usize) -> &mut TextInput {
        while self.mp_text_inputs.len() <= idx {
            let new_ti = TextInput::new_from_ptr(cx, self.tpl_text_input);
            self.mp_text_inputs.push(new_ti);
        }
        &mut self.mp_text_inputs[idx]
    }

    /// Get or grow a switch from the pool
    fn pool_switch(&mut self, cx: &mut Cx, idx: usize) -> &mut MpSwitch {
        while self.mp_switches.len() <= idx {
            let new_sw = MpSwitch::new_from_ptr(cx, self.tpl_switch);
            self.mp_switches.push(new_sw);
        }
        &mut self.mp_switches[idx]
    }
}

// Widget trait implementation (handle_event + draw_walk)
include!("events_impl.rs");

// Render methods - layout and basic components
include!("render_impl.rs");

// Render methods - shadcn UI components (Badge, Alert, Avatar, Progress, etc.)
include!("render_shadcn_impl.rs");

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
                if let A2uiSurfaceAction::PlayAudio { component_id, url, title } =
                    action.cast::<A2uiSurfaceAction>()
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
