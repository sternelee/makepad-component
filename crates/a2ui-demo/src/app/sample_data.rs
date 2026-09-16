/// Get sample A2UI JSON for a product catalog with form inputs
pub(crate) fn get_sample_product_catalog() -> String {
    r##"[
        {
            "beginRendering": {
                "surfaceId": "main",
                "root": "root-column"
            }
        },
        {
            "surfaceUpdate": {
                "surfaceId": "main",
                "components": [
                    {
                        "id": "root-column",
                        "component": {
                            "Column": {
                                "children": {
                                    "explicitList": ["header", "filters-section", "product-list"]
                                }
                            }
                        }
                    },
                    {
                        "id": "header",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Products"},
                                "usageHint": "h1"
                            }
                        }
                    },
                    {
                        "id": "filters-section",
                        "component": {
                            "Card": {
                                "child": "filters-content"
                            }
                        }
                    },
                    {
                        "id": "filters-content",
                        "component": {
                            "Column": {
                                "children": {
                                    "explicitList": ["filters-title", "search-row", "options-row", "price-row"]
                                }
                            }
                        }
                    },
                    {
                        "id": "filters-title",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Filters"},
                                "usageHint": "h3"
                            }
                        }
                    },
                    {
                        "id": "search-row",
                        "component": {
                            "Row": {
                                "children": {
                                    "explicitList": ["search-label", "search-input"]
                                }
                            }
                        }
                    },
                    {
                        "id": "search-label",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Search:"}
                            }
                        }
                    },
                    {
                        "id": "search-input",
                        "component": {
                            "TextField": {
                                "text": {"path": "/filters/search"},
                                "placeholder": {"literalString": "Enter product name..."}
                            }
                        }
                    },
                    {
                        "id": "options-row",
                        "component": {
                            "Row": {
                                "children": {
                                    "explicitList": ["in-stock-checkbox", "on-sale-checkbox"]
                                }
                            }
                        }
                    },
                    {
                        "id": "in-stock-checkbox",
                        "component": {
                            "CheckBox": {
                                "value": {"path": "/filters/inStock"},
                                "label": {"literalString": "In Stock Only"}
                            }
                        }
                    },
                    {
                        "id": "on-sale-checkbox",
                        "component": {
                            "CheckBox": {
                                "value": {"path": "/filters/onSale"},
                                "label": {"literalString": "On Sale"}
                            }
                        }
                    },
                    {
                        "id": "price-row",
                        "component": {
                            "Row": {
                                "children": {
                                    "explicitList": ["price-label", "price-slider", "price-value"]
                                }
                            }
                        }
                    },
                    {
                        "id": "price-label",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Max Price:"}
                            }
                        }
                    },
                    {
                        "id": "price-slider",
                        "component": {
                            "Slider": {
                                "value": {"path": "/filters/maxPrice"},
                                "min": 0,
                                "max": 200,
                                "step": 10
                            }
                        }
                    },
                    {
                        "id": "price-value",
                        "component": {
                            "Text": {
                                "text": {"path": "/filters/maxPriceDisplay"}
                            }
                        }
                    },
                    {
                        "id": "product-list",
                        "component": {
                            "Column": {
                                "children": {
                                    "explicitList": ["product-1", "product-2", "product-3"]
                                }
                            }
                        }
                    },
                    {
                        "id": "product-1",
                        "component": {
                            "Card": {
                                "child": "product-1-content"
                            }
                        }
                    },
                    {
                        "id": "product-1-content",
                        "component": {
                            "Row": {
                                "children": {
                                    "explicitList": ["product-1-image", "product-1-info", "product-1-btn"]
                                }
                            }
                        }
                    },
                    {
                        "id": "product-1-image",
                        "component": {
                            "Image": {
                                "url": {"literalString": "https://example.com/headphones.jpg"},
                                "usageHint": "smallFeature"
                            }
                        }
                    },
                    {
                        "id": "product-1-info",
                        "component": {
                            "Column": {
                                "children": {
                                    "explicitList": ["product-1-name", "product-1-price"]
                                }
                            }
                        }
                    },
                    {
                        "id": "product-1-name",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Premium Headphones"},
                                "usageHint": "h3"
                            }
                        }
                    },
                    {
                        "id": "product-1-price",
                        "component": {
                            "Text": {
                                "text": {"literalString": "$99.99"}
                            }
                        }
                    },
                    {
                        "id": "product-1-btn",
                        "component": {
                            "Button": {
                                "child": "product-1-btn-text",
                                "primary": true,
                                "action": {
                                    "name": "addToCart",
                                    "context": [
                                        {"key": "productId", "value": {"literalString": "SKU001"}}
                                    ]
                                }
                            }
                        }
                    },
                    {
                        "id": "product-1-btn-text",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Add to Cart"}
                            }
                        }
                    },
                    {
                        "id": "product-2",
                        "component": {
                            "Card": {
                                "child": "product-2-content"
                            }
                        }
                    },
                    {
                        "id": "product-2-content",
                        "component": {
                            "Row": {
                                "children": {
                                    "explicitList": ["product-2-image", "product-2-info", "product-2-btn"]
                                }
                            }
                        }
                    },
                    {
                        "id": "product-2-image",
                        "component": {
                            "Image": {
                                "url": {"literalString": "https://example.com/mouse.jpg"},
                                "usageHint": "smallFeature"
                            }
                        }
                    },
                    {
                        "id": "product-2-info",
                        "component": {
                            "Column": {
                                "children": {
                                    "explicitList": ["product-2-name", "product-2-price"]
                                }
                            }
                        }
                    },
                    {
                        "id": "product-2-name",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Wireless Mouse"},
                                "usageHint": "h3"
                            }
                        }
                    },
                    {
                        "id": "product-2-price",
                        "component": {
                            "Text": {
                                "text": {"literalString": "$49.99"}
                            }
                        }
                    },
                    {
                        "id": "product-2-btn",
                        "component": {
                            "Button": {
                                "child": "product-2-btn-text",
                                "primary": true,
                                "action": {
                                    "name": "addToCart",
                                    "context": [
                                        {"key": "productId", "value": {"literalString": "SKU002"}}
                                    ]
                                }
                            }
                        }
                    },
                    {
                        "id": "product-2-btn-text",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Add to Cart"}
                            }
                        }
                    },
                    {
                        "id": "product-3",
                        "component": {
                            "Card": {
                                "child": "product-3-content"
                            }
                        }
                    },
                    {
                        "id": "product-3-content",
                        "component": {
                            "Row": {
                                "children": {
                                    "explicitList": ["product-3-image", "product-3-info", "product-3-btn"]
                                }
                            }
                        }
                    },
                    {
                        "id": "product-3-image",
                        "component": {
                            "Image": {
                                "url": {"literalString": "https://example.com/keyboard.jpg"},
                                "usageHint": "smallFeature"
                            }
                        }
                    },
                    {
                        "id": "product-3-info",
                        "component": {
                            "Column": {
                                "children": {
                                    "explicitList": ["product-3-name", "product-3-price"]
                                }
                            }
                        }
                    },
                    {
                        "id": "product-3-name",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Mechanical Keyboard"},
                                "usageHint": "h3"
                            }
                        }
                    },
                    {
                        "id": "product-3-price",
                        "component": {
                            "Text": {
                                "text": {"literalString": "$129.99"}
                            }
                        }
                    },
                    {
                        "id": "product-3-btn",
                        "component": {
                            "Button": {
                                "child": "product-3-btn-text",
                                "primary": true,
                                "action": {
                                    "name": "addToCart",
                                    "context": [
                                        {"key": "productId", "value": {"literalString": "SKU003"}}
                                    ]
                                }
                            }
                        }
                    },
                    {
                        "id": "product-3-btn-text",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Add to Cart"}
                            }
                        }
                    }
                ]
            }
        },
        {
            "dataModelUpdate": {
                "surfaceId": "main",
                "path": "/",
                "contents": [
                    {
                        "key": "filters",
                        "valueMap": [
                            {"key": "search", "valueString": ""},
                            {"key": "inStock", "valueBoolean": true},
                            {"key": "onSale", "valueBoolean": false},
                            {"key": "maxPrice", "valueNumber": 150},
                            {"key": "maxPriceDisplay", "valueString": "$150"}
                        ]
                    }
                ]
            }
        }
    ]"##.to_string()
}

