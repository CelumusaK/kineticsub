use egui::{Color32, Rounding, Stroke, Style, Visuals};

// ── Palette ───────────────────────────────────────────────────────────────────

pub const BG_BASE:       Color32 = Color32::from_rgb(13,  15,  19);
pub const BG_PANEL:      Color32 = Color32::from_rgb(17,  19,  24);
pub const BG_PANEL_ALT:  Color32 = Color32::from_rgb(15,  17,  21);
pub const BG_HOVER:      Color32 = Color32::from_rgb(24,  27,  35);
pub const BG_SELECTED:   Color32 = Color32::from_rgb(30,  35,  48);

pub const BORDER:        Color32 = Color32::from_rgb(28,  32,  40);
pub const BORDER_FOCUS:  Color32 = Color32::from_rgb(50,  56,  70);

pub const TEXT_DIM:      Color32 = Color32::from_rgb(107, 114, 128);
pub const TEXT_NORM:     Color32 = Color32::from_rgb(156, 163, 175);
pub const TEXT_BRIGHT:   Color32 = Color32::from_rgb(220, 224, 230);

pub const ACCENT_CYAN:   Color32 = Color32::from_rgb(34,  211, 238);
pub const ACCENT_CYAN_DIM: Color32 = Color32::from_rgb(22, 140, 158);
pub const ACCENT_AMBER:  Color32 = Color32::from_rgb(245, 158,  11);

// ── egui style ────────────────────────────────────────────────────────────────

/// Apply the KineticSub dark theme to an egui `Style`.
pub fn apply(style: &mut Style) {
    style.visuals = dark_visuals();
    style.spacing.item_spacing    = egui::vec2(6.0, 4.0);
    style.spacing.button_padding  = egui::vec2(10.0, 5.0);
    style.spacing.window_margin   = egui::Margin::same(0.0);
    style.spacing.indent          = 14.0;
}

fn dark_visuals() -> Visuals {
    let mut v = Visuals::dark();

    v.override_text_color = Some(TEXT_NORM);
    v.window_fill         = BG_PANEL;
    v.panel_fill          = BG_PANEL;
    v.extreme_bg_color    = BG_BASE;
    v.code_bg_color       = BG_BASE;

    let border_stroke = Stroke::new(1.0, BORDER);

    v.window_stroke       = border_stroke;
    v.widgets.noninteractive.bg_fill   = BG_PANEL;
    v.widgets.noninteractive.bg_stroke = border_stroke;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    v.widgets.noninteractive.rounding  = Rounding::same(3.0);

    v.widgets.inactive.bg_fill   = BG_PANEL_ALT;
    v.widgets.inactive.bg_stroke = border_stroke;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_NORM);
    v.widgets.inactive.rounding  = Rounding::same(3.0);

    v.widgets.hovered.bg_fill   = BG_HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_FOCUS);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_BRIGHT);
    v.widgets.hovered.rounding  = Rounding::same(3.0);

    v.widgets.active.bg_fill   = BG_SELECTED;
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_CYAN);
    v.widgets.active.fg_stroke = Stroke::new(1.0, ACCENT_CYAN);
    v.widgets.active.rounding  = Rounding::same(3.0);

    v.selection.bg_fill  = ACCENT_CYAN.linear_multiply(0.2);
    v.selection.stroke   = Stroke::new(1.0, ACCENT_CYAN);

    v.hyperlink_color    = ACCENT_CYAN;
    v.faint_bg_color     = BG_BASE;

    v
}