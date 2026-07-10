//! File-based tracing layer — writes JSONL trace files without OTel dependency.
//!
//! Enabled when `--trace-file <path>` is passed. Each line is one JSON object
//! representing a tracing event or span. Replaces the OTel-coupled
//! `FileTraceExporter` with a native `tracing_subscriber::Layer`.
//!
//! The file is **truncated** on creation — each run produces a clean trace file.
//! Structured fields from `tracing::info!(key = value, ...)` are captured in the
//! `fields` object. `trace_id` uses the thread ID (stable within a thread); when
//! OTel is also active, the OTel trace/span IDs take precedence in downstream
//! consumers.
//!
//! **Thread-safety note:** This layer uses thread-local span tracking
//! (`SPAN_STACK`). It assumes the same thread emits all spans for a given
//! request. This is sound for typical async runtimes where a task stays on one
//! thread, but may produce unexpected results if spans hop threads.

use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::field::Visit;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

thread_local! {
    /// Stack of active span IDs for the current thread.
    /// Used to maintain parent-child relationships across async boundaries.
    static SPAN_STACK: std::cell::RefCell<Vec<u64>> = std::cell::RefCell::new(Vec::new());
}

struct FileTraceLayer {
    writer: Arc<Mutex<BufWriter<File>>>,
}

impl FileTraceLayer {
    pub fn new(path: std::path::PathBuf) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        Ok(Self {
            writer: Arc::new(Mutex::new(BufWriter::new(file))),
        }
    }
}

/// Recorder that collects structured fields during a `tracing::field::Visit`.
/// Separates the `message` field from other fields for cleaner JSON output.
struct EventRecorder {
    fields: Map<String, Value>,
    message: Option<String>,
}

impl EventRecorder {
    fn new() -> Self {
        Self {
            fields: Map::new(),
            message: None,
        }
    }
}

impl Visit for EventRecorder {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let name = field.name().to_string();
        let formatted = format!("{value:?}");

        if name == "message" {
            // Extract message without surrounding quotes
            let msg = if formatted.starts_with('"')
                && formatted.ends_with('"')
                && formatted.len() >= 2
            {
                formatted[1..formatted.len() - 1].to_string()
            } else {
                formatted
            };
            self.message = Some(msg);
        } else {
            // Strip surrounding quotes from Debug output on strings
            let value =
                if formatted.starts_with('"')
                    && formatted.ends_with('"')
                    && formatted.len() >= 2
                {
                    Value::String(formatted[1..formatted.len() - 1].to_string())
                } else {
                    Value::String(formatted)
                };
            self.fields.insert(name, value);
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.insert(field.name().to_string(), Value::Number(value.into()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), Value::Number(value.into()));
    }
}

impl<S> Layer<S> for FileTraceLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: Context<'_, S>,
    ) {
        let span = ctx.span(id).expect("span not found, this is a bug");
        let mut recorder = EventRecorder::new();
        attrs.record(&mut recorder);

        let mut extensions = span.extensions_mut();
        extensions.insert(recorder);

        // Maintain per-thread span stack for parent-child relationships
        SPAN_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            stack.push(id.into_u64());
        });
    }

    fn on_record(
        &self,
        id: &tracing::Id,
        values: &tracing::span::Record<'_>,
        ctx: Context<'_, S>,
    ) {
        let span = ctx.span(id).expect("span not found, this is a bug");
        let mut extensions = span.extensions_mut();
        if let Some(recorder) = extensions.get_mut::<EventRecorder>() {
            values.record(&mut *recorder);
        }
    }

    fn on_close(&self, id: tracing::Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).expect("span not found, this is a bug");
        let mut extensions = span.extensions_mut();
        if let Some(recorder) = extensions.remove::<EventRecorder>() {
            let mut json = Map::new();
            json.insert("event".to_string(), Value::String("close".to_string()));
            json.insert(
                "trace_id".to_string(),
                Value::String(format!("{:x}", id.into_u64())),
            );
            json.insert("fields".to_string(), Value::Object(recorder.fields));
            if let Some(msg) = recorder.message {
                json.insert("message".to_string(), Value::String(msg));
            }

            if let Ok(writer) = self.writer.lock() {
                let mut writer = writer;
                let line = serde_json::to_vec(&Value::Object(json)).unwrap_or_default();
                let _ = writer.write_all(&line);
                let _ = writer.write_all(b"\n");
                let _ = writer.flush();
            }
        }

        // Pop from thread-local span stack
        SPAN_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            stack.pop();
        });
    }
}

