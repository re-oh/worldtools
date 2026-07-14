use std::{
    backtrace::Backtrace,
    env,
    fmt::{self, Write as _},
    fs::{self, File},
    io::{self, Write},
    panic::{self, PanicHookInfo},
    path::{Path, PathBuf},
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use bevy::{
    app::App,
    ecs::resource::Resource,
    log::{BoxedLayer, DEFAULT_FILTER, LogPlugin, tracing_subscriber},
};
use crossbeam_channel::{Receiver, Sender, TrySendError};
use tracing::{Event, Level, Subscriber, field::Visit, info};
use tracing_subscriber::{Layer, fmt::format::FmtSpan, layer::Context, registry::LookupSpan};

const DEFAULT_APP_FILTER: &str =
    "worldtools=debug,worldtools_render=debug,worldtools_ui=debug,worldtools_world=info";
const LOG_ENV: &str = "WORLDTOOLS_LOG";
const LOG_DIR_ENV: &str = "WORLDTOOLS_LOG_DIR";
const EVENT_BUFFER_CAPACITY: usize = 2_048;

static LOG_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
static EVENT_SENDER: OnceLock<Sender<DiagnosticEvent>> = OnceLock::new();
static PROCESS_STARTED: OnceLock<Instant> = OnceLock::new();
static DROPPED_EVENTS: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl DiagnosticLevel {
    fn from_tracing(level: Level) -> Self {
        match level {
            Level::TRACE => Self::Trace,
            Level::DEBUG => Self::Debug,
            Level::INFO => Self::Info,
            Level::WARN => Self::Warn,
            Level::ERROR => Self::Error,
        }
    }
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        })
    }
}

#[derive(Clone, Debug)]
pub struct DiagnosticEvent {
    pub elapsed_seconds: f64,
    pub level: DiagnosticLevel,
    pub target: String,
    pub message: String,
    pub fields: String,
    pub thread: String,
}

impl fmt::Display for DiagnosticEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "[{}] {}: {}",
            self.level, self.target, self.message
        )?;
        if !self.fields.is_empty() {
            write!(formatter, " {{{}}}", self.fields)?;
        }
        Ok(())
    }
}

/// Cloneable, non-blocking input for an in-app debug event log.
///
/// Drain this with [`Self::try_iter`] from a Bevy system. Do not emit tracing
/// events for each drained item, since those would re-enter the event stream.
#[derive(Clone, Resource)]
pub struct DiagnosticEventReceiver(Receiver<DiagnosticEvent>);

impl DiagnosticEventReceiver {
    #[must_use]
    pub fn try_iter(&self) -> crossbeam_channel::TryIter<'_, DiagnosticEvent> {
        self.0.try_iter()
    }

    #[must_use]
    #[allow(clippy::unused_self)] // The receiver is the public handle for this shared channel counter.
    pub fn take_dropped_events(&self) -> u64 {
        DROPPED_EVENTS.swap(0, Ordering::Relaxed)
    }
}

/// Stable output directory shared by snapshots, audits, rolling logs, and panic reports.
#[derive(Clone, Debug, Resource)]
pub struct DiagnosticDirectory(PathBuf);

impl DiagnosticDirectory {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }
}

/// Process-level diagnostics configured before Bevy starts.
///
/// Keep this value in `main` long enough to call [`Self::record_startup`] after
/// Bevy installs its tracing subscriber.
pub struct Diagnostics {
    directory: PathBuf,
    events: DiagnosticEventReceiver,
    filter: String,
    started_at: Instant,
}