/// Get sample A2UI JSON for a music player with 3 songs, volume control, and visualizations
pub(crate) fn get_sample_music_player() -> String {
    r##"[
        {
            "beginRendering": {
                "surfaceId": "main",
                "root": "root-column"
            }
        },
        {
            "surfaceUpdate": {
                "surfaceId": "main",
                "components": [
                    {
                        "id": "root-column",
                        "component": {
                            "Column": {
                                "children": { "explicitList": ["main-title", "player-card", "volume-row", "status-text"] }
                            }
                        }
                    },
                    {
                        "id": "main-title",
                        "component": {
                            "Text": {
                                "text": { "literalString": "🎵 Makepad Music Player" },
                                "usageHint": "h1"
                            }
                        }
                    },
                    {
                        "id": "player-card",
                        "component": {
                            "Card": { "child": "player-list" }
                        }
                    },
                    {
                        "id": "player-list",
                        "component": {
                            "Column": {
                                "children": { "explicitList": ["song-1", "song-2", "song-3"] }
                            }
                        }
                    },
                    {
                        "id": "song-1",
                        "component": {
                            "AudioPlayer": {
                                "url": { "literalString": "https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3" },
                                "title": { "literalString": "SoundHelix Song 1" },
                                "artist": { "literalString": "SoundHelix" },
                                "visualization": "barsAndTaiji"
                            }
                        }
                    },
                    {
                        "id": "song-2",
                        "component": {
                            "AudioPlayer": {
                                "url": { "literalString": "https://www.soundhelix.com/examples/mp3/SoundHelix-Song-2.mp3" },
                                "title": { "literalString": "SoundHelix Song 2" },
                                "artist": { "literalString": "SoundHelix" },
                                "visualization": "barsAndTaiji"
                            }
                        }
                    },
                    {
                        "id": "song-3",
                        "component": {
                            "AudioPlayer": {
                                "url": { "literalString": "https://www.soundhelix.com/examples/mp3/SoundHelix-Song-3.mp3" },
                                "title": { "literalString": "SoundHelix Song 3" },
                                "artist": { "literalString": "SoundHelix" },
                                "visualization": "barsAndTaiji"
                            }
                        }
                    },
                    {
                        "id": "volume-row",
                        "component": {
                            "Row": {
                                "children": { "explicitList": ["volume-label", "volume-slider"] },
                                "alignment": "center"
                            }
                        }
                    },
                    {
                        "id": "volume-label",
                        "component": {
                            "Text": {
                                "text": { "literalString": "🔊 Volume" },
                                "usageHint": "body"
                            }
                        }
                    },
                    {
                        "id": "volume-slider",
                        "component": {
                            "Slider": {
                                "min": 0,
                                "max": 100,
                                "value": { "literalNumber": 50 },
                                "binding": "/volume"
                            }
                        }
                    },
                    {
                        "id": "status-text",
                        "component": {
                            "Text": {
                                "text": { "literalString": "Ready to play" },
                                "usageHint": "caption"
                            }
                        }
                    }
                ]
            }
        },
        {
            "dataModelUpdate": {
                "surfaceId": "main",
                "path": "/",
                "contents": [
                    { "key": "volume", "valueNumber": 50 }
                ]
            }
        }
    ]"##.to_string()
}

