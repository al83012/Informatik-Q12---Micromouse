use std::{
    collections::HashMap,
    fmt::Display,
    fs::{File, OpenOptions},
    io::Write,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Instant,
};

use crate::utils::logging::{BLACK, RESET_COLOR, STD_BG};

use chrono::Local;

use tracing::{
    field::{Field, Value, Visit},
    span::{self, EnteredSpan},
    Event, Level, Subscriber,
};

use tracing_subscriber::{
    filter::FilterFn, layer::Context, registry::LookupSpan, EnvFilter, Layer,
};

use pathdiff::diff_paths;

use crate::{
    comm::micromouse_message::CommandId,
    strategy::strategy_tree::AbsoluteNodeId,
    utils::logging::{level_bg_color, level_color, MessageVisitor, MyFormatter},
};

pub const BASE_FILE: &str = "index.html";
pub const BASE_EXTENSION: &str = "html";

struct NameVisitor {
    pub name: Option<String>,
}

#[derive(Clone)]
struct LinkVisitor {
    pub links: Vec<Link>,
}

#[derive(Clone)]
struct Link {
    category: String,
    name: String,
}

struct DebugVisitor {
    pub fields: HashMap<String, String>,
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

impl Visit for DebugVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn core::fmt::Debug) {
        self.fields
            .insert(field.name().to_string(), format!("{:?}", value));
    }
}

impl Visit for LinkVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        const LINK_PREFIX: &str = "link_";
        if field.name() != LINK_PREFIX && field.name().starts_with(LINK_PREFIX) {
            let category = &field.name()[LINK_PREFIX.len()..];
            self.links.push(Link {
                category: category.to_string(),
                name: value.to_string(),
            })
        }
    }
    fn record_debug(&mut self, field: &Field, value: &dyn core::fmt::Debug) {
        // self.record_str(field, format!("{value:?}").as_str());
        // panic!("Just using Debug will result in wrong file names");
    }
}

impl Visit for NameVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn core::fmt::Debug) {
        // self.record_str(field, &format!("{value:?}"));
        // panic!("Just using Debug will result in wrong file names");
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "name" {
            self.name = Some(value.to_string())
        }
    }
}

struct LogSpan {
    dir: PathBuf,
    links: LinkVisitor,
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

