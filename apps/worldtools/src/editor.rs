mod probe;

use bevy::{input::ButtonInput, prelude::*, window::PrimaryWindow};
use worldtools_render::{MapTileStreamer, MapView, TileStreamStats, VisibleMapTiles};
use worldtools_ui::{
    ActiveTool, BrushFalloff as UiFalloff, BrushOperation, DirtyRegion, DocumentStatus,
    EditorCommand, EditorUiState, GenerationActivity, GenerationScope, GenerationStatus,
    LayerAvailability, LayerCapabilities, MapProbe, MapReadout, MapViewport as UiViewport,
    PipelineStage, SaveState, WorldLayer,
};
use worldtools_world::{BrushFalloff, EditOperation, GeoPoint, TerrainEdit, angular_distance};

const PLANET_RADIUS_M: f64 = 6_371_000.0;
const SCULPT_AMOUNT_M: f32 = 500.0;

#[derive(Default, Resource)]
struct EditorSession {
    stroke: Vec<GeoPoint>,
    committed: Vec<TerrainEdit>,
    undone: Vec<TerrainEdit>,
}

pub struct WorldEditorPlugin;

impl Plugin for WorldEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorSession>()
            .add_systems(Startup, publish_layer_capabilities)
            .add_systems(
                Update,
                (
                    handle_commands,
                    probe::capture_inspection,
                    capture_sculpt_stroke,
                    sync_telemetry,
                )
                    .chain(),
            );
    }
}