impl Diagnostics {
    #[must_use]
    pub fn install() -> Self {
        let directory = diagnostics_directory();
        if let Err(error) = fs::create_dir_all(&directory) {
            eprintln!(
                "WorldTools could not create diagnostics directory {}: {error}",
                directory.display()
            );
        }

        let _ = LOG_DIRECTORY.set(directory.clone());
        let (event_sender, event_receiver) = crossbeam_channel::bounded(EVENT_BUFFER_CAPACITY);
        let _ = EVENT_SENDER.set(event_sender);
        let started_at = Instant::now();
        let _ = PROCESS_STARTED.set(started_at);
        install_panic_hook(directory.clone(), started_at);

        let app_filter = env::var(LOG_ENV).unwrap_or_else(|_| DEFAULT_APP_FILTER.to_owned());
        let filter = format!("{DEFAULT_FILTER}{app_filter}");

        Self {
            directory,
            events: DiagnosticEventReceiver(event_receiver),
            filter,
            started_at,
        }
    }

    #[must_use]
    pub fn log_plugin(&self) -> LogPlugin {
        LogPlugin {
            filter: self.filter.clone(),
            custom_layer: file_log_layer,
            ..LogPlugin::default()
        }
    }

    pub fn record_startup(&self) {
        info!(
            target: "worldtools::diagnostics",
            version = env!("CARGO_PKG_VERSION"),
            profile = build_profile(),
            os = env::consts::OS,
            architecture = env::consts::ARCH,
            process_id = std::process::id(),
            working_directory = %display_current_directory(),
            diagnostics_directory = %self.directory.display(),
            log_filter = %self.filter,
            startup_milliseconds = self.started_at.elapsed().as_millis(),
            "WorldTools diagnostics initialized"
        );
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    #[must_use]
    pub fn event_receiver(&self) -> DiagnosticEventReceiver {
        self.events.clone()
    }

    #[must_use]
    pub fn directory_resource(&self) -> DiagnosticDirectory {
        DiagnosticDirectory(self.directory.clone())
    }
}

fn diagnostics_directory() -> PathBuf {
    env::var_os(LOG_DIR_ENV).map_or_else(
        || {
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(".runtime")
                .join("diagnostics")
        },
        PathBuf::from,
    )
}

fn file_log_layer(app: &mut App) -> Option<BoxedLayer> {
    let directory = LOG_DIRECTORY
        .get()
        .cloned()
        .unwrap_or_else(diagnostics_directory);

    if let Err(error) = fs::create_dir_all(&directory) {
        eprintln!(
            "WorldTools file logging disabled; could not create {}: {error}",
            directory.display()
        );
        return None;
    }

    let appender = tracing_appender::rolling::daily(directory, "worldtools.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);
    app.insert_non_send(FileLogGuard(guard));

    let file_layer: BoxedLayer = Box::new(
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(writer),
    );
    let event_layer: BoxedLayer = Box::new(DebugEventLayer);
    Some(Box::new(vec![file_layer, event_layer]))
}

struct FileLogGuard(#[allow(dead_code)] tracing_appender::non_blocking::WorkerGuard);

struct DebugEventLayer;

impl<S> Layer<S> for DebugEventLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let metadata = event.metadata();
        if !metadata.target().starts_with("worldtools") {
            return;
        }

        let Some(sender) = EVENT_SENDER.get() else {
            return;
        };
        let mut visitor = EventFieldVisitor::default();
        event.record(&mut visitor);
        let diagnostic = DiagnosticEvent {
            elapsed_seconds: PROCESS_STARTED
                .get()
                .map_or(0.0, |started| started.elapsed().as_secs_f64()),
            level: DiagnosticLevel::from_tracing(*metadata.level()),
            target: metadata.target().to_owned(),
            message: visitor
                .message
                .unwrap_or_else(|| metadata.name().to_owned()),
            fields: visitor.fields,
            thread: std::thread::current()
                .name()
                .unwrap_or("unnamed")
                .to_owned(),
        };

        if matches!(sender.try_send(diagnostic), Err(TrySendError::Full(_))) {
            DROPPED_EVENTS.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[derive(Default)]
struct EventFieldVisitor {
    message: Option<String>,
    fields: String,
}

impl EventFieldVisitor {
    fn record_value(&mut self, field: &tracing::field::Field, value: impl fmt::Display) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
            return;
        }

        if !self.fields.is_empty() {
            self.fields.push_str(", ");
        }
        let _ = write!(self.fields, "{}={value}", field.name());
    }
}

impl Visit for EventFieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        self.record_value(field, format_args!("{value:?}"));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.record_value(field, value);
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.record_value(field, value);
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.record_value(field, value);
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.record_value(field, value);
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.record_value(field, value);
    }
}