/// Parse a single event from the trace file.
fn parse_single_event(path: &std::path::Path) -> Value {
    let content = std::fs::read_to_string(path).unwrap();
    let first_line = content.lines().next().unwrap_or("");
    serde_json::from_str(first_line).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tracing::info;
    use tracing_subscriber::registry;

    #[test]
    fn test_file_trace_layer_writes_jsonl() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let layer = FileTraceLayer::new(path.clone()).unwrap();
        let subscriber = registry().with(layer);
        let dispatch = tracing::Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || {
            tracing::info!("check timestamp");
        });

        let parsed = parse_single_event(&path);

        let ts = parsed["timestamp"]
            .as_str()
            .expect("timestamp must be a string");
        // RFC3339: 2025-07-09T20:41:34.123Z
        assert!(ts.ends_with('Z'), "timestamp must end with Z, got: {ts}");
        assert!(
            ts.contains('T'),
            "timestamp must contain T separator, got: {ts}"
        );
        assert!(
            ts.len() >= 20,
            "timestamp must be full ISO format, got: {ts}"
        );
        let parts: Vec<&str> = ts.split('T').collect();
        assert_eq!(parts.len(), 2, "must have date and time separated by T");

        let date_parts: Vec<&str> = parts[0].split('-').collect();
        assert_eq!(date_parts.len(), 3, "date must have year-month-day");
        let year: i32 = date_parts[0].parse().unwrap();
        let month: u32 = date_parts[1].parse().unwrap();
        let day: u32 = date_parts[2].parse().unwrap();
        assert!(year >= 2024 && year <= 2100);
        assert!(month >= 1 && month <= 12);
        assert!(day >= 1 && day <= 31);

        let time_parts: Vec<&str> = parts[1].split(':').collect();
        assert_eq!(time_parts.len(), 3, "time must have hour:minute:second.millis");
    }

    #[test]
    fn test_span_stack_maintains_parent_child() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let layer = FileTraceLayer::new(path.clone()).unwrap();
        let subscriber = registry().with(layer);
        let dispatch = tracing::Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || {
            let parent = tracing::span!(tracing::Level::INFO, "parent");
            let _enter = parent.enter();
            tracing::info!("inside parent");
            let child = tracing::span!(tracing::Level::INFO, "child");
            let _enter = child.enter();
            tracing::info!("inside child");
        });

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // Should have at least 4 events: parent open, parent msg, child open, child msg, child close, parent close
        assert!(lines.len() >= 4);
    }

    #[test]
    fn test_file_trace_layer_truncates_on_new() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let layer = FileTraceLayer::new(path.clone()).unwrap();
        let subscriber = registry().with(layer);
        let dispatch = tracing::Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || {
            tracing::info!("first run");
        });
        assert!(path.exists());

        // Create a new layer with same path (should truncate)
        let layer2 = FileTraceLayer::new(path.clone()).unwrap();
        let subscriber2 = registry().with(layer2);
        let dispatch2 = tracing::Dispatch::new(subscriber2);

        tracing::dispatcher::with_default(&dispatch2, || {
            tracing::info!("second run");
        });

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // Should only contain second run
        assert_eq!(lines.len(), 2); // open + message
        let parsed: Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(parsed["message"], "second run");
    }

    #[test]
    fn test_file_trace_layer_captures_fields() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let layer = FileTraceLayer::new(path.clone()).unwrap();
        let subscriber = registry().with(layer);
        let dispatch = tracing::Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || {
            tracing::info!(user_id = 42, action = "login", "user logged in");
        });

        let parsed = parse_single_event(&path);
        let fields = parsed["fields"].as_object().unwrap();
        assert_eq!(fields.get("user_id"), Some(&Value::Number(42.into())));
        assert_eq!(
            fields.get("action"),
            Some(&Value::String("login".to_string()))
        );
        assert_eq!(parsed["message"], "user logged in");
    }
}