fn publish_layer_capabilities(mut capabilities: ResMut<LayerCapabilities>) {
    capabilities.set_all(LayerAvailability::Available);
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)] // Bevy system parameters are value wrappers.
fn handle_commands(
    mut commands: MessageReader<EditorCommand>,
    mut streamer: ResMut<MapTileStreamer>,
    visible: Res<VisibleMapTiles>,
    mut session: ResMut<EditorSession>,
    mut document: ResMut<DocumentStatus>,
    mut generation: ResMut<GenerationStatus>,
    mut probe: ResMut<MapProbe>,
) {
    for command in commands.read() {
        let invalidates_probe = matches!(
            command,
            EditorCommand::Undo
                | EditorCommand::Redo
                | EditorCommand::NewWorld
                | EditorCommand::ClearLayerEdits(WorldLayer::Elevation)
                | EditorCommand::Generate(_)
        );
        let affected = match command {
            EditorCommand::Undo => undo(&mut session, &mut streamer),
            EditorCommand::Redo => redo(&mut session, &mut streamer),
            EditorCommand::NewWorld => {
                session.committed.clear();
                session.undone.clear();
                document.save_state = SaveState::Saved;
                streamer.clear_edits()
            }
            EditorCommand::ClearLayerEdits(WorldLayer::Elevation) => {
                session.committed.clear();
                session.undone.clear();
                streamer.clear_edits()
            }
            EditorCommand::Generate(GenerationScope::World) => streamer.invalidate_resident(),
            EditorCommand::Generate(GenerationScope::Dirty | GenerationScope::Visible) => {
                streamer.invalidate_tiles(visible.0.placements.iter().map(|placement| placement.id))
            }
            _ => 0,
        };
        if affected > 0 {
            mark_dirty(&mut generation, affected);
        }
        if invalidates_probe {
            probe.selected = None;
        }
        update_history_status(&session, &mut document);
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn capture_sculpt_stroke(
    buttons: Res<ButtonInput<MouseButton>>,
    ui: Res<EditorUiState>,
    viewport: Res<UiViewport>,
    view: Res<MapView>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut streamer: ResMut<MapTileStreamer>,
    mut session: ResMut<EditorSession>,
    mut document: ResMut<DocumentStatus>,
    mut generation: ResMut<GenerationStatus>,
    mut probe: ResMut<MapProbe>,
) {
    if ui.active_tool != ActiveTool::Sculpt {
        session.stroke.clear();
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let point = (!viewport.input_blocked)
        .then(|| pointer_geo(window, &viewport, *view))
        .flatten();

    if buttons.just_pressed(MouseButton::Left) {
        session.stroke.clear();
        if let Some(point) = point {
            session.stroke.push(point);
        }
    } else if buttons.pressed(MouseButton::Left)
        && let Some(point) = point
        && should_append(&session.stroke, point, ui.brush.radius_m, ui.brush.spacing)
    {
        session.stroke.push(point);
    }

    if !buttons.just_released(MouseButton::Left) || session.stroke.is_empty() {
        return;
    }
    let path = std::mem::take(&mut session.stroke);
    let operation = edit_operation(ui.brush.operation, path[0], &streamer);
    let id = streamer.allocate_edit_id();
    let edit = TerrainEdit::new(
        id,
        path,
        f64::from(ui.brush.radius_m),
        ui.brush.strength,
        edit_falloff(ui.brush.falloff),
        operation,
    );
    let Ok(edit) = edit else {
        warn!("ignored invalid sculpt stroke");
        return;
    };
    match streamer.insert_edit(edit.clone()) {
        Ok((_, affected)) => {
            session.committed.push(edit);
            session.undone.clear();
            document.save_state = SaveState::Modified;
            update_history_status(&session, &mut document);
            mark_dirty(&mut generation, affected);
            probe.selected = None;
        }
        Err(error) => error!(%error, "failed to commit sculpt stroke"),
    }
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn sync_telemetry(
    stats: Res<TileStreamStats>,
    view: Res<MapView>,
    viewport: Res<UiViewport>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut readout: ResMut<MapReadout>,
    mut generation: ResMut<GenerationStatus>,
) {
    let height = viewport.physical.height().max(1.0);
    readout.lod = stats.level;
    readout.meters_per_pixel =
        std::f64::consts::PI * PLANET_RADIUS_M * f64::from(view.vertical_span) / f64::from(height);
    readout.cursor_degrees = windows
        .single()
        .ok()
        .and_then(|window| pointer_geo(window, &viewport, *view))
        .map(|point| [point.longitude.to_degrees(), point.latitude.to_degrees()]);

    generation.activity = if stats.in_flight == 0 {
        generation.dirty = DirtyRegion::default();
        GenerationActivity::Idle
    } else {
        GenerationActivity::Running {
            stage: PipelineStage::Surface,
            completed: usize_to_u32(stats.resident_visible),
            total: usize_to_u32(stats.visible),
        }
    };
}

fn pointer_geo(window: &Window, viewport: &UiViewport, view: MapView) -> Option<GeoPoint> {
    let pointer = window.cursor_position()?;
    let logical = viewport.window_logical(window.scale_factor());
    let min = Vec2::from_array(logical.min);
    let size = Vec2::new(logical.width(), logical.height());
    let local = pointer - min;
    if size.min_element() <= 1.0 || !local.cmpge(Vec2::ZERO).all() || !local.cmple(size).all() {
        return None;
    }
    let aspect = size.x / size.y;
    let normalized = local / size - Vec2::splat(0.5);
    let world_x =
        (view.center.x + f64::from(normalized.x * view.horizontal_span(aspect))).rem_euclid(1.0);
    let world_y = (view.center.y + f64::from(normalized.y * view.vertical_span)).clamp(0.0, 1.0);
    Some(GeoPoint::from_degrees(
        90.0 - 180.0 * world_y,
        -180.0 + 360.0 * world_x,
    ))
}

fn should_append(path: &[GeoPoint], point: GeoPoint, radius_m: f32, spacing: f32) -> bool {
    path.last().is_none_or(|last| {
        angular_distance(last.direction(), point.direction()) * PLANET_RADIUS_M
            >= f64::from(radius_m * spacing)
    })
}

fn edit_operation(
    operation: BrushOperation,
    anchor: GeoPoint,
    streamer: &MapTileStreamer,
) -> EditOperation {
    match operation {
        BrushOperation::Raise => EditOperation::AddElevation {
            amount_m: SCULPT_AMOUNT_M,
        },
        BrushOperation::Lower => EditOperation::AddElevation {
            amount_m: -SCULPT_AMOUNT_M,
        },
        BrushOperation::Flatten | BrushOperation::Replace => EditOperation::SetElevation {
            elevation_m: streamer.sample_elevation(anchor),
        },
    }
}

const fn edit_falloff(falloff: UiFalloff) -> BrushFalloff {
    match falloff {
        UiFalloff::Hard => BrushFalloff::Hard,
        UiFalloff::Linear => BrushFalloff::Linear,
        UiFalloff::Smooth | UiFalloff::Gaussian => BrushFalloff::Smooth,
    }
}

fn undo(session: &mut EditorSession, streamer: &mut MapTileStreamer) -> usize {
    let Some(edit) = session.committed.pop() else {
        return 0;
    };
    let affected = streamer.remove_edit(edit.id).map_or(0, |(_, count)| count);
    session.undone.push(edit);
    affected
}

fn redo(session: &mut EditorSession, streamer: &mut MapTileStreamer) -> usize {
    let Some(edit) = session.undone.pop() else {
        return 0;
    };
    match streamer.insert_edit(edit.clone()) {
        Ok((_, affected)) => {
            session.committed.push(edit);
            affected
        }
        Err(error) => {
            error!(%error, "failed to redo sculpt stroke");
            session.undone.push(edit);
            0
        }
    }
}

fn update_history_status(session: &EditorSession, document: &mut DocumentStatus) {
    document.can_undo = !session.committed.is_empty();
    document.can_redo = !session.undone.is_empty();
}

fn mark_dirty(generation: &mut GenerationStatus, affected: usize) {
    generation.dirty.tile_count = usize_to_u32(affected);
    generation.dirty.from_stage = Some(PipelineStage::BaseShape);
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroke_spacing_is_measured_on_the_sphere() {
        let path = [GeoPoint::from_degrees(0.0, 0.0)];
        assert!(!should_append(
            &path,
            GeoPoint::from_degrees(0.0, 0.01),
            10_000.0,
            0.2,
        ));
        assert!(should_append(
            &path,
            GeoPoint::from_degrees(0.0, 1.0),
            10_000.0,
            0.2,
        ));
    }
}
