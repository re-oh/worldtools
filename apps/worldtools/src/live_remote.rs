use std::net::{IpAddr, Ipv4Addr};

use bevy::{
    prelude::*,
    remote::{
        BrpError, BrpResult, RemotePlugin,
        builtin_methods::{
            BRP_DESPAWN_COMPONENTS_METHOD, BRP_INSERT_COMPONENTS_METHOD,
            BRP_INSERT_RESOURCE_METHOD, BRP_MUTATE_COMPONENTS_METHOD, BRP_MUTATE_RESOURCE_METHOD,
            BRP_REMOVE_COMPONENTS_METHOD, BRP_REMOVE_RESOURCE_METHOD, BRP_REPARENT_ENTITIES_METHOD,
            BRP_SPAWN_ENTITY_METHOD, BRP_TRIGGER_EVENT_METHOD, BRP_WRITE_MESSAGE_METHOD,
        },
        error_codes,
        http::{DEFAULT_PORT, DEFAULT_RENDER_PORT, RemoteHttpPlugin},
    },
};
use serde_json::{Value, json};
use worldtools_render::{MapView, RenderDebugSettings, TileRenderStats, TileStreamStats};
use worldtools_ui::DocumentStatus;

const ENABLE_ENV: &str = "WORLDTOOLS_BRP";
const ALLOW_WRITE_ENV: &str = "WORLDTOOLS_BRP_ALLOW_WRITE";
const WORLDTOOLS_STATUS_METHOD: &str = "worldtools.status";

pub struct WorldToolsRemoteDebugPlugin;

impl Plugin for WorldToolsRemoteDebugPlugin {
    fn build(&self, app: &mut App) {
        let config = RemoteDebugConfig::from_env();
        if !config.enabled {
            tracing::debug!(
                target: "worldtools::remote",
                variable = ENABLE_ENV,
                "Bevy Remote Protocol is compiled in but disabled"
            );
            return;
        }

        if !cfg!(debug_assertions) {
            tracing::warn!(
                target: "worldtools::remote",
                "Refusing to start the development BRP endpoint in a release build"
            );
            return;
        }

        let access = if config.allow_write {
            RemoteAccess::ReadWrite
        } else {
            RemoteAccess::ReadOnly
        };
        let remote = remote_plugin(access);

        app.register_type::<RemoteDebugStatus>()
            .insert_resource(RemoteDebugStatus {
                address: Ipv4Addr::LOCALHOST.to_string(),
                port: DEFAULT_PORT,
                render_port: DEFAULT_RENDER_PORT,
                writable: access == RemoteAccess::ReadWrite,
            })
            .add_plugins(remote)
            .add_plugins(
                RemoteHttpPlugin::default()
                    .with_address(IpAddr::V4(Ipv4Addr::LOCALHOST))
                    .with_port(DEFAULT_PORT),
            );

        match access {
            RemoteAccess::ReadOnly => tracing::info!(
                target: "worldtools::remote",
                address = %Ipv4Addr::LOCALHOST,
                port = DEFAULT_PORT,
                render_port = DEFAULT_RENDER_PORT,
                "Read-only Bevy Remote Protocol endpoint enabled"
            ),
            RemoteAccess::ReadWrite => tracing::warn!(
                target: "worldtools::remote",
                address = %Ipv4Addr::LOCALHOST,
                port = DEFAULT_PORT,
                render_port = DEFAULT_RENDER_PORT,
                guard = ALLOW_WRITE_ENV,
                "Writable Bevy Remote Protocol endpoint enabled"
            ),
        }
    }
}

