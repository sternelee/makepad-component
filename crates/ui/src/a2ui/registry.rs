//! A2UI Component Registry
//!
//! Maps A2UI component types to Makepad widget types.

use std::collections::HashMap;

/// Component type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum A2uiComponentType {
    // Layout
    Column,
    Row,
    List,
    Card,

    // Display
    Text,
    Image,
    Icon,
    Divider,

    // Interactive
    Button,
    TextField,
    CheckBox,
    Slider,
    MultipleChoice,

    // Container
    Modal,
    Tabs,

    // Visualization
    Chart,
    Calendar,
    // Media
    AudioPlayer,
    // Shader visualization
    ShaderStage,

    // shadcn UI components (Splash script layer)
    Badge,
    Alert,
    Avatar,
    Progress,
    Spinner,
    Switch,
    Accordion,
    Notification,
    Skeleton,
}

impl A2uiComponentType {
    /// Get the A2UI component type name
    pub fn name(&self) -> &'static str {
        match self {
            A2uiComponentType::Column => "Column",
            A2uiComponentType::Row => "Row",
            A2uiComponentType::List => "List",
            A2uiComponentType::Card => "Card",
            A2uiComponentType::Text => "Text",
            A2uiComponentType::Image => "Image",
            A2uiComponentType::Icon => "Icon",
            A2uiComponentType::Divider => "Divider",
            A2uiComponentType::Button => "Button",
            A2uiComponentType::TextField => "TextField",
            A2uiComponentType::CheckBox => "CheckBox",
            A2uiComponentType::Slider => "Slider",
            A2uiComponentType::MultipleChoice => "MultipleChoice",
            A2uiComponentType::Modal => "Modal",
            A2uiComponentType::Tabs => "Tabs",
            A2uiComponentType::Chart => "Chart",
            A2uiComponentType::Calendar => "Calendar",
            A2uiComponentType::AudioPlayer => "AudioPlayer",
            A2uiComponentType::ShaderStage => "ShaderStage",
            A2uiComponentType::Badge => "Badge",
            A2uiComponentType::Alert => "Alert",
            A2uiComponentType::Avatar => "Avatar",
            A2uiComponentType::Progress => "Progress",
            A2uiComponentType::Spinner => "Spinner",
            A2uiComponentType::Switch => "Switch",
            A2uiComponentType::Accordion => "Accordion",
            A2uiComponentType::Notification => "Notification",
            A2uiComponentType::Skeleton => "Skeleton",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Column" => Some(A2uiComponentType::Column),
            "Row" => Some(A2uiComponentType::Row),
            "List" => Some(A2uiComponentType::List),
            "Card" => Some(A2uiComponentType::Card),
            "Text" => Some(A2uiComponentType::Text),
            "Image" => Some(A2uiComponentType::Image),
            "Icon" => Some(A2uiComponentType::Icon),
            "Divider" => Some(A2uiComponentType::Divider),
            "Button" => Some(A2uiComponentType::Button),
            "TextField" => Some(A2uiComponentType::TextField),
            "CheckBox" => Some(A2uiComponentType::CheckBox),
            "Slider" => Some(A2uiComponentType::Slider),
            "MultipleChoice" => Some(A2uiComponentType::MultipleChoice),
            "Modal" => Some(A2uiComponentType::Modal),
            "Tabs" => Some(A2uiComponentType::Tabs),
            "Chart" => Some(A2uiComponentType::Chart),
            "Calendar" => Some(A2uiComponentType::Calendar),
            "AudioPlayer" => Some(A2uiComponentType::AudioPlayer),
            "ShaderStage" => Some(A2uiComponentType::ShaderStage),
            "Badge" => Some(A2uiComponentType::Badge),
            "Alert" => Some(A2uiComponentType::Alert),
            "Avatar" => Some(A2uiComponentType::Avatar),
            "Progress" => Some(A2uiComponentType::Progress),
            "Spinner" => Some(A2uiComponentType::Spinner),
            "Switch" => Some(A2uiComponentType::Switch),
            "Accordion" => Some(A2uiComponentType::Accordion),
            "Notification" => Some(A2uiComponentType::Notification),
            "Skeleton" => Some(A2uiComponentType::Skeleton),
            _ => None,
        }
    }

    /// Get all component types
    pub fn all() -> &'static [A2uiComponentType] {
        &[
            A2uiComponentType::Column,
            A2uiComponentType::Row,
            A2uiComponentType::List,
            A2uiComponentType::Card,
            A2uiComponentType::Text,
            A2uiComponentType::Image,
            A2uiComponentType::Icon,
            A2uiComponentType::Divider,
            A2uiComponentType::Button,
            A2uiComponentType::TextField,
            A2uiComponentType::CheckBox,
            A2uiComponentType::Slider,
            A2uiComponentType::MultipleChoice,
            A2uiComponentType::Modal,
            A2uiComponentType::Tabs,
            A2uiComponentType::Chart,
            A2uiComponentType::Calendar,
            A2uiComponentType::AudioPlayer,
            A2uiComponentType::ShaderStage,
            A2uiComponentType::Badge,
            A2uiComponentType::Alert,
            A2uiComponentType::Avatar,
            A2uiComponentType::Progress,
            A2uiComponentType::Spinner,
            A2uiComponentType::Switch,
            A2uiComponentType::Accordion,
            A2uiComponentType::Notification,
            A2uiComponentType::Skeleton,
        ]
    }
}

