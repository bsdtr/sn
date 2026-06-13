use ratatui::style::{Color, Modifier, Style};

/// Terminal color scheme inferred from the environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

/// Detect light vs dark terminal background.
///
/// Uses `SN_LIGHT_THEME` overrides, then `COLORFGBG` when set.
/// When unknown, uses [`Theme::Dark`] which keeps terminal default colors (no hardcoded white).
pub fn detect() -> Theme {
    if std::env::var("SN_LIGHT_THEME").is_ok() {
        return Theme::Light;
    }
    if colorfgbg_is_light() {
        return Theme::Light;
    }
    Theme::Dark
}

fn colorfgbg_is_light() -> bool {
    std::env::var("COLORFGBG")
        .ok()
        .and_then(|value| value.rsplit(';').next()?.parse::<u8>().ok())
        .is_some_and(|bg| matches!(bg, 7 | 15) || bg >= 250)
}

impl Theme {
    /// Primary readable text (note content, lists, etc.).
    pub fn text_primary(self) -> Style {
        match self {
            Theme::Light => Style::default().fg(Color::Black),
            // Follow the terminal palette — never force white (invisible on light backgrounds).
            Theme::Dark => Style::default(),
        }
    }

    /// Text inside popup dialogs (create/delete note).
    pub fn popup_text(self) -> Style {
        match self {
            Theme::Light => Style::default().fg(Color::Black).bg(Color::White),
            Theme::Dark => Style::default(),
        }
    }

    /// Popup/dialog block surface.
    pub fn popup_surface(self) -> Style {
        match self {
            Theme::Light => Style::default().fg(Color::Black).bg(Color::White),
            Theme::Dark => Style::default(),
        }
    }

    /// Secondary / hint text.
    pub fn text_secondary(self) -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// Dimmed status-bar help text.
    pub fn text_dim(self) -> Style {
        Style::default().add_modifier(Modifier::DIM)
    }

    /// Selected list row.
    pub fn selection(self) -> Style {
        match self {
            Theme::Light => Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            Theme::Dark => Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Heading style for H5/H6 on any background.
    pub fn heading_emphasis(self) -> Style {
        match self {
            Theme::Light => Style::default()
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
            Theme::Dark => Style::default().add_modifier(Modifier::BOLD),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_colorfgbg_is_detected() {
        std::env::set_var("COLORFGBG", "0;15");
        assert_eq!(detect(), Theme::Light);
        std::env::remove_var("COLORFGBG");
    }

    #[test]
    fn dark_colorfgbg_is_detected() {
        std::env::set_var("COLORFGBG", "15;0");
        assert_eq!(detect(), Theme::Dark);
        std::env::remove_var("COLORFGBG");
    }
}