fn install_panic_hook(directory: PathBuf, started_at: Instant) {
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let report_path = write_panic_report(&directory, started_at, panic_info);
        match report_path {
            Ok(path) => {
                tracing::error!(
                    target: "worldtools::diagnostics",
                    report = %path.display(),
                    panic = %panic_message(panic_info),
                    "WorldTools panic report written"
                );
                eprintln!("WorldTools panic report: {}", path.display());
            }
            Err(error) => {
                eprintln!("WorldTools could not write a panic report: {error}");
            }
        }
        previous_hook(panic_info);
    }));
}

fn write_panic_report(
    directory: &Path,
    started_at: Instant,
    panic_info: &PanicHookInfo<'_>,
) -> io::Result<PathBuf> {
    fs::create_dir_all(directory)?;
    let timestamp = unix_timestamp_millis();
    let path = directory.join(format!("panic-{timestamp}-{}.txt", std::process::id()));
    let mut report = File::create(&path)?;
    let thread = std::thread::current();
    let location = panic_info
        .location()
        .map_or_else(|| "unknown".to_owned(), ToString::to_string);

    writeln!(report, "WorldTools panic report")?;
    writeln!(report, "unix_timestamp_ms: {timestamp}")?;
    writeln!(report, "version: {}", env!("CARGO_PKG_VERSION"))?;
    writeln!(report, "profile: {}", build_profile())?;
    writeln!(report, "os: {}", env::consts::OS)?;
    writeln!(report, "architecture: {}", env::consts::ARCH)?;
    writeln!(report, "process_id: {}", std::process::id())?;
    writeln!(report, "uptime_ms: {}", started_at.elapsed().as_millis())?;
    writeln!(
        report,
        "thread_name: {}",
        thread.name().unwrap_or("unnamed")
    )?;
    writeln!(report, "thread_id: {:?}", thread.id())?;
    writeln!(report, "working_directory: {}", display_current_directory())?;
    writeln!(report, "location: {location}")?;
    writeln!(report, "message: {}", panic_message(panic_info))?;
    writeln!(report, "{LOG_ENV}: {}", display_environment(LOG_ENV))?;
    writeln!(report, "RUST_LOG: {}", display_environment("RUST_LOG"))?;
    writeln!(
        report,
        "RUST_BACKTRACE: {}",
        display_environment("RUST_BACKTRACE")
    )?;
    writeln!(report, "\nbacktrace:\n{}", Backtrace::force_capture())?;
    report.flush()?;

    Ok(path)
}

fn panic_message(panic_info: &PanicHookInfo<'_>) -> String {
    panic_info.payload().downcast_ref::<&str>().map_or_else(
        || {
            panic_info
                .payload()
                .downcast_ref::<String>()
                .map_or_else(|| "non-string panic payload".to_owned(), Clone::clone)
        },
        |message| (*message).to_owned(),
    )
}

fn display_current_directory() -> String {
    env::current_dir().map_or_else(
        |error| format!("unavailable ({error})"),
        |path| path.display().to_string(),
    )
}

fn display_environment(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| "unset".to_owned())
}

const fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_path_has_stable_default_suffix() {
        if env::var_os(LOG_DIR_ENV).is_none() {
            assert!(diagnostics_directory().ends_with(Path::new(".runtime/diagnostics")));
        }
    }

    #[test]
    fn build_profile_is_known() {
        assert!(matches!(build_profile(), "debug" | "release"));
    }
}
