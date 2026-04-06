use std::{fmt::Debug, time::Instant};

use tracing::{Event, Instrument, Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::Writer, time, FmtContext, FormatEvent, FormatFields},
    layer::{Context, SubscriberExt},
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

pub const ENABLED_LOG_TARGETS: [&str; 3] = ["comm", "strat", "main"];

struct TargetFilter;

impl<S> Layer<S> for TargetFilter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn enabled(&self, metadata: &tracing::Metadata<'_>, _: Context<'_, S>) -> bool {
        //sanity check
        // return true;
        let target = metadata.target();

        if ENABLED_LOG_TARGETS.contains(&target) {
            return true;
        }
        for filter in ENABLED_LOG_TARGETS.iter() {
            if target.starts_with(&format!("{filter}/")) {
                return true;
            }
        }
        false
    }
}

fn level_color(level: &tracing::Level) -> &'static str {
    match *level {
        tracing::Level::ERROR => "\x1b[31m", // red
        tracing::Level::WARN  => "\x1b[33m", // yellow
        tracing::Level::INFO  => "\x1b[32m", // green
        tracing::Level::DEBUG => "\x1b[96m", // blue
        tracing::Level::TRACE => "\x1b[90m", // gray
    }
}

fn level_bg_color(level: &tracing::Level) -> &'static str {
    match *level {
        tracing::Level::ERROR => "\x1b[41m",  // red background
        tracing::Level::WARN  => "\x1b[43m",  // yellow background
        tracing::Level::INFO  => "\x1b[42m",  // green background
        tracing::Level::DEBUG => "\x1b[106m",  // blue background
        tracing::Level::TRACE => "\x1b[100m", // bright black (gray)
    }
}

const RESET_COLOR: &str = "\x1b[0m";
const BLACK: &str = "\x1b[30m";
const STD_BG: &str = "\x1b[0;100m";

struct MyFormatter {
    start_time: Instant,
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
        ctx: &FmtContext<'_, S, N>,
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


        let info = format!("[{time:>8.2}] [{BLACK} {level} {module:<10} {target:<6} {RESET_COLOR}]");

        let info_len = console::measure_text_width(info.as_str());

        let target_len = 50;
        let pad = target_len - usize::min(target_len, info_len);

        let pad_str = " ".repeat(pad);

        write!(
            writer,
            "{info}{pad_str}   "
        )?;

        write!(writer, "{level_color}")?;

        ctx.format_fields(writer.by_ref(), event)?;

        write!(writer, "{RESET_COLOR}")?;

        writeln!(writer)
    }
}

pub fn init_logging() {
    let env_filter = EnvFilter::new("debug");
    let fmt_layer = fmt::layer().event_format(MyFormatter::new()).with_ansi(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