/// Mapping information for a component type
#[derive(Debug, Clone)]
pub struct ComponentMapping {
    /// The A2UI component type
    pub a2ui_type: A2uiComponentType,

    /// The corresponding Makepad widget type name
    pub makepad_widget: &'static str,

    /// Description of the component
    pub description: &'static str,

    /// Whether this mapping is fully implemented
    pub implemented: bool,
}

/// Registry for A2UI to Makepad component mappings.
///
/// The registry maintains mappings between A2UI component types and their
/// corresponding Makepad widget implementations.
///
/// # Example
///
/// ```rust,ignore
/// let registry = ComponentRegistry::with_standard_catalog();
///
/// // Get mapping for a component type
/// if let Some(mapping) = registry.get(A2uiComponentType::Button) {
///     println!("Button maps to: {}", mapping.makepad_widget);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    mappings: HashMap<A2uiComponentType, ComponentMapping>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        ComponentRegistry {
            mappings: HashMap::new(),
        }
    }

    /// Create a registry with the standard A2UI catalog mappings
    pub fn with_standard_catalog() -> Self {
        let mut registry = Self::new();

        // Layout components
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Column,
            makepad_widget: "View",
            description: "Vertical layout container (flow: Down)",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Row,
            makepad_widget: "View",
            description: "Horizontal layout container (flow: Right)",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::List,
            makepad_widget: "PortalList",
            description: "Scrollable list with virtualization",
            implemented: false,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Card,
            makepad_widget: "MpCard",
            description: "Card container with elevation/shadow",
            implemented: true,
        });

        // Display components
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Text,
            makepad_widget: "MpLabel",
            description: "Text display with usage hints",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Image,
            makepad_widget: "Image",
            description: "Image display with fit modes",
            implemented: false,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Icon,
            makepad_widget: "Icon",
            description: "Icon display",
            implemented: false,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Divider,
            makepad_widget: "MpDivider",
            description: "Visual separator",
            implemented: true,
        });

        // Interactive components
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Button,
            makepad_widget: "MpButton",
            description: "Clickable button with action",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::TextField,
            makepad_widget: "MpInput",
            description: "Text input field with two-way binding",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::CheckBox,
            makepad_widget: "MpCheckbox",
            description: "Boolean toggle checkbox",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Slider,
            makepad_widget: "MpSlider",
            description: "Numeric range slider",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::MultipleChoice,
            makepad_widget: "MpDropdown",
            description: "Selection from multiple options",
            implemented: false,
        });

        // Container components
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Modal,
            makepad_widget: "MpModal",
            description: "Modal dialog overlay",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Tabs,
            makepad_widget: "MpTabPill",
            description: "Tabbed interface",
            implemented: true,
        });

        // Visualization components
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Chart,
            makepad_widget: "A2uiChart",
            description: "Chart visualization (bar, line, pie)",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Calendar,
            makepad_widget: "A2uiCalendar",
            description: "Calendar/grid visualization for schedules and planners",
            implemented: true,
        });

        // Media components
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::AudioPlayer,
            makepad_widget: "A2uiAudioPlayer",
            description: "Audio player with playback controls",
            implemented: true,
        });

        // Shader visualization components
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::ShaderStage,
            makepad_widget: "DrawQuad (shader stage)",
            description: "Full-screen shader art stage with audio-reactive effects",
            implemented: true,
        });

        // shadcn UI components (Splash script layer)
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Badge,
            makepad_widget: "MpBadgeStandalone",
            description: "Inline badge / pill label (default, secondary, destructive, outline)",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Alert,
            makepad_widget: "MpAlert",
            description: "Alert banner (default, destructive, warning, success, info)",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Avatar,
            makepad_widget: "MpAvatar",
            description: "Circular avatar with initials fallback",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Progress,
            makepad_widget: "MpProgress",
            description: "Horizontal progress bar (0–100)",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Spinner,
            makepad_widget: "MpSpinner",
            description: "Rotating loading spinner",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Switch,
            makepad_widget: "MpSwitch",
            description: "Toggle switch with two-way data binding",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Accordion,
            makepad_widget: "MpAccordion",
            description: "Collapsible accordion sections",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Notification,
            makepad_widget: "MpNotification",
            description: "Toast / notification popup",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Skeleton,
            makepad_widget: "MpSkeleton",
            description: "Loading skeleton placeholder",
            implemented: true,
        });

        registry
    }

    /// Register a component mapping
    pub fn register(&mut self, mapping: ComponentMapping) {
        self.mappings.insert(mapping.a2ui_type, mapping);
    }

    /// Get a component mapping
    pub fn get(&self, component_type: A2uiComponentType) -> Option<&ComponentMapping> {
        self.mappings.get(&component_type)
    }

    /// Get a component mapping by name
    pub fn get_by_name(&self, name: &str) -> Option<&ComponentMapping> {
        A2uiComponentType::from_str(name).and_then(|t| self.get(t))
    }

    /// Check if a component type is registered
    pub fn contains(&self, component_type: A2uiComponentType) -> bool {
        self.mappings.contains_key(&component_type)
    }

    /// Get all registered mappings
    pub fn all_mappings(&self) -> impl Iterator<Item = &ComponentMapping> {
        self.mappings.values()
    }

    /// Get the Makepad widget type for an A2UI component
    pub fn makepad_widget_for(&self, component_type: A2uiComponentType) -> Option<&'static str> {
        self.get(component_type).map(|m| m.makepad_widget)
    }

    /// Get implemented component types
    pub fn implemented_types(&self) -> Vec<A2uiComponentType> {
        self.mappings
            .values()
            .filter(|m| m.implemented)
            .map(|m| m.a2ui_type)
            .collect()
    }

    /// Get unimplemented component types
    pub fn unimplemented_types(&self) -> Vec<A2uiComponentType> {
        self.mappings
            .values()
            .filter(|m| !m.implemented)
            .map(|m| m.a2ui_type)
            .collect()
    }
}

