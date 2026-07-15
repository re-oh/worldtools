use bevy_egui::egui::{self, Color32, Order};

use crate::{WorldLayer, style};

struct LegendSpec {
    subtitle: &'static str,
    entries: &'static [(&'static str, [u8; 3])],
}

pub fn show(ctx: &egui::Context, viewport: egui::Rect, layer: WorldLayer) {
    let spec = spec(layer);
    let position = egui::pos2(
        viewport.min.x + 8.0,
        (viewport.max.y - 138.0).max(viewport.min.y + 48.0),
    );
    egui::Area::new("worldtools_map_legend".into())
        .order(Order::Foreground)
        .fixed_pos(position)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(style::BG_PANEL.gamma_multiply(0.96))
                .stroke(egui::Stroke::new(1.0, style::BORDER))
                .shadow(egui::epaint::Shadow {
                    offset: [2, 3],
                    blur: 8,
                    spread: 0,
                    color: Color32::from_black_alpha(96),
                })
                .inner_margin(egui::Margin::symmetric(9, 7))
                .show(ui, |ui| {
                    ui.set_width(178.0);
                    ui.label(
                        egui::RichText::new(layer.label().to_uppercase())
                            .strong()
                            .size(10.0),
                    );
                    ui.label(
                        egui::RichText::new(spec.subtitle)
                            .small()
                            .color(style::TEXT_MUTED),
                    );
                    ui.add_space(4.0);
                    for &(label, rgb) in spec.entries {
                        ui.horizontal(|ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(15.0, 8.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                rect,
                                0.0,
                                Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
                            );
                            ui.painter().rect_stroke(
                                rect,
                                0.0,
                                egui::Stroke::new(1.0, style::BORDER_DARK),
                                egui::StrokeKind::Inside,
                            );
                            ui.label(egui::RichText::new(label).small());
                        });
                    }
                });
        });
}

fn spec(layer: WorldLayer) -> LegendSpec {
    match layer {
        WorldLayer::Elevation => LegendSpec {
            subtitle: "height relative to sea level",
            entries: &[
                ("deep ocean  < -4 km", [7, 23, 46]),
                ("lowland  0-450 m", [69, 122, 71]),
                ("highland  0.5-2.2 km", [120, 89, 64]),
                ("alpine  > 2.2 km", [224, 230, 224]),
            ],
        },
        WorldLayer::Tectonics => LegendSpec {
            subtitle: "plate domains and active margins",
            entries: &[
                ("convergent", [255, 72, 14]),
                ("divergent", [13, 179, 232]),
                ("stable interior", [54, 110, 128]),
                ("volcanic activity", [255, 184, 20]),
            ],
        },
        WorldLayer::Hydrology => LegendSpec {
            subtitle: "discharge, storage, and sediment",
            entries: &[
                ("major channel", [168, 237, 250]),
                ("river network", [6, 110, 179]),
                ("wetland / lake", [20, 87, 79]),
                ("sediment load", [158, 110, 48]),
            ],
        },
        WorldLayer::Climate => LegendSpec {
            subtitle: "annual temperature and precipitation",
            entries: &[
                ("cold / polar", [43, 107, 179]),
                ("temperate", [31, 150, 110]),
                ("hot / arid", [237, 148, 26]),
                ("humid", [14, 74, 122]),
            ],
        },
        WorldLayer::Soil => LegendSpec {
            subtitle: "soil order, depth, and fertility",
            entries: &[
                ("forest soil", [94, 59, 31]),
                ("alluvial", [148, 99, 41]),
                ("laterite", [163, 51, 19]),
                ("peat / organic", [26, 17, 12]),
            ],
        },
        WorldLayer::Vegetation => LegendSpec {
            subtitle: "biome and canopy structure",
            entries: &[
                ("forest", [25, 102, 46]),
                ("grassland", [138, 158, 46]),
                ("dryland", [209, 148, 56]),
                ("wetland", [20, 102, 87]),
            ],
        },
        WorldLayer::Geology => LegendSpec {
            subtitle: "surface lithology and crustal history",
            entries: &[
                ("igneous", [176, 92, 87]),
                ("sedimentary", [186, 125, 51]),
                ("metamorphic", [120, 54, 133]),
                ("unconsolidated", [163, 110, 46]),
            ],
        },
        WorldLayer::Resources => LegendSpec {
            subtitle: "deposit family; size = richness",
            entries: &[
                ("iron / bauxite", [209, 77, 32]),
                ("copper sulfides", [43, 158, 151]),
                ("fuel deposits", [61, 48, 63]),
                ("salts / industrial", [224, 211, 176]),
            ],
        },
    }
}
