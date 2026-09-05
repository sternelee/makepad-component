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

    // Raycast-style containers
    Detail,
    Form,
    ActionPanel,
    Grid,

    // Raycast-style form components
    PasswordField,
    TextArea,
    DatePicker,
    Dropdown,
    TagPicker,
    FilePicker,
    ListItem,
    DropdownItem,
    DropdownSection,
    TagPickerItem,

    // Extended components
    Tag,
    StepIndicator,
    NumberInput,
    SearchableList,
    StatusBar,
    AvatarGroup,
    ColorPicker,
    DescriptionList,
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
            // Raycast-style containers
            A2uiComponentType::Detail => "Detail",
            A2uiComponentType::Form => "Form",
            A2uiComponentType::ActionPanel => "ActionPanel",
            A2uiComponentType::Grid => "Grid",
            // Raycast-style form components
            A2uiComponentType::PasswordField => "PasswordField",
            A2uiComponentType::TextArea => "TextArea",
            A2uiComponentType::DatePicker => "DatePicker",
            A2uiComponentType::Dropdown => "Dropdown",
            A2uiComponentType::TagPicker => "TagPicker",
            A2uiComponentType::FilePicker => "FilePicker",
            A2uiComponentType::ListItem => "ListItem",
            A2uiComponentType::DropdownItem => "DropdownItem",
            A2uiComponentType::DropdownSection => "DropdownSection",
            A2uiComponentType::TagPickerItem => "TagPickerItem",
            // Extended components
            A2uiComponentType::Tag => "Tag",
            A2uiComponentType::StepIndicator => "StepIndicator",
            A2uiComponentType::NumberInput => "NumberInput",
            A2uiComponentType::SearchableList => "SearchableList",
            A2uiComponentType::StatusBar => "StatusBar",
            A2uiComponentType::AvatarGroup => "AvatarGroup",
            A2uiComponentType::ColorPicker => "ColorPicker",
            A2uiComponentType::DescriptionList => "DescriptionList",
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
            // Raycast-style containers
            "Detail" => Some(A2uiComponentType::Detail),
            "Form" => Some(A2uiComponentType::Form),
            "ActionPanel" => Some(A2uiComponentType::ActionPanel),
            "Grid" => Some(A2uiComponentType::Grid),
            // Raycast-style form components
            "PasswordField" => Some(A2uiComponentType::PasswordField),
            "TextArea" => Some(A2uiComponentType::TextArea),
            "DatePicker" => Some(A2uiComponentType::DatePicker),
            "Dropdown" => Some(A2uiComponentType::Dropdown),
            "TagPicker" => Some(A2uiComponentType::TagPicker),
            "FilePicker" => Some(A2uiComponentType::FilePicker),
            "ListItem" => Some(A2uiComponentType::ListItem),
            "DropdownItem" => Some(A2uiComponentType::DropdownItem),
            "DropdownSection" => Some(A2uiComponentType::DropdownSection),
            "TagPickerItem" => Some(A2uiComponentType::TagPickerItem),
            // Extended components
            "Tag" => Some(A2uiComponentType::Tag),
            "StepIndicator" => Some(A2uiComponentType::StepIndicator),
            "NumberInput" => Some(A2uiComponentType::NumberInput),
            "SearchableList" => Some(A2uiComponentType::SearchableList),
            "StatusBar" => Some(A2uiComponentType::StatusBar),
            "AvatarGroup" => Some(A2uiComponentType::AvatarGroup),
            "ColorPicker" => Some(A2uiComponentType::ColorPicker),
            "DescriptionList" => Some(A2uiComponentType::DescriptionList),
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
            // Raycast-style containers
            A2uiComponentType::Detail,
            A2uiComponentType::Form,
            A2uiComponentType::ActionPanel,
            A2uiComponentType::Grid,
            // Raycast-style form components
            A2uiComponentType::PasswordField,
            A2uiComponentType::TextArea,
            A2uiComponentType::DatePicker,
            A2uiComponentType::Dropdown,
            A2uiComponentType::TagPicker,
            A2uiComponentType::FilePicker,
            A2uiComponentType::ListItem,
            A2uiComponentType::DropdownItem,
            A2uiComponentType::DropdownSection,
            A2uiComponentType::TagPickerItem,
            // Extended components
            A2uiComponentType::Tag,
            A2uiComponentType::StepIndicator,
            A2uiComponentType::NumberInput,
            A2uiComponentType::SearchableList,
            A2uiComponentType::StatusBar,
            A2uiComponentType::AvatarGroup,
            A2uiComponentType::ColorPicker,
            A2uiComponentType::DescriptionList,
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

        // Raycast-style containers
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Detail,
            makepad_widget: "View",
            description: "Raycast-style detail view (markdown content + metadata)",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Form,
            makepad_widget: "View",
            description: "Raycast-style form container",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::ActionPanel,
            makepad_widget: "View",
            description: "Raycast-style action panel with action buttons",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Grid,
            makepad_widget: "View",
            description: "Raycast-style grid layout container",
            implemented: true,
        });

        // Raycast-style form components
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::PasswordField,
            makepad_widget: "TextInput",
            description: "Password input with hidden text",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::TextArea,
            makepad_widget: "TextInput",
            description: "Multi-line text input (rendered as larger TextInput)",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::DatePicker,
            makepad_widget: "TextInput",
            description: "Date picker input field",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Dropdown,
            makepad_widget: "MpDropdown",
            description: "Raycast-style dropdown selection",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::TagPicker,
            makepad_widget: "View",
            description: "Raycast-style tag picker with selectable tokens",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::FilePicker,
            makepad_widget: "TextInput",
            description: "File picker input with browse button",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::ListItem,
            makepad_widget: "View",
            description: "Raycast-style list item with icon, title and accessories",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::DropdownItem,
            makepad_widget: "Label",
            description: "Dropdown item (rendered as child of Dropdown)",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::DropdownSection,
            makepad_widget: "View",
            description: "Dropdown section (rendered as child of Dropdown)",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::TagPickerItem,
            makepad_widget: "View",
            description: "Tag picker item (rendered as child of TagPicker)",
            implemented: true,
        });

        // Extended components
        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::Tag,
            makepad_widget: "MpTag",
            description: "Semantic status tag",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::StepIndicator,
            makepad_widget: "MpStepIndicator",
            description: "Step progress indicator",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::NumberInput,
            makepad_widget: "MpNumberInput",
            description: "Numeric input with steppers and bounds",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::SearchableList,
            makepad_widget: "MpSearchableList",
            description: "Filterable list with search box",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::StatusBar,
            makepad_widget: "MpStatusBar",
            description: "Status bar strip with hairline top border",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::AvatarGroup,
            makepad_widget: "MpAvatarGroup",
            description: "Overlapping avatar stack with overflow tail",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::ColorPicker,
            makepad_widget: "MpColorPicker",
            description: "Color swatch grid",
            implemented: true,
        });

        registry.register(ComponentMapping {
            a2ui_type: A2uiComponentType::DescriptionList,
            makepad_widget: "MpDescriptionList",
            description: "Term/description pairs",
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
        // Raycast-style containers
        ComponentType::Detail(_) => A2uiComponentType::Detail,
        ComponentType::Form(_) => A2uiComponentType::Form,
        ComponentType::ActionPanel(_) => A2uiComponentType::ActionPanel,
        ComponentType::Grid(_) => A2uiComponentType::Grid,
        // Raycast-style form components
        ComponentType::PasswordField(_) => A2uiComponentType::PasswordField,
        ComponentType::TextArea(_) => A2uiComponentType::TextArea,
        ComponentType::DatePicker(_) => A2uiComponentType::DatePicker,
        ComponentType::Dropdown(_) => A2uiComponentType::Dropdown,
        ComponentType::TagPicker(_) => A2uiComponentType::TagPicker,
        ComponentType::FilePicker(_) => A2uiComponentType::FilePicker,
        ComponentType::ListItem(_) => A2uiComponentType::ListItem,
        ComponentType::DropdownItem(_) => A2uiComponentType::DropdownItem,
        ComponentType::DropdownSection(_) => A2uiComponentType::DropdownSection,
        ComponentType::TagPickerItem(_) => A2uiComponentType::TagPickerItem,
        // Extended components
        ComponentType::Tag(_) => A2uiComponentType::Tag,
        ComponentType::StepIndicator(_) => A2uiComponentType::StepIndicator,
        ComponentType::NumberInput(_) => A2uiComponentType::NumberInput,
        ComponentType::SearchableList(_) => A2uiComponentType::SearchableList,
        ComponentType::StatusBar(_) => A2uiComponentType::StatusBar,
        ComponentType::AvatarGroup(_) => A2uiComponentType::AvatarGroup,
        ComponentType::ColorPicker(_) => A2uiComponentType::ColorPicker,
        ComponentType::DescriptionList(_) => A2uiComponentType::DescriptionList,
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
