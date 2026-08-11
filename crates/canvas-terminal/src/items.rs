use makepad_widgets::*;

use crate::terminal::session::TerminalSession;

/// What a canvas item can be.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    Note,
    Terminal,
    Browser,
}

/// A single canvas item (note, terminal, or browser).
pub enum CanvasItem {
    Note {
        id: u64,
        world: Rect,
        title: String,
    },
    Terminal {
        id: u64,
        world: Rect,
        title: String,
        session: Option<Box<TerminalSession>>,
    },
    Browser {
        id: u64,
        world: Rect,
        title: String,
        url: String,
    },
}

impl CanvasItem {
    pub fn id(&self) -> u64 {
        match self {
            CanvasItem::Note { id, .. }
            | CanvasItem::Terminal { id, .. }
            | CanvasItem::Browser { id, .. } => *id,
        }
    }

    pub fn kind(&self) -> ItemKind {
        match self {
            CanvasItem::Note { .. } => ItemKind::Note,
            CanvasItem::Terminal { .. } => ItemKind::Terminal,
            CanvasItem::Browser { .. } => ItemKind::Browser,
        }
    }

    pub fn world(&self) -> Rect {
        match self {
            CanvasItem::Note { world, .. }
            | CanvasItem::Terminal { world, .. }
            | CanvasItem::Browser { world, .. } => *world,
        }
    }

    pub fn world_mut(&mut self) -> &mut Rect {
        match self {
            CanvasItem::Note { world, .. }
            | CanvasItem::Terminal { world, .. }
            | CanvasItem::Browser { world, .. } => world,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            CanvasItem::Note { title, .. }
            | CanvasItem::Terminal { title, .. }
            | CanvasItem::Browser { title, .. } => title,
        }
    }

    #[allow(dead_code)]
    pub fn title_mut(&mut self) -> &mut String {
        match self {
            CanvasItem::Note { title, .. }
            | CanvasItem::Terminal { title, .. }
            | CanvasItem::Browser { title, .. } => title,
        }
    }

    pub fn session(&self) -> Option<&TerminalSession> {
        match self {
            CanvasItem::Terminal { session, .. } => session.as_deref(),
            CanvasItem::Note { .. } | CanvasItem::Browser { .. } => None,
        }
    }

    #[allow(dead_code)]
    pub fn session_mut(&mut self) -> Option<&mut TerminalSession> {
        match self {
            CanvasItem::Terminal { session, .. } => session.as_deref_mut(),
            CanvasItem::Note { .. } | CanvasItem::Browser { .. } => None,
        }
    }

    pub fn url(&self) -> Option<&str> {
        match self {
            CanvasItem::Browser { url, .. } => Some(url),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn url_mut(&mut self) -> Option<&mut String> {
        match self {
            CanvasItem::Browser { url, .. } => Some(url),
            _ => None,
        }
    }
}
