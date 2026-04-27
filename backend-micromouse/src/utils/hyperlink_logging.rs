use std::{
    collections::HashMap,
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

struct NameVisitor {
    pub name: Option<String>,
}

struct LinkVisitor {
    pub links: Vec<Link>,
}

struct Link {
    category: String,
    name: String,
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

            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap();

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
        let current_file = current_dir.join("index.md");
        let parent_file = parent_dir.join("index.md");

        span.extensions_mut().insert(LogSpan {
            dir: current_dir.clone(),
        });

        // create current file
        {
            let mut file = self.get_file(&current_file);
            writeln!(file, "# Span: {name}\n").ok();

            if parent_dir != current_dir {
                let rel = diff_paths(&parent_file, &current_dir).unwrap();
                writeln!(file, "{} ->", link_str(rel.to_string_lossy(), "SOURCE")).ok();
                // writeln!(file, "[SOURCE]({})", rel.display()).ok();
            }
        }

        // link from parent
        {
            let mut file = self.get_file(&parent_file);
            let rel = diff_paths(&current_file, &parent_dir).unwrap();
            writeln!(file, "-> {}", link_str(rel.to_string_lossy(), name)).ok();
            // writeln!(file, "[{name}]({})", rel.display()).ok();
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();

        let extensions = span.extensions();

        let Some(span_data) = extensions.get::<LogSpan>() else {
            return;
        };

        let span_file = span_data.dir.join("index.md");
        let parent_span = span_data.dir.parent().unwrap_or(self.run_root.as_path());

        let mut file = self.get_file(&span_file);
        writeln!(
            file,
            "<- {}\n",
            link_str("../index.md", parent_span.to_string_lossy())
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
        let current_file = current_dir.join("index.md");

        let mut link_visitor = LinkVisitor { links: vec![] };
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
        let event_str =
            format!("{info}{pad_str}   {RESET_COLOR} ] {level_color} {msg}{RESET_COLOR}");

        println!("{event_str}");
        let event_str = ansi_to_html::convert(&event_str).expect("unable to convert ANSI to HTML");

        // write to span file
        {
            let mut file = self.get_file(&current_file);
            write!(file, "{event_str}").ok();

            // process links
            if num_links > 0 {
                write!(file, "\n(").ok();
            }
        }
        for link in link_visitor.links {
            let link_dir = self.run_root.join(link.category());
            let link_path = link_dir.join(link.file_name()).with_extension("md");

            let mut link_file = self.get_file(&link_path);

            // eprintln!("current_file = {current_file:?}");
            // eprintln!("link_path = {link_path:?}");

            let rel_from_link = diff_paths(&current_file, &link_dir).unwrap();
            // eprintln!("rel_from_link = {rel_from_link:?}");

            // writeln!(link_file, "[{}]({})", event_str, rel.display()).ok();
            writeln!(
                link_file,
                "{}",
                link_str(rel_from_link.to_string_lossy(), &event_str,)
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
            write!(file, ") ").ok();
        }
        writeln!(file).ok();
    }
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
    format!(
        "<a href = \"{}\" style=\"text-decoration:none\">{}</a>",
        to.into(),
        content.into()
    )
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
