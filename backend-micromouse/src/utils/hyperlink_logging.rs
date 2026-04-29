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
use tracing_subscriber::{layer::Context, registry::LookupSpan, EnvFilter, Layer};

use crate::{
    comm::micromouse_message::CommandId,
    strategy::strategy_tree::AbsoluteNodeId,
    utils::logging::{level_bg_color, level_color, MessageVisitor, BLACK, RESET_COLOR, STD_BG},
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
            self.name = Some(value.to_string());
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
            details summary { cursor: pointer; color: #4ec9b0; outline: none; }
            pre {
                background-color: rgba(0, 0, 0, 0.2);
                padding: 10px;
                border-radius: 5px;
                overflow-x: auto;
                white-space: pre-wrap;
                margin: 5px 0;
                line-height: 1;
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

        let parent_dir = span
            .parent()
            .and_then(|p| p.extensions().get::<LogSpan>().map(|s| s.dir.clone()))
            .unwrap_or_else(|| self.run_root.clone());

        let current_dir = parent_dir.join(name);
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
                for (k, v) in visitor.fields {
                    writeln!(file, "    <dt>{}</dt><dd><code>{}</code></dd>", k, v).ok();
                }
                writeln!(file, "  </dl></details>").ok();
            }
            for link in visitor.links {
                let link_dir = self.run_root.join(&link.category);
                let link_path = link_dir.join(&link.name).with_extension(BASE_EXTENSION);
                let mut link_file = self.get_file(&link_path);

                let rel_back = diff_paths(&current_file, &link_dir).unwrap();
                writeln!(
                    link_file,
                    "<div>Linked from: {}</div>",
                    link_str(rel_back.to_string_lossy(), name)
                )
                .ok();

                let rel_to = diff_paths(&link_path, &current_dir).unwrap();
                writeln!(
                    file,
                    "<div>→ Related: {}</div>",
                    link_str(rel_to.to_string_lossy(), link.name)
                )
                .ok();
            }
        }

        // 2. Link from the parent file
        {
            let mut file = self.get_file(&parent_file);
            let rel_down = diff_paths(&current_file, &parent_dir).unwrap();
            writeln!(
                file,
                "<div>↳ Entering Span: {}</div>",
                link_str(rel_down.to_string_lossy(), name)
            )
            .ok();
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
        let mut file = self.get_file(&current_file);

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

        writeln!(file, "<div class='log-entry'>").ok();
        writeln!(file, "  <pre>{}</pre>", event_str).ok();

        if !visitor.fields.is_empty() {
            writeln!(file, "  <dl>").ok();
            for (k, v) in visitor.fields {
                writeln!(file, "    <dt>{}</dt><dd><code>{}</code></dd>", k, v).ok();
            }
            writeln!(file, "  </dl>").ok();
        }

        if !visitor.links.is_empty() {
            writeln!(file, "<details><summary>Related</summary><dl>").ok();
        }

        for link in visitor.links.iter() {
            let link_dir = self.run_root.join(&link.category);
            let link_path = link_dir.join(&link.name).with_extension(BASE_EXTENSION);
            let mut link_file = self.get_file(&link_path);

            let rel_back = diff_paths(&current_file, &link_dir).unwrap();
            writeln!(
                link_file,
                "<div class = 'log-entry'>{} <pre>{}</pre></div>",
                link_str(rel_back.to_string_lossy(), "→"),
                event_str
            )
            .ok();

            let rel_to = diff_paths(&link_path, &span_data.dir).unwrap();
            writeln!(
                file,
                "<div>→ Related: {}</div>",
                link_str(rel_to.to_string_lossy(), &link.name)
            )
            .ok();
        }

        if !visitor.links.is_empty() {
            writeln!(file, "</dl></details>");
        }

        writeln!(file, "</div>").ok();
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

impl Link {
    pub fn category(&self) -> &str {
        &self.category
    }
    pub fn file_name(&self) -> &str {
        &self.name
    }
}

pub fn init_tree_logger() {
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(EnvFilter::new("debug"))
        .with(RoutingLayer::new())
        .init();
}

pub fn enter_process(span_name: impl Into<String>) -> EnteredSpan {
    let span_name = span_name.into();
    tracing::span!(Level::DEBUG, "process_span", name = span_name).entered()
}

pub fn process_span(span_name: impl Into<String>) -> tracing::span::Span {
    let span_name = span_name.into();
    tracing::span!(Level::DEBUG, "process_span", name = span_name)
}