#[derive(Clone, Debug, Reflect, Resource)]
#[reflect(Resource)]
pub struct RemoteDebugStatus {
    pub address: String,
    pub port: u16,
    pub render_port: u16,
    pub writable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RemoteAccess {
    ReadOnly,
    ReadWrite,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RemoteDebugConfig {
    enabled: bool,
    allow_write: bool,
}

impl RemoteDebugConfig {
    fn from_env() -> Self {
        Self::from_values(
            std::env::var(ENABLE_ENV).ok().as_deref(),
            std::env::var(ALLOW_WRITE_ENV).ok().as_deref(),
        )
    }

    fn from_values(enabled: Option<&str>, allow_write: Option<&str>) -> Self {
        let enabled = enabled.is_some_and(flag_is_enabled);
        Self {
            enabled,
            allow_write: enabled && allow_write.is_some_and(flag_is_enabled),
        }
    }
}

fn flag_is_enabled(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn remote_plugin(access: RemoteAccess) -> RemotePlugin {
    let mut plugin =
        RemotePlugin::default().with_method_main(WORLDTOOLS_STATUS_METHOD, worldtools_status);
    if access == RemoteAccess::ReadOnly {
        for method in MUTATING_METHODS {
            plugin = plugin
                .with_method_main(method, reject_write_request)
                .with_method_render(method, reject_write_request);
        }
    }
    plugin
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
#[allow(clippy::unnecessary_wraps)] // BRP handlers require a Result even for infallible snapshots.
fn worldtools_status(
    _: In<Option<Value>>,
    document: Res<DocumentStatus>,
    view: Res<MapView>,
    streaming: Res<TileStreamStats>,
    renderer: Res<TileRenderStats>,
    debug: Res<RenderDebugSettings>,
) -> BrpResult {
    Ok(json!({
        "document": {
            "name": document.name,
            "seed": document.seed,
            "save_state": format!("{:?}", document.save_state),
            "can_undo": document.can_undo,
            "can_redo": document.can_redo,
        },
        "view": {
            "center": [view.center.x, view.center.y],
            "vertical_span": view.vertical_span,
        },
        "streaming": {
            "level": streaming.level,
            "visible": streaming.visible,
            "resident_visible": streaming.resident_visible,
            "resident_total": streaming.resident_total,
            "in_flight": streaming.in_flight,
            "completed": streaming.completed,
            "discarded": streaming.discarded,
            "requested": streaming.requested,
            "invalidated": streaming.invalidated,
            "last_generation_ms": streaming.last_generation_ms,
            "max_generation_ms": streaming.max_generation_ms,
            "ready_results": streaming.ready_results,
            "edit_count": streaming.edit_count,
        },
        "renderer": {
            "rendered": renderer.rendered,
            "exact": renderer.exact,
            "fallback": renderer.fallback,
            "stale": renderer.stale,
            "missing": renderer.missing,
            "gpu_resident": renderer.gpu_resident,
        },
        "debug": {
            "tile_borders": debug.tile_borders,
            "lod_tint": debug.lod_tint,
            "residency_tint": debug.residency_tint,
            "trace_streaming": debug.trace_streaming,
            "freeze_streaming": debug.freeze_streaming,
        },
    }))
}

const MUTATING_METHODS: [&str; 11] = [
    BRP_SPAWN_ENTITY_METHOD,
    BRP_INSERT_COMPONENTS_METHOD,
    BRP_REMOVE_COMPONENTS_METHOD,
    BRP_DESPAWN_COMPONENTS_METHOD,
    BRP_REPARENT_ENTITIES_METHOD,
    BRP_MUTATE_COMPONENTS_METHOD,
    BRP_INSERT_RESOURCE_METHOD,
    BRP_REMOVE_RESOURCE_METHOD,
    BRP_MUTATE_RESOURCE_METHOD,
    BRP_TRIGGER_EVENT_METHOD,
    BRP_WRITE_MESSAGE_METHOD,
];

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn reject_write_request(_: In<Option<Value>>) -> BrpResult {
    Err(BrpError {
        code: error_codes::INVALID_REQUEST,
        message: format!(
            "WorldTools BRP is read-only; set {ALLOW_WRITE_ENV}=1 before launch to permit writes"
        ),
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_debug_is_disabled_without_explicit_opt_in() {
        assert_eq!(
            RemoteDebugConfig::from_values(None, Some("1")),
            RemoteDebugConfig::default()
        );
    }

    #[test]
    fn write_access_requires_both_flags() {
        assert_eq!(
            RemoteDebugConfig::from_values(Some("true"), None),
            RemoteDebugConfig {
                enabled: true,
                allow_write: false,
            }
        );
        assert_eq!(
            RemoteDebugConfig::from_values(Some("on"), Some("yes")),
            RemoteDebugConfig {
                enabled: true,
                allow_write: true,
            }
        );
    }

    #[test]
    fn flag_parsing_is_strict() {
        assert!(flag_is_enabled(" TRUE "));
        assert!(!flag_is_enabled("enabled"));
        assert!(!flag_is_enabled("0"));
    }

    #[test]
    fn read_only_handler_rejects_mutation() {
        let error = reject_write_request(In(None)).expect_err("write must be rejected");
        assert_eq!(error.code, error_codes::INVALID_REQUEST);
        assert!(error.message.contains(ALLOW_WRITE_ENV));
    }
}