/// Get the component type from a ComponentType enum variant
pub fn component_type_of(component: &super::message::ComponentType) -> A2uiComponentType {
    use super::message::ComponentType;
    match component {
        ComponentType::Column(_) => A2uiComponentType::Column,
        ComponentType::Row(_) => A2uiComponentType::Row,
        ComponentType::List(_) => A2uiComponentType::List,
        ComponentType::Card(_) => A2uiComponentType::Card,
        ComponentType::Text(_) => A2uiComponentType::Text,
        ComponentType::Image(_) => A2uiComponentType::Image,
        ComponentType::Icon(_) => A2uiComponentType::Icon,
        ComponentType::Divider(_) => A2uiComponentType::Divider,
        ComponentType::Button(_) => A2uiComponentType::Button,
        ComponentType::TextField(_) => A2uiComponentType::TextField,
        ComponentType::CheckBox(_) => A2uiComponentType::CheckBox,
        ComponentType::Slider(_) => A2uiComponentType::Slider,
        ComponentType::MultipleChoice(_) => A2uiComponentType::MultipleChoice,
        ComponentType::Modal(_) => A2uiComponentType::Modal,
        ComponentType::Tabs(_) => A2uiComponentType::Tabs,
        ComponentType::Chart(_) => A2uiComponentType::Chart,
        ComponentType::Calendar(_) => A2uiComponentType::Calendar,
        ComponentType::AudioPlayer(_) => A2uiComponentType::AudioPlayer,
        ComponentType::ShaderStage(_) => A2uiComponentType::ShaderStage,
        ComponentType::Badge(_) => A2uiComponentType::Badge,
        ComponentType::Alert(_) => A2uiComponentType::Alert,
        ComponentType::Avatar(_) => A2uiComponentType::Avatar,
        ComponentType::Progress(_) => A2uiComponentType::Progress,
        ComponentType::Spinner(_) => A2uiComponentType::Spinner,
        ComponentType::Switch(_) => A2uiComponentType::Switch,
        ComponentType::Accordion(_) => A2uiComponentType::Accordion,
        ComponentType::Notification(_) => A2uiComponentType::Notification,
        ComponentType::Skeleton(_) => A2uiComponentType::Skeleton,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_catalog() {
        let registry = ComponentRegistry::with_standard_catalog();

        // All component types should be registered
        for component_type in A2uiComponentType::all() {
            assert!(
                registry.contains(*component_type),
                "Missing mapping for {:?}",
                component_type
            );
        }
    }

    #[test]
    fn test_get_mapping() {
        let registry = ComponentRegistry::with_standard_catalog();

        let mapping = registry.get(A2uiComponentType::Button).unwrap();
        assert_eq!(mapping.makepad_widget, "MpButton");
        assert!(mapping.implemented);
    }

    #[test]
    fn test_get_by_name() {
        let registry = ComponentRegistry::with_standard_catalog();

        let mapping = registry.get_by_name("Text").unwrap();
        assert_eq!(mapping.a2ui_type, A2uiComponentType::Text);
    }

    #[test]
    fn test_implemented_types() {
        let registry = ComponentRegistry::with_standard_catalog();

        let implemented = registry.implemented_types();
        assert!(implemented.contains(&A2uiComponentType::Button));
        assert!(implemented.contains(&A2uiComponentType::Text));
    }
}
