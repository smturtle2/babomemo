use ratatui::style::{Modifier, Style};

pub struct TerminalStyle;

impl TerminalStyle {
    pub fn background() -> Style {
        Style::default()
    }

    pub fn toolbar() -> Style {
        Style::default()
    }

    pub fn button() -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }

    pub fn destructive_button() -> Style {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
    }

    pub fn disabled() -> Style {
        Style::default().add_modifier(Modifier::DIM)
    }

    pub fn border(focused: bool) -> Style {
        if focused {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        }
    }

    pub fn text() -> Style {
        Style::default()
    }

    pub fn selection() -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }

    pub fn status() -> Style {
        Style::default().add_modifier(Modifier::DIM)
    }

    pub fn error() -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    pub fn modal() -> Style {
        Style::default()
    }
}
