use std::time::Instant;

use tracing::{
    field::{Field, Visit},
    Event, Level, Subscriber,
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::FilterFn,
    fmt::{self, format::Writer, FmtContext, FormatEvent, FormatFields},
    layer::{Context, SubscriberExt},
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

use std::io::{self, Write};

use crate::utils::file_writer;

fn unbuffered_stdout() -> impl Write {
    io::stdout()
}

pub const ENABLED_LOG_TARGETS: [&str; 4] = ["comm", "strat", "main", "test"];
// pub const DISABLE_LOG_FILES: [&str; 1] = ["websocket"];
pub const DISABLE_LOG_FILES: [&str; 0] = [];

pub struct TargetFilter;

impl<S> Layer<S> for TargetFilter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn enabled(&self, metadata: &tracing::Metadata<'_>, _: Context<'_, S>) -> bool {
        //sanity check
        // return true;
        let target = metadata.target();
        let file = metadata.file().unwrap_or("");

        if ENABLED_LOG_TARGETS.contains(&target) {
            return true;
        }
        for filter in DISABLE_LOG_FILES.iter() {
            if file.ends_with(filter) {
                return false;
            }
        }
        for filter in ENABLED_LOG_TARGETS.iter() {
            if target.starts_with(&format!("{filter}/")) {
                return true;
            }
        }
        false
    }
}

pub fn level_color(level: &tracing::Level) -> &'static str {
    match *level {
        tracing::Level::ERROR => "\x1b[31m", // red
        tracing::Level::WARN => "\x1b[33m",  // yellow
        tracing::Level::INFO => "\x1b[32m",  // green
        tracing::Level::DEBUG => "\x1b[96m", // blue
        tracing::Level::TRACE => "\x1b[90m", // gray
    }
}

pub fn level_bg_color(level: &tracing::Level) -> &'static str {
    match *level {
        tracing::Level::ERROR => "\x1b[41m",  // red background
        tracing::Level::WARN => "\x1b[43m",   // yellow background
        tracing::Level::INFO => "\x1b[42m",   // green background
        tracing::Level::DEBUG => "\x1b[106m", // blue background
        tracing::Level::TRACE => "\x1b[100m", // bright black (gray)
    }
}

pub const RESET_COLOR: &str = "\x1b[0m";
pub const BLACK: &str = "\x1b[30m";
pub const STD_BG: &str = "\x1b[0;100m";

pub struct MyFormatter {
    start_time: Instant,
}

impl Default for MyFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl MyFormatter {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
}

impl<S, N> FormatEvent<S, N> for MyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();
        let module = meta.module_path().unwrap_or("");
        let module = module.rsplit("::").next().unwrap_or("");
        let target = meta.target();
        let level = meta.level();
        let time = self.start_time.elapsed().as_secs_f64();

        let level_color = level_color(level);
        let level_bg_color = level_bg_color(level);

        let level = format!("{level_bg_color} {level:<6} {STD_BG}");

        let info = format!("[{time:>8.2}] [{BLACK} {level} {module:<10} {target:<6}");

        let info_len = console::measure_text_width(info.as_str());

        let target_len = 55;
        let pad = target_len - usize::min(target_len, info_len);

        let pad_str = " ".repeat(pad);

        write!(writer, "{info}{pad_str}   ")?;
        write!(writer, " {RESET_COLOR} ] ")?;

        write!(writer, "{level_color}")?;

        let mut visitor = MessageVisitor { msg: None };
        event.record(&mut visitor);
        let msg = visitor.msg.unwrap_or_default();

        write!(writer, " {msg}")?;

        // ctx.format_fields(writer.by_ref(), event)?;

        write!(writer, "{RESET_COLOR}")?;

        writeln!(writer)
    }
}

struct TestFormatter {
    start: Instant,
}

impl TestFormatter {
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl<S, N> FormatEvent<S, N> for TestFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();

        let elapsed = self.start.elapsed().as_secs_f64();
        let level = meta.level();
        let target = meta.target();

        let mut visitor = MessageVisitor { msg: None };
        event.record(&mut visitor);
        let msg = visitor.msg.unwrap_or_default();
        let module = meta.module_path().unwrap_or("");

        let _level_color = level_color(level);
        let level_bg_color = level_bg_color(level);

        let level = format!("{level_bg_color} {level:<6} {STD_BG}");

        let info =
            format!("[{elapsed:>8.2}] [{BLACK} {level} {module:<10} {target:<6} {RESET_COLOR}]");

        // header
        write!(writer, "{info} {msg} ",)?;

        // ctx.format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

pub struct MessageVisitor {
    pub msg: Option<String>,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, _field: &Field, value: &dyn std::fmt::Debug) {
        self.msg = Some(format!("{:?}", value));
    }

    fn record_str(&mut self, _field: &Field, value: &str) {
        self.msg = Some(value.to_string());
    }
}

pub fn init_logging() -> Vec<WorkerGuard> {
    let env_filter = EnvFilter::new("debug");
    let fmt_layer = fmt::layer()
        .event_format(MyFormatter::new())
        .with_ansi(true);
    // .with_filter(FilterFn::new(|meta| {
    //     !meta.module_path().unwrap_or("").ends_with("websocket")
    // }));

    let (non_blocking, guard) = file_writer::file_appender("comm/msg_log", "messages");

    let msg_log_layer = tracing_subscriber::fmt::layer()
        // .with_file(true)
        // .with_target(true)
        .with_ansi(true)
        .with_writer(non_blocking)
        .event_format(MyFormatter::new())
        .with_filter(FilterFn::new(|meta| {
            meta.target() == "comm/msg_log"
                || meta.target().ends_with("event")
                || *meta.level() == Level::ERROR
        }));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(msg_log_layer)
        .init();

    vec![guard]
}

pub fn test_logging(env_filter: &str) -> (impl tracing::Subscriber, Vec<WorkerGuard>) {
    let fmt_layer = fmt::layer()
        .with_file(true)
        .with_target(true)
        .with_ansi(true)
        .event_format(TestFormatter::new());

    let (non_blocking, guard) = file_writer::file_appender("comm/msg_log", "messages");

    let msg_log_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_target(true)
        .with_ansi(false)
        .with_writer(non_blocking)
        .event_format(TestFormatter::new());

    (
        tracing_subscriber::registry()
            .with(EnvFilter::new(env_filter))
            .with(fmt_layer)
            .with(msg_log_layer),
        vec![guard],
    )
}

pub fn run_test<T>(env_filter: &str, f: impl FnOnce() -> T) -> T {
    let (subscriber, _guards) = test_logging(env_filter);
    tracing::subscriber::with_default(subscriber, f)
}
