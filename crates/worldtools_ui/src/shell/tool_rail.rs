use bevy::prelude::MessageWriter;
use bevy_egui::egui::{self, Vec2};
use egui_phosphor_icons::{Icon, icons};

use crate::{
    ActiveTool, EditorCommand, EditorUiState,
    style::{self, BG_PANEL, TOOL_RAIL_WIDTH},
    widgets,
};

pub fn show(
    root: &mut egui::Ui,
    state: &mut EditorUiState,
    commands: &mut MessageWriter<EditorCommand>,
) {
    egui::Panel::left("worldtools_tool_rail")
        .exact_size(TOOL_RAIL_WIDTH)
        .resizable(false)
        .frame(style::panel_frame(BG_PANEL).inner_margin(egui::Margin::ZERO))
        .show(root, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            for tool in ActiveTool::PRIMARY {
                let response = widgets::icon_button(
                    ui,
                    tool_icon(tool),
                    &format!("{}\n{}", tool.label(), tool.description()),
                    Vec2::new(TOOL_RAIL_WIDTH - 1.0, 39.0),
                    state.active_tool == tool,
                );
                if response.clicked() && state.active_tool != tool {
                    state.active_tool = tool;
                    commands.write(EditorCommand::SelectTool(tool));
                }
            }
        });
}

fn tool_icon(tool: ActiveTool) -> Icon {
    match tool {
        ActiveTool::Navigate => icons::HAND,
        ActiveTool::Inspect => icons::EYEDROPPER,
        ActiveTool::Sculpt => icons::MOUNTAINS,
        ActiveTool::Smooth => icons::WAVES,
        ActiveTool::RiverGuide => icons::DROP,
        ActiveTool::PaintConstraint => icons::PAINT_BRUSH,
    }
}
