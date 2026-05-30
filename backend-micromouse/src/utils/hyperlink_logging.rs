use std::{
    collections::HashMap,
    fmt::Display,
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Instant,
};

use chrono::Local;
use pathdiff::diff_paths;
use tracing::{
    field::{Field, Visit},
    span::{self, EnteredSpan},
    Event, Level, Subscriber,
};
use tracing_subscriber::{fmt, layer::Context, registry::LookupSpan, EnvFilter, Layer};

use crate::{
    comm::micromouse_message::CommandId,
    strategy::strategy_tree::{AbsoluteLayerId, AbsoluteNodeId, AbsolutePathId},
    utils::logging::{
        level_bg_color, level_color, MessageVisitor, TestFormatter, BLACK, RESET_COLOR, STD_BG,
    },
};

pub const BASE_FILE: &str = "index.html";
pub const BASE_EXTENSION: &str = "html";

/// Unified Visitor to collect everything in one pass over the metadata/fields
struct LogVisitor {
    pub name: Option<String>,
    pub links: Vec<Link>,
    pub fields: HashMap<String, String>,
    pub message: Option<String>,
}

impl Visit for LogVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "name" {
            let name = value.to_string();
            // .split_at_checked(10)
            // .map(|m| m.0.to_string())
            // .unwrap_or(value.to_string());
            self.name = Some(name);
        } else if field.name() == "message" {
            self.message = Some(value.to_string());
        } else if field.name().starts_with("link_") {
            let category = &field.name()["link_".len()..];
            self.links.push(Link {
                category: category.to_string(),
                name: value.to_string(),
            });
        } else {
            self.fields
                .insert(field.name().to_string(), value.to_string());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn core::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        } else if field.name() != "name" && !field.name().starts_with("link_") {
            self.fields
                .insert(field.name().to_string(), format!("{:?}", value));
        }
    }
}

pub trait LinkFileName {
    fn link(&self) -> String;
}

impl LinkFileName for CommandId {
    fn link(&self) -> String {
        format!("cmd_{}", self.0)
    }
}

impl LinkFileName for AbsoluteNodeId {
    fn link(&self) -> String {
        format!("node_L{}_N{}", self.layer_id().0, self.node_id().0)
    }
}

impl LinkFileName for AbsoluteLayerId {
    fn link(&self) -> String {
        format!("layer_{}", self.0)
    }
}

impl LinkFileName for AbsolutePathId {
    fn link(&self) -> String {
        format!(
            "path_N{}_S{}_I{}",
            self.from_node.link(),
            self.branch.at_step,
            self.branch.from_interrupt.link()
        )
    }
}

#[derive(Clone)]
struct Link {
    category: String,
    name: String,
}

struct LogSpan {
    dir: PathBuf,
    links: Vec<Link>,
}

pub struct RoutingLayer {
    files: Mutex<HashMap<PathBuf, File>>,
    run_root: PathBuf,
    start_time: Instant,
}

impl RoutingLayer {
    pub fn new() -> Self {
        let run_id = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let root = PathBuf::from("logs").join(run_id);
        Self {
            files: Mutex::new(HashMap::new()),
            run_root: root,
            start_time: Instant::now(),
        }
    }

    fn get_file(&self, path: &Path) -> File {
        let mut files = self.files.lock().unwrap();
        if !files.contains_key(path) {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap();

            // Write the Header/Styles only once per file
            writeln!(file, "{}", self.get_styles()).ok();
            files.insert(path.to_path_buf(), file);
        }
        files.get(path).unwrap().try_clone().unwrap()
    }

