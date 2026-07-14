use bevy_egui::egui::{self, Vec2};
use egui_phosphor_icons::{Icon, icons};

use crate::{
    ActiveTool, EditorUiState,
    style::{self, BG_PANEL, TOOL_RAIL_WIDTH},
    widgets,
};

pub fn show(root: &mut egui::Ui, state: &mut EditorUiState) {
    egui::Panel::left("worldtools_tool_rail")
        .exact_size(TOOL_RAIL_WIDTH)
        .resizable(false)
        .frame(style::panel_frame(BG_PANEL).inner_margin(egui::Margin::ZERO))
        .show(root, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            for tool in ActiveTool::ALL {
                let response = widgets::icon_button(
                    ui,
                    tool_icon(tool),
                    &format!("{}\n{}", tool.label(), tool.description()),
                    Vec2::new(TOOL_RAIL_WIDTH - 1.0, 39.0),
                    state.active_tool == tool,
                );
                if response.clicked() {
                    state.active_tool = tool;
                }
            }
        });
}

const fn tool_icon(tool: ActiveTool) -> Icon {
    match tool {
        ActiveTool::Navigate => icons::HAND,
        ActiveTool::Inspect => icons::EYEDROPPER,
    }
}
