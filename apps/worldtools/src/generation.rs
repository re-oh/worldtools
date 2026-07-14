use std::{sync::Arc, time::Instant};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};
use worldtools_render::MapTileStreamer;
use worldtools_simulation::WorldSnapshot;
use worldtools_ui::{
    DocumentStatus, GenerationActivity, GenerationStatus, MapProbe, PipelineStage, RegenerateWorld,
    WorldGenerationDraft,
};
use worldtools_world::WorldSeed;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenerationCoordinator>()
            .add_systems(Update, (queue_regeneration, finish_regeneration).chain());
    }
}

#[derive(Resource, Default)]
struct GenerationCoordinator {
    task: Option<Task<GeneratedWorld>>,
}

struct GeneratedWorld {
    snapshot: Arc<WorldSnapshot>,
    elapsed_ms: f64,
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn queue_regeneration(
    mut requests: MessageReader<RegenerateWorld>,
    streamer: Res<MapTileStreamer>,
    mut coordinator: ResMut<GenerationCoordinator>,
    mut status: ResMut<GenerationStatus>,
) {
    let Some(request) = requests.read().last().copied() else {
        return;
    };
    if coordinator.task.is_some() {
        tracing::warn!(
            target: "worldtools::generation",
            requested_seed = request.seed,
            "ignored regeneration request while world history is running"
        );
        return;
    }

    let terrain = streamer.snapshot().terrain_settings();
    let simulation = streamer.snapshot().simulation_settings();
    let seed = WorldSeed(request.seed);
    status.activity = GenerationActivity::Running {
        stage: PipelineStage::Tectonics,
        completed: 0,
        total: 1,
    };
    tracing::info!(
        target: "worldtools::generation",
        seed = request.seed,
        atlas_width = simulation.atlas_width,
        atlas_height = simulation.atlas_height,
        "world-history regeneration started"
    );
    coordinator.task = Some(AsyncComputeTaskPool::get().spawn(async move {
        let started = Instant::now();
        let snapshot = Arc::new(WorldSnapshot::generate(seed, terrain, simulation));
        GeneratedWorld {
            snapshot,
            elapsed_ms: started.elapsed().as_secs_f64() * 1_000.0,
        }
    }));
}

#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
fn finish_regeneration(
    mut coordinator: ResMut<GenerationCoordinator>,
    mut streamer: ResMut<MapTileStreamer>,
    mut document: ResMut<DocumentStatus>,
    mut draft: ResMut<WorldGenerationDraft>,
    mut status: ResMut<GenerationStatus>,
    mut probe: ResMut<MapProbe>,
) {
    let Some(task) = coordinator.task.as_mut() else {
        return;
    };
    let Some(generated) = block_on(poll_once(task)) else {
        return;
    };
    coordinator.task = None;

    let seed = generated.snapshot.seed().0;
    let revision = generated.snapshot.revision();
    streamer.replace_snapshot(generated.snapshot);
    document.seed = seed;
    draft.seed = seed;
    probe.selected = None;
    status.activity = GenerationActivity::Idle;
    tracing::info!(
        target: "worldtools::generation",
        seed,
        revision,
        world_epoch = streamer.world_epoch(),
        duration_ms = generated.elapsed_ms,
        "world-history regeneration installed"
    );
}