    fn get_styles(&self) -> &str {
        r#"<style>
            html, body {
                background-image: radial-gradient(circle at top right, #1a1a2e, #0a0a0c);
                color: #e0e0e0;
                margin: 0; padding: 20px;
                font-family: 'Consolas', 'Monaco', monospace;
                line-height: 1.5;
            }
            section.span-container {
                margin: 20px 0;
                padding: 15px;
                border-radius: 8px;
                background-color: rgba(255, 255, 255, 0.03);
                backdrop-filter: blur(8px);
                border: 1px solid rgba(255, 255, 255, 0.1);
            }
            header.span-header {
                border-bottom: 1px solid rgba(255, 255, 255, 0.1);
                margin-bottom: 10px;
                padding-bottom: 5px;
                display: flex;
                justify-content: space-between;
            }
            dl {
                display: grid;
                grid-template-columns: max-content auto;
                gap: 5px 15px;
                margin: 10px 0;
                font-size: 0.9em;
                padding: 10px;
                background: rgba(0, 0, 0, 0.2);
                border-radius: 4px;
            }
            dt { color: #569cd6; font-weight: bold; }
            dd { margin: 0; color: #ce9178; }
            dd code { background: rgba(255, 255, 255, 0.05); padding: 2px 4px; border-radius: 3px; }
            /* 1. Universal Log Style - Everything starts at the same X-coordinate */
/* The main container for EVERY log line */
.log-entry {
    position: relative;
    margin-bottom: 6px;
    padding: 8px 12px 8px 25px; /* 25px left padding creates the 'gutter' */
    background-color: rgba(255, 255, 255, 0.03);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    display: block;
    transition: background 0.2s;
}

/* The Teal Indicator - Positioned absolutely so it NEVER shifts text */
details.log-entry > summary::before {
    content: "";
    position: absolute;
    left: 8px;   /* Inside the 25px gutter */
    top: 8px;    /* Aligned with top padding */
    bottom: 8px; /* Aligned with bottom padding */
    width: 4px;
    background-color: #4ec9b0;
    border-radius: 2px;
    opacity: 0.6;
}

/* Hover/Open States */
.log-entry:hover {
    background-color: rgba(255, 255, 255, 0.06);
}

details[open].log-entry > summary::before {
    opacity: 1;
    box-shadow: 0 0 8px rgba(78, 201, 176, 0.4);
}

/* Clean up summary/pre tag defaults */
summary {
    list-style: none;
    outline: none;
    cursor: pointer;
}
summary::-webkit-details-marker { display: none; }

pre {
    display: inline;
    margin: 0;
    padding: 0;
    background: transparent;
    font-family: 'Consolas', monospace;
    white-space: pre-wrap;
}

.expanded-entry {
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
}
            a { color: #4ec9b0; text-decoration: none; border-bottom: 1px dashed rgba(78, 201, 176, 0.3); }
            a:hover { color: #fff; border-bottom: 1px solid #4ec9b0; }
        </style>"#
    }
}

impl<S> Layer<S> for RoutingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();

        let parent_links = span
            .parent()
            .as_ref()
            .map(|p| p.extensions())
            .as_ref()
            .and_then(|e| e.get::<LogSpan>())
            .map(|e| e.links.clone())
            .unwrap_or(vec![]);

        let mut visitor = LogVisitor {
            name: None,
            links: parent_links,
            fields: HashMap::new(),
            message: None,
        };
        attrs.record(&mut visitor);

        let name = visitor.name.as_deref().unwrap_or(span.metadata().name());

        let short_name = name.split_at_checked(8).map(|m| m.0).unwrap_or(name);

        let parent_dir = span
            .parent()
            .and_then(|p| p.extensions().get::<LogSpan>().map(|s| s.dir.clone()))
            .unwrap_or_else(|| self.run_root.clone());

        let current_dir = parent_dir.join(short_name);
        let current_file = current_dir.join(BASE_FILE);
        let parent_file = parent_dir.join(BASE_FILE);

        span.extensions_mut().insert(LogSpan {
            dir: current_dir.clone(),
            links: visitor.links.clone(),
        });

        {
            let mut file = self.get_file(&current_file);
            let rel_up = diff_paths(&parent_file, &current_dir).unwrap();

            writeln!(file, "<section class='span-container'>").ok();
            writeln!(file, "  <header class='span-header'>").ok();
            writeln!(file, "    <span><strong>SPAN: {}</strong></span>", name).ok();
            writeln!(
                file,
                "    <span>{}</span>",
                link_str(rel_up.to_string_lossy(), "↑ Up to Parent")
            )
            .ok();
            writeln!(file, "  </header>").ok();

            if !visitor.fields.is_empty() {
                writeln!(
                    file,
                    "  <details open><summary>Captured Fields</summary><dl>"
                )
                .ok();
                for (k, v) in visitor.fields.iter() {
                    writeln!(file, "    <dt>{}</dt><dd><code>{}</code></dd>", k, v).ok();
                }
                writeln!(file, "  </dl></details>").ok();
            }
            if !visitor.links.is_empty() {
                writeln!(file, "<details><summary>Related</summary><dl>").ok();
            }
            for link in visitor.links.iter() {
                let link_dir = self.run_root.join(&link.category);
                let link_path = link_dir.join(&link.name).with_extension(BASE_EXTENSION);
                let mut link_file = self.get_file(&link_path);

                let rel_back = diff_paths(&current_file, &link_dir).unwrap();
                // writeln!(link_file, "<section class = 'span-container'>").ok();

                writeln!(link_file, "<details class='log-entry'><summary>").ok();
                writeln!(link_file, "  <pre>Linked span: {}</pre>", name).ok();
                writeln!(link_file, "</summary><div class='expanded-entry'><dl>").ok();
                for (field_k, field_v) in visitor.fields.iter() {
                    writeln!(link_file, "<dt>{field_k}</dt><dd>{field_v}</dd>").ok();
                }
                writeln!(
                    link_file,
                    "<dt>SOURCE</dt><dd>{}</dd>",
                    link_str(rel_back.to_string_lossy(), name)
                )
                .ok();
                writeln!(link_file, "</dl></details>").ok();
            }
            if !visitor.links.is_empty() {
                writeln!(file, "</details>").ok();
            }
        }

        // 2. Link from the parent file
        {
            let mut file = self.get_file(&parent_file);
            let rel_down = diff_paths(&current_file, &parent_dir).unwrap();
            let all_empty = visitor.links.is_empty() && visitor.fields.is_empty();

            if !all_empty {
                writeln!(file, "<details class='log-entry'><summary>").ok();
                writeln!(
                    file,
                    "  <pre>↳ Entering Span: {}</pre>",
                    link_str(rel_down.to_string_lossy(), name)
                )
                .ok();
                writeln!(file, "</summary><div class='expanded-entry'><dl>").ok();
            } else {
                writeln!(file, "<div class='log-entry'>").ok();
                writeln!(
                    file,
                    "  <pre>↳ Entering Span: {}</pre>",
                    link_str(rel_down.to_string_lossy(), name)
                )
                .ok();
            }

            for (k, v) in visitor.fields.iter() {
                writeln!(file, "    <dt>{}</dt><dd><code>{}</code></dd>", k, v).ok();
            }

            for link in visitor.links.iter() {
                let cat = &link.category;
                let name = &link.name;
                let link_dir = self.run_root.join(&link.category);
                let link_path = link_dir.join(&link.name).with_extension(BASE_EXTENSION);
                let rel_to =
                    diff_paths(&link_path, current_dir.parent().expect("Should exist")).unwrap();
                writeln!(
                    file,
                    "    <dt>{cat}</dt><dd>{}</dd>",
                    link_str(rel_to.to_string_lossy(), name)
                )
                .ok();
            }

            if !all_empty {
                writeln!(file, "</dl></div></details>").ok();
            } else {
                writeln!(file, "</div>").ok();
            }
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let span = match ctx.event_scope(event).and_then(|s| s.from_root().last()) {
            Some(s) => s,
            None => return,
        };

        let extensions = span.extensions();
        let span_data = match extensions.get::<LogSpan>() {
            Some(d) => d,
            None => return,
        };

        let mut visitor = LogVisitor {
            name: None,
            links: span_data.links.clone(),
            fields: HashMap::new(),
            message: None,
        };
        event.record(&mut visitor);

        let current_file = span_data.dir.join(BASE_FILE);

        // Formatting the ANSI message (reuse your existing logic)
        let time = self.start_time.elapsed().as_secs_f64();
        let meta = event.metadata();
        let level = meta.level();
        let msg = visitor.message.clone().unwrap_or_default();

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

        let ansii_event_str =
            format!("{info}{pad_str}   {RESET_COLOR} ] {level_color} {msg}{RESET_COLOR}");

        println!("{ansii_event_str}");
        let event_str = ansi_to_html::convert(ansii_event_str.as_str())
            .expect("unable to convert ANSI to HTML");

        {
            let mut main_log_file = self.get_file(&self.run_root.join("out.html"));

            let link_to_file = diff_paths(&current_file, &self.run_root).expect("No link to file");

            writeln!(main_log_file, "<details class='log-entry'><summary>").ok();
            writeln!(main_log_file, "  <pre>{}</pre>", event_str).ok();
            writeln!(main_log_file, "</summary><div class='expanded-entry'><dl>").ok();
            writeln!(
                main_log_file,
                "    <dt>SOURCE</dt><dd><code>{}</code></dd>",
                link_str(
                    link_to_file.to_string_lossy(),
                    current_file.to_string_lossy()
                )
            )
            .ok();
            writeln!(main_log_file, "</dl></div></details>").ok();
        }
        let mut file = self.get_file(&current_file);

        let all_empty = visitor.links.is_empty() && visitor.fields.is_empty();

        if !all_empty {
            writeln!(file, "<details class='log-entry'><summary>").ok();
            writeln!(file, "  <pre>{}</pre>", event_str).ok();
            writeln!(file, "</summary><div class='expanded-entry'><dl>").ok();
        } else {
            writeln!(file, "<div class='log-entry'>").ok();
            writeln!(file, "  <pre>{}</pre>", event_str).ok();
        }

        for (k, v) in visitor.fields.iter() {
            writeln!(file, "    <dt>{}</dt><dd><code>{}</code></dd>", k, v).ok();
        }

        for link in visitor.links.iter() {
            let link_dir = self.run_root.join(&link.category);
            let link_path = link_dir.join(&link.name).with_extension(BASE_EXTENSION);
            let mut link_file = self.get_file(&link_path);

            let rel_back = diff_paths(&current_file, &link_dir).unwrap();
            writeln!(
                link_file,
                "<details class = 'log-entry'><summary><pre>{}</pre></summary>",
                event_str.clone()
            )
            .ok();
            writeln!(link_file, "<div class = 'expanded-entry'><dl>").ok();
            writeln!(
                link_file,
                "<dt>SOURCE</dt><dd>{}</dd>",
                link_str(rel_back.to_string_lossy(), span_data.dir.to_string_lossy())
            )
            .ok();
            for (field_k, field_v) in visitor.fields.iter() {
                writeln!(link_file, "<dt>{field_k}</dt><dd>{field_v}</dd>").ok();
            }
            writeln!(link_file, "</dl></div></details>").ok();

            let rel_to = diff_paths(&link_path, &span_data.dir).unwrap();
            writeln!(
                file,
                "<dt>{}</dt><dd>{}</dd>",
                &link.category,
                link_str(rel_to.to_string_lossy(), &link.name)
            )
            .ok();
        }

        if !all_empty {
            writeln!(file, "</dl></div></details>").ok();
        } else {
            writeln!(file, "</div>").ok();
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            if let Some(span_data) = span.extensions().get::<LogSpan>() {
                let mut file = self.get_file(&span_data.dir.join(BASE_FILE));
                writeln!(file, "</section> ").ok();
            }
        }
    }
}

fn link_str(to: impl Into<String>, content: impl Into<String>) -> String {
    format!("<a href=\"{}\">{}</a>", to.into(), content.into())
}

pub fn init_tree_logger() {
    use tracing_subscriber::prelude::*;

    #[cfg(feature = "hyperlink_logging")]
    {
        tracing_subscriber::registry()
            .with(EnvFilter::new("debug"))
            .with(RoutingLayer::new())
            .init();
    }
    #[cfg(not(feature = "hyperlink_logging"))]
    {
        let warn_fmt_layer = fmt::layer()
            .with_file(true)
            .with_target(true)
            .with_ansi(true)
            .event_format(TestFormatter::new());

        tracing_subscriber::registry()
            .with(EnvFilter::new("warn"))
            .with(warn_fmt_layer);
    }
}

pub fn enter_process(span_name: impl Into<String>) -> EnteredSpan {
    let span_name = span_name.into();
    tracing::span!(Level::DEBUG, "process_span", name = span_name).entered()
}

pub fn process_span(span_name: impl Into<String>) -> tracing::span::Span {
    let span_name = span_name.into();
    tracing::span!(Level::DEBUG, "process_span", name = span_name)
}
