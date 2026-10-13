use egui::{Color32, Margin, Rounding, Stroke, Style, Visuals, vec2};

pub const BG_BASE:         Color32 = Color32::from_rgb(12, 14, 18);
pub const BG_PANEL:        Color32 = Color32::from_rgb(18, 20, 26);
pub const BG_PANEL_ALT:    Color32 = Color32::from_rgb(15, 17, 22);
pub const BG_HOVER:        Color32 = Color32::from_rgb(26, 30, 40);
pub const BG_SELECTED:     Color32 = Color32::from_rgb(32, 38, 52);

pub const BORDER:          Color32 = Color32::from_rgb(30, 34, 44);
pub const BORDER_FOCUS:    Color32 = Color32::from_rgb(52, 60, 78);

pub const TEXT_MUTED:      Color32 = Color32::from_rgb(86, 94, 108);
pub const TEXT_DIM:        Color32 = Color32::from_rgb(130, 138, 153);
pub const TEXT_NORM:       Color32 = Color32::from_rgb(188, 194, 206);
pub const TEXT_BRIGHT:     Color32 = Color32::from_rgb(240, 243, 248);

pub const ACCENT_CYAN:     Color32 = Color32::from_rgb(45, 185, 240);
pub const ACCENT_CYAN_DIM: Color32 = Color32::from_rgb(24, 105, 142);
pub const ACCENT_AMBER:    Color32 = Color32::from_rgb(245, 158, 11);
pub const ACCENT_EMERALD:  Color32 = Color32::from_rgb(34, 197, 94);

pub fn apply(style: &mut Style) {
    style.visuals = dark_visuals();
    style.spacing.item_spacing   = vec2(8.0, 5.0);
    style.spacing.button_padding = vec2(10.0, 5.0);
    style.spacing.window_margin  = Margin::same(0.0);
    style.spacing.indent         = 12.0;
}

fn dark_visuals() -> Visuals {
    let mut v = Visuals::dark();

    v.override_text_color = Some(TEXT_NORM);
    v.window_fill         = BG_PANEL;
    v.panel_fill          = BG_PANEL;
    v.extreme_bg_color    = BG_BASE;
    v.code_bg_color       = BG_BASE;

    let border_stroke = Stroke::new(1.0, BORDER);
    let border_focus  = Stroke::new(1.0, BORDER_FOCUS);

    v.window_stroke       = border_stroke;
    v.window_rounding     = Rounding::same(8.0);

    v.widgets.noninteractive.bg_fill   = BG_PANEL;
    v.widgets.noninteractive.bg_stroke = border_stroke;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    v.widgets.noninteractive.rounding  = Rounding::same(5.0);

    v.widgets.inactive.bg_fill   = BG_PANEL_ALT;
    v.widgets.inactive.bg_stroke = border_stroke;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_NORM);
    v.widgets.inactive.rounding  = Rounding::same(5.0);

    v.widgets.hovered.bg_fill   = BG_HOVER;
    v.widgets.hovered.bg_stroke = border_focus;
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_BRIGHT);
    v.widgets.hovered.rounding  = Rounding::same(5.0);

    v.widgets.active.bg_fill   = BG_SELECTED;
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_CYAN);
    v.widgets.active.fg_stroke = Stroke::new(1.0, ACCENT_CYAN);
    v.widgets.active.rounding  = Rounding::same(5.0);

    v.selection.bg_fill  = Color32::from_rgba_unmultiplied(45, 185, 240, 45);
    v.selection.stroke   = Stroke::new(1.0, ACCENT_CYAN);

    v.hyperlink_color = ACCENT_CYAN;
    v.faint_bg_color  = BG_BASE;

    v
}