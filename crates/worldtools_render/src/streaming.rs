use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use bevy::{prelude::*, tasks::AsyncComputeTaskPool, window::PrimaryWindow};
use crossbeam_channel::{Receiver, Sender, TryRecvError, bounded};
use moka::sync::Cache;
use worldtools_world::{
    EditId, EditJournal, EditJournalError, EditRevision, GeoPoint, TerrainEdit, TerrainGenerator,
    TerrainSettings, WorldSeed,
};

use crate::{
    debug::RenderDebugSettings,
    projection::{MapTileId, MapTilePlacement, TilePlan, plan_tiles},
    tile_data::MapTileData,
    view::{MapView, MapViewport},
};

const MAX_RESIDENT_TILES: u64 = 128;
const MAX_IN_FLIGHT: usize = 8;
const MAX_RESULTS_PER_FRAME: usize = 8;

#[derive(Clone, Debug, Default, Resource)]
pub struct VisibleMapTiles(pub TilePlan);

#[derive(Clone, Copy, Debug, Default, Resource)]
pub struct TileStreamStats {
    pub level: u8,
    pub visible: usize,
    pub resident_visible: usize,
    pub resident_total: u64,
    pub in_flight: usize,
    pub completed: u64,
    pub discarded: u64,
    pub requested: u64,
    pub invalidated: u64,
    pub last_generation_ms: f32,
    pub max_generation_ms: f32,
    pub resident_capacity: u64,
    pub max_in_flight: usize,
    pub ready_results: usize,
    pub edit_count: usize,
}

#[derive(Clone, Debug, Message)]
pub struct MapTileInvalidation {
    pub tiles: Vec<MapTileId>,
}

#[derive(Resource)]
pub struct MapTileStreamer {
    seed: WorldSeed,
    settings: TerrainSettings,
    edits: EditJournal,
    cache: Cache<MapTileId, Arc<MapTileData>>,
    in_flight: HashSet<MapTileId>,
    revisions: HashMap<MapTileId, u64>,
    sender: Sender<TileResult>,
    receiver: Receiver<TileResult>,
    completed: u64,
    discarded: u64,
    requested: u64,
    invalidated: u64,
    last_generation: Duration,
    max_generation: Duration,
    trace_streaming: bool,
}

impl Default for MapTileStreamer {
    fn default() -> Self {
        Self::new(WorldSeed(1), TerrainSettings::default())
    }
}

impl MapTileStreamer {
    #[must_use]
    pub fn new(seed: WorldSeed, settings: TerrainSettings) -> Self {
        let (sender, receiver) = bounded(MAX_IN_FLIGHT * 2);
        Self {
            seed,
            settings,
            edits: EditJournal::new(),
            cache: Cache::builder().max_capacity(MAX_RESIDENT_TILES).build(),
            in_flight: HashSet::new(),
            revisions: HashMap::new(),
            sender,
            receiver,
            completed: 0,
            discarded: 0,
            requested: 0,
            invalidated: 0,
            last_generation: Duration::ZERO,
            max_generation: Duration::ZERO,
            trace_streaming: false,
        }
    }

    #[must_use]
    pub fn get(&self, id: MapTileId) -> Option<Arc<MapTileData>> {
        self.cache.get(&id)
    }

    #[must_use]
    pub fn contains(&self, id: MapTileId) -> bool {
        self.cache.contains_key(&id)
    }

