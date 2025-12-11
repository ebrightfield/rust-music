//! Color themes and styling for SVG diagrams.

/// Theme for consistent styling across diagrams.
#[derive(Clone, Debug)]
pub struct SvgTheme {
    pub highlight_color: &'static str,
    pub root_color: &'static str,
    pub background_color: &'static str,
    pub stroke_color: &'static str,
    pub text_color: &'static str,
    pub inactive_color: &'static str,
}

impl Default for SvgTheme {
    fn default() -> Self {
        Self {
            highlight_color: "#4a90d9",
            root_color: "#2d5a87",
            background_color: "#ffffff",
            stroke_color: "#333333",
            text_color: "#333333",
            inactive_color: "#cccccc",
        }
    }
}

impl SvgTheme {
    /// Dark theme suitable for dark mode UIs.
    pub fn dark() -> Self {
        Self {
            highlight_color: "#5dade2",
            root_color: "#2e86ab",
            background_color: "#1a1a2e",
            stroke_color: "#666666",
            text_color: "#cccccc",
            inactive_color: "#333333",
        }
    }

    /// High contrast black and white theme suitable for printing.
    pub fn print() -> Self {
        Self {
            highlight_color: "#000000",
            root_color: "#000000",
            background_color: "#ffffff",
            stroke_color: "#000000",
            text_color: "#000000",
            inactive_color: "#999999",
        }
    }

    /// Colorful theme with warm highlight colors.
    pub fn colorful() -> Self {
        Self {
            highlight_color: "#e74c3c",
            root_color: "#c0392b",
            background_color: "#ffffff",
            stroke_color: "#2c3e50",
            text_color: "#2c3e50",
            inactive_color: "#bdc3c7",
        }
    }
}
