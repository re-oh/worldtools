use bevy_egui::egui::{self, Color32, FontFamily, FontId, Stroke, TextStyle, Vec2, epaint::Shadow};

pub const TOP_BAR_HEIGHT: f32 = 32.0;
pub const TOOL_RAIL_WIDTH: f32 = 40.0;
pub const EXPLORER_WIDTH: f32 = 260.0;
pub const INSPECTOR_WIDTH: f32 = 320.0;
pub const DRAWER_HEADER_HEIGHT: f32 = 26.0;
pub const DRAWER_OPEN_HEIGHT: f32 = 184.0;
pub const STATUS_BAR_HEIGHT: f32 = 24.0;

pub const BG_MAP: Color32 = Color32::from_rgb(18, 19, 20);
pub const BG_PANEL: Color32 = Color32::from_rgb(29, 30, 31);
pub const BG_HEADER: Color32 = Color32::from_rgb(36, 38, 40);
pub const BG_HOVER: Color32 = Color32::from_rgb(47, 50, 52);
pub const BG_ACTIVE: Color32 = Color32::from_rgb(57, 61, 63);
pub const BORDER: Color32 = Color32::from_rgb(67, 69, 70);
pub const BORDER_DARK: Color32 = Color32::from_rgb(12, 13, 14);
pub const TEXT: Color32 = Color32::from_rgb(216, 214, 207);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(145, 145, 140);
pub const ACCENT: Color32 = Color32::from_rgb(116, 174, 184);
pub const VALID: Color32 = Color32::from_rgb(127, 182, 157);
pub const WARNING: Color32 = Color32::from_rgb(214, 163, 95);
pub const ERROR: Color32 = Color32::from_rgb(204, 105, 101);

pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor_icons::add_fonts(&mut fonts);
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        if let Some(family_fonts) = fonts.families.get_mut(&family) {
            family_fonts.push("phosphor-icons".to_owned());
        }
    }
    ctx.set_fonts(fonts);

    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.text_styles = [
        (TextStyle::Small, FontId::new(10.0, FontFamily::Monospace)),
        (TextStyle::Body, FontId::new(12.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(12.0, FontFamily::Monospace)),
        (TextStyle::Heading, FontId::new(13.0, FontFamily::Monospace)),
        (
            TextStyle::Monospace,
            FontId::new(12.0, FontFamily::Monospace),
        ),
    ]
    .into();

    style.spacing.item_spacing = Vec2::new(6.0, 4.0);
    style.spacing.button_padding = Vec2::new(5.0, 2.0);
    style.spacing.indent = 12.0;
    style.spacing.interact_size = Vec2::new(20.0, 20.0);
    style.spacing.slider_width = 108.0;
    style.spacing.combo_width = 120.0;
    style.spacing.menu_width = 156.0;

    let visuals = &mut style.visuals;
    visuals.dark_mode = true;
    visuals.override_text_color = Some(TEXT);
    visuals.panel_fill = BG_PANEL;
    visuals.window_fill = BG_PANEL;
    visuals.extreme_bg_color = BG_MAP;
    visuals.faint_bg_color = Color32::from_rgb(33, 34, 35);
    visuals.code_bg_color = BG_MAP;
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.weak_bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.bg_fill = BG_HEADER;
    visuals.widgets.inactive.weak_bg_fill = BG_HEADER;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.weak_bg_fill = BG_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.bg_fill = BG_ACTIVE;
    visuals.widgets.active.weak_bg_fill = BG_ACTIVE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.open.bg_fill = BG_ACTIVE;
    visuals.widgets.open.weak_bg_fill = BG_ACTIVE;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.selection.bg_fill = Color32::from_rgb(47, 84, 89);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.hyperlink_color = ACCENT;
    visuals.warn_fg_color = WARNING;
    visuals.error_fg_color = ERROR;
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;
    visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::ZERO;
    }

    ctx.set_style_of(egui::Theme::Dark, style);
}

pub fn panel_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, BORDER_DARK))
        .inner_margin(egui::Margin::same(5))
}

#[must_use]
pub fn icon_text(icon: egui_phosphor_icons::Icon, size: f32, color: Color32) -> egui::RichText {
    egui::RichText::new(icon.as_str())
        .family(FontFamily::Proportional)
        .size(size)
        .color(color)
}
