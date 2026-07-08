//! Theme colours for the TUI — Catppuccin Mocha palette.
//!
//! Provides a central Theme struct with static methods so every widget
//! gets consistent colours without repeating hex values.

use ratatui::style::Color;

/// Catppuccin Mocha colour tokens for the terminal UI.
pub struct Theme;

impl Theme {
    /// Primary accent — Blue `#89b4fa`
    pub fn accent() -> Color {
        Color::Rgb(0x89, 0xb4, 0xfa)
    }

    /// Primary text — Text `#cdd6f4`
    pub fn text() -> Color {
        Color::Rgb(0xcd, 0xd6, 0xf4)
    }

    /// Subtle text (labels, separators) — Subtext0 `#a6adc8`
    pub fn text_subtle() -> Color {
        Color::Rgb(0xa6, 0xad, 0xc8)
    }

    /// Muted text (status, hints) — Overlay0 `#6c7086`
    pub fn text_muted() -> Color {
        Color::Rgb(0x6c, 0x70, 0x86)
    }

    /// Warning / attention — Yellow `#f9e2af`
    pub fn warning() -> Color {
        Color::Rgb(0xf9, 0xe2, 0xaf)
    }

    /// Surface / border colour — Surface1 `#45475a` (brightened for WCAG visibility)
    pub fn surface() -> Color {
        Color::Rgb(0x45, 0x47, 0x5a)
    }

    /// Success / completed — Green `#a6e3a1`
    pub fn success() -> Color {
        Color::Rgb(0xa6, 0xe3, 0xa1)
    }

    /// Error / failure — Red `#f38ba8`
    pub fn error() -> Color {
        Color::Rgb(0xf3, 0x8b, 0xa8)
    }

    /// Background / base colour — Base `#1e1e2e`
    pub fn background() -> Color {
        Color::Rgb(0x1e, 0x1e, 0x2e)
    }

    /// Processing / active state — Sky `#89dceb`
    pub fn processing() -> Color {
        Color::Rgb(0x89, 0xdc, 0xeb)
    }

    /// Highlight / cursor — Lavender `#b4befe`
    pub fn highlight() -> Color {
        Color::Rgb(0xb4, 0xbe, 0xfe)
    }

    /// Parse error / warning — Peach `#fab387`
    pub fn parse_error() -> Color {
        Color::Rgb(0xfa, 0xb3, 0x87)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Calculate relative luminance per WCAG 2.0
    fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
        let [r, g, b] = [r, g, b].map(|c| {
            let c = c as f64 / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        });
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Calculate contrast ratio between two colors
    fn contrast_ratio(rgb1: (u8, u8, u8), rgb2: (u8, u8, u8)) -> f64 {
        let l1 = relative_luminance(rgb1.0, rgb1.1, rgb1.2);
        let l2 = relative_luminance(rgb2.0, rgb2.1, rgb2.2);
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    fn color_to_rgb(c: Color) -> (u8, u8, u8) {
        match c {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => panic!("Test requires Rgb colors"),
        }
    }

    #[test]
    fn text_has_sufficient_contrast_against_background() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::text()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 4.5,
            "text vs background contrast {ratio:.2} < 4.5 (WCAG AA)"
        );
    }

    #[test]
    fn error_has_sufficient_contrast() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::error()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 4.5,
            "error vs background contrast {ratio:.2} < 4.5"
        );
    }

    #[test]
    fn success_has_sufficient_contrast() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::success()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 4.5,
            "success vs background contrast {ratio:.2} < 4.5"
        );
    }

    #[test]
    fn warning_has_sufficient_contrast() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::warning()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 4.5,
            "warning vs background contrast {ratio:.2} < 4.5"
        );
    }

    #[test]
    fn accent_has_sufficient_contrast() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::accent()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 4.5,
            "accent vs background contrast {ratio:.2} < 4.5"
        );
    }

    #[test]
    fn text_muted_has_minimum_contrast() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::text_muted()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 3.0,
            "text_muted vs background contrast {ratio:.2} < 3.0 (WCAG AA large text)"
        );
    }

    #[test]
    fn surface_has_distinct_border_contrast() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::surface()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 1.5,
            "surface vs background contrast {ratio:.2} < 1.5 (minimum visibility)"
        );
    }

    #[test]
    fn processing_has_sufficient_contrast() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::processing()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 4.5,
            "processing vs background contrast {ratio:.2} < 4.5"
        );
    }

    #[test]
    fn highlight_has_sufficient_contrast() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::highlight()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 4.5,
            "highlight vs background contrast {ratio:.2} < 4.5"
        );
    }

    #[test]
    fn parse_error_has_sufficient_contrast() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::parse_error()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 4.5,
            "parse_error vs background contrast {ratio:.2} < 4.5"
        );
    }

    #[test]
    fn text_subtle_has_minimum_contrast() {
        let ratio = contrast_ratio(
            color_to_rgb(Theme::text_subtle()),
            color_to_rgb(Theme::background()),
        );
        assert!(
            ratio >= 3.0,
            "text_subtle vs background contrast {ratio:.2} < 3.0 (WCAG AA large text)"
        );
    }
}
