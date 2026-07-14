use std::collections::VecDeque;

use bevy::prelude::{Message, Resource};

use super::WorldLayer;

/// Rendering diagnostics which can be applied without changing world data.
#[allow(clippy::struct_excessive_bools)] // Independent overlay toggles are clearer than an artificial state machine.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DebugRenderOptions {
    pub tile_borders: bool,
    pub lod_tint: bool,
    pub fallback_tint: bool,
    pub trace_streaming: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DebugTab {
    #[default]
    Summary,
    Streaming,
    Viewport,
    Layers,
    Events,
}

impl DebugTab {
    pub const ALL: [Self; 5] = [
        Self::Summary,
        Self::Streaming,
        Self::Viewport,
        Self::Layers,
        Self::Events,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Streaming => "Streaming",
            Self::Viewport => "Viewport",
            Self::Layers => "Layers",
            Self::Events => "Events",
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct DebugUiState {
    pub visible: bool,
    pub selected_tab: DebugTab,
    pub render_options: DebugRenderOptions,
    pub freeze_streaming: bool,
    pub event_filter: String,
    pub follow_events: bool,
}

impl Default for DebugUiState {
    fn default() -> Self {
        Self {
            visible: false,
            selected_tab: DebugTab::Summary,
            render_options: DebugRenderOptions::default(),
            freeze_streaming: false,
            event_filter: String::new(),
            follow_events: true,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FrameDiagnostics {
    pub frames_per_second: f32,
    pub frame_time_ms: f32,
    pub frame_number: u64,
    pub entity_count: u64,
    pub process_cpu_percent: Option<f32>,
    pub process_memory_gib: Option<f32>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct StreamingDiagnostics {
    pub level: u8,
    pub visible_tiles: usize,
    pub resident_visible_tiles: usize,
    pub resident_total_tiles: u64,
    pub in_flight_tiles: usize,
    pub completed_jobs: u64,
    pub discarded_jobs: u64,
    pub requested_jobs: u64,
    pub invalidated_tiles: u64,
    pub last_generation_ms: f32,
    pub max_generation_ms: f32,
    pub resident_capacity: u64,
    pub max_in_flight: usize,
    pub ready_results: usize,
    pub edit_count: usize,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ViewportDiagnostics {
    pub center_degrees: [f64; 2],
    pub vertical_span_degrees: f64,
    pub logical_size: [f32; 2],
    pub physical_size: [f32; 2],
    pub pixels_per_point: f32,
    pub lod: u8,
    pub meters_per_pixel: f64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RenderDiagnostics {
    pub rendered_tiles: usize,
    pub exact_tiles: usize,
    pub fallback_tiles: usize,
    pub stale_tiles: usize,
    pub missing_tiles: usize,
    pub gpu_resident_tiles: usize,
}

/// Live data supplied by renderer and application diagnostic systems.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct DebugTelemetry {
    pub frame: Option<FrameDiagnostics>,
    pub streaming: Option<StreamingDiagnostics>,
    pub viewport: Option<ViewportDiagnostics>,
    pub renderer: Option<RenderDiagnostics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerAvailability {
    Available,
    Unavailable(&'static str),
}

impl LayerAvailability {
    #[must_use]
    pub const fn is_available(self) -> bool {
        matches!(self, Self::Available)
    }

    #[must_use]
    pub const fn reason(self) -> Option<&'static str> {
        match self {
            Self::Available => None,
            Self::Unavailable(reason) => Some(reason),
        }
    }
}

/// Declares which layer controls have a native implementation.
///
/// Integrators should update this resource as native pipeline stages are added.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct LayerCapabilities {
    layers: [LayerAvailability; WorldLayer::COUNT],
}

impl Default for LayerCapabilities {
    fn default() -> Self {
        let mut layers = [LayerAvailability::Unavailable("not implemented in the native pipeline");
            WorldLayer::COUNT];
        layers[WorldLayer::Elevation.index()] = LayerAvailability::Available;
        Self { layers }
    }
}

impl LayerCapabilities {
    #[must_use]
    pub const fn availability(&self, layer: WorldLayer) -> LayerAvailability {
        self.layers[layer.index()]
    }

    pub fn set(&mut self, layer: WorldLayer, availability: LayerAvailability) {
        self.layers[layer.index()] = availability;
    }

    pub fn set_all(&mut self, availability: LayerAvailability) {
        self.layers.fill(availability);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DebugEventLevel {
    Trace,
    Debug,
    #[default]
    Information,
    Warning,
    Error,
}

impl DebugEventLevel {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Information => "INFO",
            Self::Warning => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DebugEvent {
    pub elapsed_seconds: f64,
    pub level: DebugEventLevel,
    pub target: String,
    pub message: String,
}

impl DebugEvent {
    #[must_use]
    pub fn new(
        elapsed_seconds: f64,
        level: DebugEventLevel,
        target: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            elapsed_seconds,
            level,
            target: target.into(),
            message: message.into(),
        }
    }
}

const DEFAULT_EVENT_CAPACITY: usize = 512;

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct DebugEventLog {
    events: VecDeque<DebugEvent>,
    capacity: usize,
    pub dropped_events: u64,
}

impl Default for DebugEventLog {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_EVENT_CAPACITY)
    }
}

impl DebugEventLog {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
            dropped_events: 0,
        }
    }

    pub fn push(&mut self, event: DebugEvent) {
        if self.capacity == 0 {
            self.dropped_events += 1;
            return;
        }
        if self.events.len() == self.capacity {
            self.events.pop_front();
            self.dropped_events += 1;
        }
        self.events.push_back(event);
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.dropped_events = 0;
    }

    #[must_use]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &DebugEvent> {
        self.events.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Requests diagnostic work from the application and renderer plugins.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum DebugCommand {
    SetRenderOptions(DebugRenderOptions),
    SetStreamingFrozen(bool),
    FlushTileCache,
    CaptureSnapshot,
    RunTerrainAudit,
    ClearEvents,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_layer_support_is_explicit() {
        let capabilities = LayerCapabilities::default();

        assert!(
            capabilities
                .availability(WorldLayer::Elevation)
                .is_available()
        );
        assert!(
            !capabilities
                .availability(WorldLayer::Climate)
                .is_available()
        );
        assert_eq!(
            capabilities.availability(WorldLayer::Resources).reason(),
            Some("not implemented in the native pipeline")
        );
    }

    #[test]
    fn event_log_is_bounded_and_reports_drops() {
        let mut log = DebugEventLog::with_capacity(2);
        for index in 0..3 {
            log.push(DebugEvent::new(
                f64::from(index),
                DebugEventLevel::Debug,
                "test",
                index.to_string(),
            ));
        }

        assert_eq!(log.len(), 2);
        assert_eq!(log.dropped_events, 1);
        assert_eq!(
            log.iter().next().map(|event| event.message.as_str()),
            Some("1")
        );
    }

    #[test]
    fn telemetry_is_explicitly_disconnected_by_default() {
        let telemetry = DebugTelemetry::default();
        assert!(telemetry.frame.is_none());
        assert!(telemetry.streaming.is_none());
        assert!(telemetry.viewport.is_none());
        assert!(telemetry.renderer.is_none());
    }
}