/// A fixture whose only job is to reach the **icon** path, named `A2UI_SAMPLE=icons`.
///
/// **Because nothing else did.** The catalog sample's three `addToCart` strings are the *button action's* name rather
/// than an Icon component, so the moment the icon pool moved to `mp` it had no sample that would reach it — and a
/// migration that nothing exercises is not verified, however green the build. That is the same lesson `[E]=0` taught
/// earlier in this port: a log without errors is what a renderer that did nothing also produces.
///
/// It carries **both** answers on purpose. Four names resolve, and `not-a-real-icon` does not, so one run proves the
/// glyph is drawn and that an unknown name draws **nothing** rather than a guessed box.
pub(crate) fn get_sample_icons() -> String {
    r##"[
        {
            "beginRendering": {
                "surfaceId": "main",
                "root": "root-column"
            }
        },
        {
            "surfaceUpdate": {
                "surfaceId": "main",
                "components": [
                    {
                        "id": "root-column",
                        "component": {
                            "Column": {
                                "children": {
                                    "explicitList": ["title", "row", "unknown-row"]
                                }
                            }
                        }
                    },
                    {
                        "id": "title",
                        "component": {
                            "Text": {
                                "text": {"literalString": "Icons"},
                                "usageHint": "h1"
                            }
                        }
                    },
                    {
                        "id": "row",
                        "component": {
                            "Row": {
                                "children": {
                                    "explicitList": ["i-settings", "i-check", "i-close", "i-cart"]
                                }
                            }
                        }
                    },
                    {"id": "i-settings", "component": {"Icon": {"name": {"literalString": "settings"}}}},
                    {"id": "i-check", "component": {"Icon": {"name": {"literalString": "check"}}}},
                    {"id": "i-close", "component": {"Icon": {"name": {"literalString": "close"}}}},
                    {"id": "i-cart", "component": {"Icon": {"name": {"literalString": "addToCart"}}}},
                    {
                        "id": "unknown-row",
                        "component": {
                            "Row": {
                                "children": {
                                    "explicitList": ["i-unknown"]
                                }
                            }
                        }
                    },
                    {"id": "i-unknown", "component": {"Icon": {"name": {"literalString": "not-a-real-icon"}}}}
                ]
            }
        }
    ]"##
    .to_string()
}
