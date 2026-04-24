use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    path::{absolute, Path},
    sync::Mutex,
};
use tracing::{
    field::{Field, Visit},
    span::EnteredSpan,
    Event, Subscriber,
};
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan, Layer};

use chrono::{self, Local};

use crate::comm::micromouse_message::CommandId;

struct LogFileSpan {
    dir_path: String,
}

struct RoutingLayer {
    files: Mutex<HashMap<String, std::fs::File>>,
    run_id: String,
}

struct FieldVisitor {
    log_file: Option<String>,
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "log_dir" {
            self.log_file = Some(format!("{:?}", value));
        }
    }
}

impl<S> Layer<S> for RoutingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(scope) = ctx.event_scope(event) {
            if let Some(span) = scope.from_root().last() {
                if let Some(data) = span.extensions().get::<LogFileSpan>() {
                    println!("Processing event {event:?}");
                    // let path = format!(
                    //     "logs/{}/{}/{}.md",
                    //     self.run_id,
                    //     data.dir_path,
                    //     event.metadata().target().replace('.', "/")
                    // );
                    let path = format!("{}/index.md", data.dir_path);

                    let mut files_lock = self.files.lock().expect("Should not be poisoned");
                    let file = files_lock.entry(path.clone()).or_insert_with(|| {
                        std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).ok();
                        OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&path)
                            .unwrap()
                    });
                    writeln!(file, "{:?}", event).expect("Error while writing to file")
                }
            }
        }
    }
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let span = ctx.span(id).unwrap();
        let mut visitor = FieldVisitor { log_file: None };
        attrs.record(&mut visitor);

        let name = visitor
            .log_file
            .unwrap_or_else(|| span.metadata().name().to_string());
        // let name = span.metadata().name();

        let parent_folder_path = if let Some(parent) = span.parent() {
            if let Some(parent_file) = parent.extensions().get::<LogFileSpan>() {
                parent_file.dir_path.clone()
            } else {
                format!("logs/{}", self.run_id)
            }
        } else {
            format!("logs/{}", self.run_id)
        };
        // create file path
        let child_folder_path = format!("{parent_folder_path}/{name}");
        let child_file_path = format!("{child_folder_path}/index.md");
        let parent_file_path = format!("{parent_folder_path}/index.md");
        // let [parent_name, _p] = parent_file_path
        let parent_name = parent_folder_path
            .rsplitn(2, "/")
            .next()
            .expect("Should be at least 1");
        // let parent_file_path_abs = absolute(parent_file_path);

        // store in extensions
        span.extensions_mut().insert(LogFileSpan {
            dir_path: child_folder_path.clone(),
        });

        // ensure file exists / create the new file
        // INFO: Create the new span's file
        println!("child_folder_path = {child_folder_path}, child_file_path = {child_file_path}");
        std::fs::create_dir_all(std::path::Path::new(&child_file_path).parent().unwrap()).ok();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&child_file_path)
            .unwrap();

        writeln!(file, "# Span: {}", name).ok();

        // if let Ok(parent_file_path_abs) = parent_file_path_abs {
        writeln!(file, "[SOURCE](../index.md)").expect("Error while writing to file");

        let mut file_lock = self.files.lock().expect("Should not be poisoned");

        let parent_file = file_lock
            .entry(parent_file_path.clone())
            .or_insert_with(|| {
                std::fs::create_dir_all(
                    std::path::Path::new(&parent_folder_path).parent().unwrap(),
                )
                .ok();
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&parent_file_path)
                    .unwrap()
            });

        writeln!(parent_file, "[{name}](./{name}/index.md)").expect("Error while writing to file");
    }
}

impl RoutingLayer {
    pub fn new() -> Self {
        let date = Local::now();
        Self {
            files: Mutex::new(HashMap::new()),
            run_id: date.format("%Y-%m-%d_%H-%M-%S").to_string(),
        }
    }
}

// pub struct CommandLink {
//     pub cmd_id: CommandId,
// }

pub fn init_tree_logger() {
    use tracing_subscriber::util::SubscriberInitExt;
    tracing_subscriber::registry()
        .with(RoutingLayer::new())
        .init();
}

pub fn process_span(name: impl Into<String>) -> EnteredSpan {
    use tracing::span;
    use tracing::Level;
    let name: String = name.into();
    span!(Level::DEBUG, "process", log_dir = %name).entered()
}