    /// Returns a stable, sorted view of cached page identifiers for diagnostics.
    #[must_use]
    pub fn resident_tile_ids(&self) -> Vec<MapTileId> {
        let mut ids = self.cache.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    /// Returns a stable, sorted view of page identifiers currently generating.
    #[must_use]
    pub fn in_flight_tile_ids(&self) -> Vec<MapTileId> {
        let mut ids = self.in_flight.iter().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    #[must_use]
    pub fn tile_revision(&self, id: MapTileId) -> u64 {
        self.revisions.get(&id).copied().unwrap_or_default()
    }

    pub fn set_trace_streaming(&mut self, enabled: bool) {
        self.trace_streaming = enabled;
    }

    #[must_use]
    pub fn best_available(&self, mut id: MapTileId) -> Option<Arc<MapTileData>> {
        loop {
            if let Some(tile) = self.get(id) {
                return Some(tile);
            }
            id = id.parent()?;
        }
    }

    pub fn allocate_edit_id(&mut self) -> EditId {
        self.edits.allocate_id()
    }

    /// Inserts an edit and invalidates only resident or running pages whose
    /// spherical bounds intersect it.
    ///
    /// # Errors
    /// Returns [`EditJournalError`] when the edit identifier already exists.
    pub fn insert_edit(
        &mut self,
        edit: TerrainEdit,
    ) -> Result<(EditRevision, usize), EditJournalError> {
        let affected = self.tiles_intersecting(&edit);
        let revision = self.edits.insert(edit)?;
        let count = self.invalidate_ids(affected);
        if self.trace_streaming {
            tracing::debug!(
                target: "worldtools_render::streaming",
                edit_revision = revision.0,
                affected_tiles = count,
                edit_count = self.edits.edits().len(),
                "terrain edit inserted"
            );
        }
        Ok((revision, count))
    }

    pub fn remove_edit(&mut self, id: EditId) -> Option<(TerrainEdit, usize)> {
        let edit = self.edits.remove(id)?;
        let affected = self.tiles_intersecting(&edit);
        let count = self.invalidate_ids(affected);
        if self.trace_streaming {
            tracing::debug!(
                target: "worldtools_render::streaming",
                affected_tiles = count,
                edit_count = self.edits.edits().len(),
                "terrain edit removed"
            );
        }
        Some((edit, count))
    }

    pub fn clear_edits(&mut self) -> usize {
        if self.edits.edits().is_empty() {
            return 0;
        }
        self.edits.clear();
        let ids = self
            .cache
            .iter()
            .map(|(id, _)| *id)
            .chain(self.in_flight.iter().copied())
            .collect::<HashSet<_>>();
        let count = self.invalidate_tiles(ids);
        if self.trace_streaming {
            tracing::debug!(
                target: "worldtools_render::streaming",
                affected_tiles = count,
                "terrain edits cleared"
            );
        }
        count
    }

    pub fn invalidate_tiles(&mut self, ids: impl IntoIterator<Item = MapTileId>) -> usize {
        self.invalidate_ids(ids.into_iter().collect::<HashSet<_>>())
    }

    pub fn invalidate_resident(&mut self) -> usize {
        let ids = self
            .cache
            .iter()
            .map(|(id, _)| *id)
            .chain(self.in_flight.iter().copied())
            .collect::<HashSet<_>>();
        self.invalidate_ids(ids)
    }

    #[must_use]
    pub fn sample_elevation(&self, point: GeoPoint) -> f32 {
        let base = TerrainGenerator::new(self.seed, self.settings).sample_geo(point);
        self.edits
            .apply_elevation(point.direction(), base, self.settings.planet_radius_m)
    }

    fn tiles_intersecting(&self, edit: &TerrainEdit) -> HashSet<MapTileId> {
        self.cache
            .iter()
            .map(|(id, _)| *id)
            .chain(self.in_flight.iter().copied())
            .filter(|id| {
                let (center, radius) = id.bounding_cap();
                edit.might_affect_cap(center.direction(), radius, self.settings.planet_radius_m)
            })
            .collect()
    }

    fn invalidate_ids(&mut self, ids: impl IntoIterator<Item = MapTileId>) -> usize {
        let mut count = 0;
        for id in ids {
            self.cache.invalidate(&id);
            let revision = self.revisions.entry(id).or_default();
            *revision = revision.wrapping_add(1);
            count += 1;
            if self.trace_streaming {
                tracing::debug!(
                    target: "worldtools_render::streaming",
                    level = id.level,
                    x = id.x,
                    y = id.y,
                    revision = *revision,
                    "tile invalidated"
                );
            }
        }
        self.invalidated = self
            .invalidated
            .saturating_add(u64::try_from(count).unwrap_or(u64::MAX));
        count
    }

    fn request(&mut self, id: MapTileId) {
        if self.cache.contains_key(&id) || !self.in_flight.insert(id) {
            return;
        }
        let revision = self.revisions.get(&id).copied().unwrap_or_default();
        let seed = self.seed;
        let settings = self.settings;
        let edits = self.edits.clone();
        let sender = self.sender.clone();
        let trace_streaming = self.trace_streaming;
        self.requested = self.requested.saturating_add(1);
        if trace_streaming {
            tracing::debug!(
                target: "worldtools_render::streaming",
                level = id.level,
                x = id.x,
                y = id.y,
                revision,
                in_flight = self.in_flight.len(),
                "tile generation requested"
            );
        }
        AsyncComputeTaskPool::get()
            .spawn(async move {
                let started = Instant::now();
                let span = tracing::debug_span!(
                    target: "worldtools_render::generation",
                    "generate_map_tile",
                    level = id.level,
                    x = id.x,
                    y = id.y,
                    revision
                );
                let _guard = trace_streaming.then(|| span.enter());
                let data = Arc::new(MapTileData::generate_with_edits(id, seed, settings, &edits));
                let generated_in = started.elapsed();
                if trace_streaming {
                    tracing::debug!(
                        target: "worldtools_render::generation",
                        duration_ms = generated_in.as_secs_f64() * 1_000.0,
                        "tile generation completed"
                    );
                }
                if sender
                    .send(TileResult {
                        id,
                        revision,
                        data,
                        generated_in,
                    })
                    .is_err()
                {
                    tracing::warn!(
                        target: "worldtools_render::streaming",
                        level = id.level,
                        x = id.x,
                        y = id.y,
                        "tile result receiver disconnected"
                    );
                }
            })
            .detach();
    }
}

#[derive(Debug)]
struct TileResult {
    id: MapTileId,
    revision: u64,
    data: Arc<MapTileData>,
    generated_in: Duration,
}

pub(crate) struct TileStreamingPlugin;

impl Plugin for TileStreamingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapTileStreamer>()
            .init_resource::<VisibleMapTiles>()
            .init_resource::<TileStreamStats>()
            .add_message::<MapTileInvalidation>()
            .add_systems(
                Update,
                (update_plan, receive_tiles, invalidate_tiles, request_tiles).chain(),
            );
    }
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn update_plan(
    view: Res<MapView>,
    viewport: Res<MapViewport>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut visible: ResMut<VisibleMapTiles>,
    debug: Res<RenderDebugSettings>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let fallback = Vec2::new(window.width(), window.height());
    let next = plan_tiles(*view, viewport.physical_size(fallback));
    if debug.trace_streaming && visible.0 != next {
        tracing::debug!(
            target: "worldtools_render::planning",
            level = next.level,
            visible = next.placements.len(),
            viewport_width = viewport.physical_size(fallback).x,
            viewport_height = viewport.physical_size(fallback).y,
            "visible tile plan changed"
        );
    }
    visible.0 = next;
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn receive_tiles(mut streamer: ResMut<MapTileStreamer>, debug: Res<RenderDebugSettings>) {
    streamer.set_trace_streaming(debug.trace_streaming);
    for _ in 0..MAX_RESULTS_PER_FRAME {
        match streamer.receiver.try_recv() {
            Ok(result) => {
                streamer.in_flight.remove(&result.id);
                let current_revision = streamer
                    .revisions
                    .get(&result.id)
                    .copied()
                    .unwrap_or_default();
                if result.revision == current_revision {
                    streamer.last_generation = result.generated_in;
                    streamer.max_generation = streamer.max_generation.max(result.generated_in);
                    if streamer.trace_streaming {
                        tracing::debug!(
                            target: "worldtools_render::streaming",
                            level = result.id.level,
                            x = result.id.x,
                            y = result.id.y,
                            revision = result.revision,
                            duration_ms = result.generated_in.as_secs_f64() * 1_000.0,
                            "tile result accepted"
                        );
                    }
                    streamer.cache.insert(result.id, result.data);
                    streamer.completed += 1;
                } else {
                    if streamer.trace_streaming {
                        tracing::debug!(
                            target: "worldtools_render::streaming",
                            level = result.id.level,
                            x = result.id.x,
                            y = result.id.y,
                            result_revision = result.revision,
                            current_revision,
                            "stale tile result discarded"
                        );
                    }
                    streamer.discarded += 1;
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                tracing::error!(
                    target: "worldtools_render::streaming",
                    "tile generation result channel disconnected"
                );
                break;
            }
        }
    }
}

fn invalidate_tiles(
    mut invalidations: MessageReader<MapTileInvalidation>,
    mut streamer: ResMut<MapTileStreamer>,
) {
    for invalidation in invalidations.read() {
        for &id in &invalidation.tiles {
            streamer.invalidate_ids([id]);
        }
    }
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn request_tiles(
    visible: Res<VisibleMapTiles>,
    settings: Res<RenderDebugSettings>,
    mut streamer: ResMut<MapTileStreamer>,
    mut stats: ResMut<TileStreamStats>,
    mut was_frozen: Local<bool>,
) {
    if *was_frozen != settings.freeze_streaming {
        if settings.trace_streaming {
            tracing::debug!(
                target: "worldtools_render::streaming",
                frozen = settings.freeze_streaming,
                in_flight = streamer.in_flight.len(),
                "tile request scheduler state changed"
            );
        }
        *was_frozen = settings.freeze_streaming;
    }
    let available = request_capacity(settings.freeze_streaming, streamer.in_flight.len());
    if available > 0 {
        let candidates = request_priority(&visible.0);
        for id in candidates
            .into_iter()
            .filter(|id| !streamer.cache.contains_key(id) && !streamer.in_flight.contains(id))
            .take(available)
            .collect::<Vec<_>>()
        {
            streamer.request(id);
        }
    }

    stats.level = visible.0.level;
    stats.visible = visible.0.placements.len();
    stats.resident_visible = visible
        .0
        .placements
        .iter()
        .filter(|placement| streamer.best_available(placement.id).is_some())
        .count();
    stats.resident_total = streamer.cache.entry_count();
    stats.in_flight = streamer.in_flight.len();
    stats.completed = streamer.completed;
    stats.discarded = streamer.discarded;
    stats.requested = streamer.requested;
    stats.invalidated = streamer.invalidated;
    stats.last_generation_ms = streamer.last_generation.as_secs_f32() * 1_000.0;
    stats.max_generation_ms = streamer.max_generation.as_secs_f32() * 1_000.0;
    stats.resident_capacity = MAX_RESIDENT_TILES;
    stats.max_in_flight = MAX_IN_FLIGHT;
    stats.ready_results = streamer.receiver.len();
    stats.edit_count = streamer.edits.edits().len();
}

fn request_capacity(frozen: bool, in_flight: usize) -> usize {
    if frozen {
        0
    } else {
        MAX_IN_FLIGHT.saturating_sub(in_flight)
    }
}

fn request_priority(plan: &TilePlan) -> Vec<MapTileId> {
    let mut roots = Vec::new();
    let mut desired = Vec::new();
    let mut fallback = Vec::new();
    let mut seen = HashSet::new();

    for MapTilePlacement { id, .. } in &plan.placements {
        let mut ancestors = Vec::new();
        let mut cursor = *id;
        while let Some(parent) = cursor.parent() {
            ancestors.push(parent);
            cursor = parent;
        }
        if let Some(root) = ancestors.last().copied()
            && seen.insert(root)
        {
            roots.push(root);
        }
        if seen.insert(*id) {
            desired.push(*id);
        }
        for ancestor in ancestors.into_iter().rev().skip(1).rev() {
            if seen.insert(ancestor) {
                fallback.push(ancestor);
            }
        }
    }
    roots.extend(desired);
    roots.extend(fallback);
    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_and_desired_tiles_are_requested_before_intermediate_fallbacks() {
        let desired = MapTileId {
            level: 4,
            x: 11,
            y: 6,
        };
        let priority = request_priority(&TilePlan {
            level: 4,
            placements: vec![MapTilePlacement {
                id: desired,
                unwrapped_x: 11,
            }],
        });
        assert_eq!(priority[0].level, 0);
        assert_eq!(priority[1], desired);
        assert!(priority[2..].iter().all(|tile| tile.level < desired.level));
    }

    #[test]
    fn diagnostic_tile_lists_are_stable_and_revisions_are_visible() {
        let mut streamer = MapTileStreamer::default();
        let high = MapTileId {
            level: 2,
            x: 2,
            y: 1,
        };
        let low = MapTileId {
            level: 0,
            x: 0,
            y: 0,
        };
        streamer.cache.insert(
            high,
            Arc::new(MapTileData::generate(
                high,
                WorldSeed(1),
                TerrainSettings::default(),
            )),
        );
        streamer.cache.insert(
            low,
            Arc::new(MapTileData::generate(
                low,
                WorldSeed(1),
                TerrainSettings::default(),
            )),
        );

        assert_eq!(streamer.resident_tile_ids(), vec![low, high]);
        assert_eq!(streamer.tile_revision(high), 0);
        streamer.invalidate_tiles([high]);
        assert_eq!(streamer.tile_revision(high), 1);
    }

    #[test]
    fn frozen_scheduler_stops_new_work_without_changing_the_limit() {
        assert_eq!(request_capacity(true, 0), 0);
        assert_eq!(request_capacity(false, 3), MAX_IN_FLIGHT - 3);
        assert_eq!(request_capacity(false, MAX_IN_FLIGHT + 1), 0);
    }
}