            writeln!(
                file,
                r#"<style>
  /* 1. Global Page Style */
  html, body {{
    background-image: radial-gradient(circle at top right, #1a1a2e, #0a0a0c);
    color: #e0e0e0;
    margin: 0;
    padding: 20px;
    font-family: 'Consolas', 'Monaco', monospace;
    line-height: 1.5;
  }}

  /* 2. Log Entry Container (Replacing/Extending Div) */
  section, .log-entry {{
    margin-bottom: 20px;
    padding: 15px;
    border-radius: 8px;
    background-color: rgba(255, 255, 255, 0.03); 
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }}

  /* 3. The Fields (Description List) */
  dl {{
    display: grid;
    grid-template-columns: max-content auto; /* Aligns keys and values in a neat grid */
    gap: 5px 15px;
    margin: 10px 0;
    font-size: 0.9em;
    padding: 10px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 4px;
  }}
  dt {{
    color: #569cd6; /* Light blue for field names */
    font-weight: bold;
    opacity: 0.8;
  }}
  dd {{
    margin: 0;
    color: #ce9178; /* Muted orange/ginger for values */
  }}
  dd code {{
    background: rgba(255, 255, 255, 0.05);
    padding: 2px 4px;
    border-radius: 3px;
  }}

  /* 4. Interactive Spans (Details/Summary) */
  details {{
    cursor: pointer;
  }}
  summary {{
    list-style: none; /* Hides default arrow in some browsers */
    outline: none;
    padding: 5px;
    border-radius: 4px;
    transition: background 0.2s;
  }}
  summary::-webkit-details-marker {{ display: none; }} /* Hides arrow in Safari */
  
  summary:hover {{
    background: rgba(255, 255, 255, 0.05);
  }}
  
  summary::before {{
    content: "▶";
    display: inline-block;
    margin-right: 8px;
    font-size: 0.8em;
    color: #4ec9b0;
    transition: transform 0.2s;
  }}
  details[open] summary::before {{
    transform: rotate(90deg);
  }}

  /* 5. Code & Pre blocks */
  pre {{
    background-color: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(8px);
    border-radius: 8px;
    padding: 15px;
    overflow-x: auto;
    border: 1px solid rgba(78, 201, 176, 0.2); /* Subtle teal border */
  }}

  /* 6. Link handling */
  a {{
    color: #4ec9b0;
    text-decoration: none;
    border-bottom: 1px dashed rgba(78, 201, 176, 0.3);
  }}
  a:hover {{
    color: #fff;
    border-bottom: 1px solid #4ec9b0;
  }}
  a:visited {{
    color: #c586c0;
  }}

.inline-fields {
    display: inline-flex;
    gap: 10px;
    font-size: 0.8em;
    margin-left: 20px;
    background: none;
    padding: 0;
}
.inline-fields dt { color: #569cd6; }
.inline-fields dd { color: #ce9178; margin-right: 10px; }
</style>"#
            )
            .ok();

            files.insert(path.to_path_buf(), file);
        }

        files.get(path).unwrap().try_clone().unwrap()
    }
}

impl<S> Layer<S> for RoutingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: Context<'_, S>,
    ) {
        let span = ctx.span(id).unwrap();

        // let name = span.metadata().name();
        let mut name_visitor = NameVisitor { name: None };
        attrs.record(&mut name_visitor);
        let name = name_visitor
            .name
            .as_deref()
            .unwrap_or(span.metadata().name());

        let parent_dir = span
            .parent()
            .as_ref()
            .map(|p| p.extensions())
            .as_ref()
            .and_then(|e| e.get::<LogSpan>())
            .map(|s| s.dir.clone())
            .unwrap_or_else(|| self.run_root.clone());

        let current_dir = parent_dir.join(name);
        let current_file = current_dir.join(BASE_FILE);
        let parent_file = parent_dir.join(BASE_FILE);

        let mut links = span
            .parent()
            .as_ref()
            .map(|p| p.extensions())
            .as_ref()
            .and_then(|e| e.get::<LogSpan>())
            .map(|s| s.links.clone())
            .unwrap_or(LinkVisitor { links: vec![] });
        // let mut links = LinkVisitor { links: vec![] };

        attrs.record(&mut links);

        let mut attr_visitor = DebugVisitor {
            fields: HashMap::new(),
        };
        attrs.record(&mut attr_visitor);

        span.extensions_mut().insert(LogSpan {
            dir: current_dir.clone(),
            links,
        });

        // create current file
        {
            let mut file = self.get_file(&current_file);

            if parent_dir != current_dir {
                let rel = diff_paths(&parent_file, &current_dir).unwrap();
                writeln!(
                    file,
                    "\n<div><code>{}</code>",
                    link_str(rel.to_string_lossy(), parent_dir.to_string_lossy())
                )
                .ok();
                writeln!(file, "<details>").ok();
                writeln!(file, "<summary><strong>{name}</strong></summary>\n").ok();
                if !attr_visitor.fields.is_empty() {
                    writeln!(file, "<dl>").ok();
                }

                for (field_k, field_v) in attr_visitor.fields.iter() {
                    writeln!(file, "<dt>{field_k}</dt> <dd><code>{field_v}</code></dd>").ok();
                }

                if !attr_visitor.fields.is_empty() {
                    writeln!(file, "</dl>").ok();
                }

                writeln!(file, "</details>").ok();
            }
        }

        // link from parent
        {
            let mut file = self.get_file(&parent_file);
            let rel = diff_paths(&current_file, &parent_dir).unwrap();
            writeln!(
                file,
                "<code><pre>{}</pre></code>",
                link_str(rel.to_string_lossy(), name)
            )
            .ok();
            // writeln!(file, "[{name}]({})", rel.display()).ok();
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();

        let extensions = span.extensions();

        let Some(span_data) = extensions.get::<LogSpan>() else {
            return;
        };

        let span_file = span_data.dir.join(BASE_FILE);
        let parent_span = span_data.dir.parent().unwrap_or(self.run_root.as_path());

        let mut file = self.get_file(&span_file);
        writeln!(
            file,
            "<code>{}</code></div>",
            link_str(format!("../{BASE_FILE}"), parent_span.to_string_lossy())
        )
        .ok();
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

        let current_dir = &span_data.dir;
        let current_file = current_dir.join(BASE_FILE);

        let mut link_visitor = span_data.links.clone();
        // let mut link_visitor = LinkVisitor { links: vec![] };
        event.record(&mut link_visitor);

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

        let mut visitor = MessageVisitor { msg: None };
        let num_links = link_visitor.links.len();
        event.record(&mut visitor);
        let msg = visitor.msg.unwrap_or_default();
        let ansii_event_str =
            format!("{info}{pad_str}   {RESET_COLOR} ] {level_color} {msg}{RESET_COLOR}");

        println!("{ansii_event_str}");
        let event_str = preformatted_str_start(
            ansi_to_html::convert(&ansii_event_str).expect("unable to convert ANSI to HTML"),
        );

        // write to span file
        {
            let mut file = self.get_file(&current_file);
            write!(file, "{event_str}").ok();

            // process links
            if num_links > 0 {
                write!(file, "\n<pre><code>").ok();
            }
        }
        for link in link_visitor.links {
            let link_dir = self.run_root.join(link.category());
            let link_path = link_dir
                .join(link.file_name())
                .with_extension(BASE_EXTENSION);

            let mut link_file = self.get_file(&link_path);

            // eprintln!("current_file = {current_file:?}");
            // eprintln!("link_path = {link_path:?}");

            let rel_from_link = diff_paths(&current_file, &link_dir).unwrap();
            // eprintln!("rel_from_link = {rel_from_link:?}");

            // writeln!(link_file, "[{}]({})", event_str, rel.display()).ok();
            writeln!(
                link_file,
                " {}{} ",
                link_str(rel_from_link.to_string_lossy(), &event_str),
                preformatted_str_end()
            )
            .ok();

            // backlink
            let mut file = self.get_file(&current_file);
            let rel_back = diff_paths(&link_path, current_dir).unwrap();
            // eprintln!("rel_from_current = {rel_back:?}");

            write!(
                file,
                "{} ",
                link_str(rel_back.to_string_lossy(), link.file_name())
            )
            .ok();
            // writeln!(file, "[{}]({})", link.file_name(), rel_back.display()).ok();
        }
        let mut file = self.get_file(&current_file);
        if num_links > 0 {
            write!(file, "</code></pre> ").ok();
        }
        writeln!(file, "{}", preformatted_str_end()).ok();
    }
}

pub fn preformatted_str_start(from: impl Display) -> String {
    format!("<pre>{from}")
}
pub fn preformatted_str_end() -> String {
    "</pre>".into()
}

impl Link {
    pub fn category(&self) -> &str {
        &self.category
    }
    pub fn file_name(&self) -> &str {
        &self.name
    }
}

fn link_str(to: impl Into<String>, content: impl Into<String>) -> String {
    format!("<a href = \"{}\">{}</a>", to.into(), content.into())
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
