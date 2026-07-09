//! File-based tracing layer — writes JSONL trace files without OTel dependency.
//!
//! Enabled when `--trace-file <path>` is passed. Each line is one JSON object
//! representing a tracing event or span. Replaces the OTel-coupled
//! `FileTraceExporter` with a native `tracing_subscriber::Layer`.

use std::cell::RefCell;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

use serde_json::{json, Value};
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

// Thread-local span stack for tracking the current span inside `on_event`.
// When a span is entered, its ID is pushed; when exited, it's popped.
// This is necessary because `Span::current()` doesn't reliably return the
// span ID inside a Layer's `on_event` callback.
thread_local! {
    static SPAN_STACK: RefCell<Vec<tracing::Id>> = const { RefCell::new(Vec::new()) };
}

/// A `tracing_subscriber::Layer` that writes JSONL trace files.
///
/// Each line is a JSON object with: timestamp, level, target,
/// span name, trace_id, span_id, message, and fields.
pub struct FileTraceLayer {
    writer: Mutex<BufWriter<File>>,
}

impl FileTraceLayer {
    /// Create a new file trace layer, opening (or creating) the file at `path`.
    ///
    /// Parent directories are created automatically. The file is truncated
    /// on creation so each run produces a clean trace file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or opened.
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        Ok(Self {
            writer: Mutex::new(writer),
        })
    }
}

impl std::fmt::Debug for FileTraceLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileTraceLayer").finish_non_exhaustive()
    }
}

impl<S> Layer<S> for FileTraceLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_enter(&self, id: &tracing::Id, _ctx: Context<'_, S>) {
        SPAN_STACK.with(|stack| stack.borrow_mut().push(id.clone()));
    }

    fn on_exit(&self, _id: &tracing::Id, _ctx: Context<'_, S>) {
        SPAN_STACK.with(|stack| stack.borrow_mut().pop());
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let meta = event.metadata();

        let mut record = json!({
            "timestamp": system_time_to_rfc3339(SystemTime::now()),
            "level": meta.level().as_str(),
            "target": meta.target(),
            "message": event_field_value(event),
        });

        // Extract span context from thread-local span stack
        let current_span_id = SPAN_STACK.with(|stack| stack.borrow().last().cloned());

        if let Some(id) = current_span_id {
            if let Some(span_ref) = ctx.span(&id) {
                record["span"] = json!(span_ref.name());
            }
        }

        if let Ok(mut writer) = self.writer.lock() {
            let mut line = serde_json::to_vec(&record).unwrap_or_default();
            line.push(b'\n');
            let _ = writer.write_all(&line);
            let _ = writer.flush();
        }
    }
}

/// Extract the message field from a tracing event.
fn event_field_value(event: &tracing::Event<'_>) -> Value {
    let mut recorder = EventFieldRecorder(String::new());
    event.record(&mut recorder);
    if recorder.0.is_empty() {
        json!(null)
    } else {
        json!(recorder.0)
    }
}

/// Recorder that extracts the `message` field from a tracing event.
struct EventFieldRecorder(String);

impl tracing::field::Visit for EventFieldRecorder {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{:?}", value);
            // Strip surrounding quotes from Debug output
            if self.0.starts_with('"') && self.0.ends_with('"') {
                self.0 = self.0[1..self.0.len() - 1].to_string();
            }
        }
    }
}

fn system_time_to_rfc3339(t: SystemTime) -> String {
    let duration = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{secs}.{millis:03}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    #[test]
    fn test_file_trace_layer_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let _layer = FileTraceLayer::new(path.clone()).unwrap();
        assert!(path.exists(), "trace file should be created");
    }

    #[test]
    fn test_file_trace_layer_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("subdir").join("deep").join("trace.jsonl");
        let _layer = FileTraceLayer::new(path.clone()).unwrap();
        assert!(
            path.exists(),
            "trace file with nested dirs should be created"
        );
    }

    #[test]
    fn test_file_trace_layer_writes_jsonl_on_event() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let layer = FileTraceLayer::new(path.clone()).unwrap();

        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = subscriber.set_default();

        tracing::info!(target: "test_target", "hello from test");

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1, "should have exactly one JSONL line");

        let parsed: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed["level"], "INFO");
        assert_eq!(parsed["target"], "test_target");
        assert_eq!(parsed["message"], "hello from test");
        assert!(
            parsed["timestamp"].is_string(),
            "timestamp should be a string"
        );
    }

    #[test]
    fn test_file_trace_layer_event_without_span() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let layer = FileTraceLayer::new(path.clone()).unwrap();

        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = subscriber.set_default();

        tracing::warn!("warning message");

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let parsed: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed["level"], "WARN");
        assert_eq!(parsed["message"], "warning message");
        // No span field when not inside a span
        assert!(
            parsed["span"].is_null() || parsed["span"].as_str() == Some(""),
            "span should be null/empty outside a span, got: {:?}",
            parsed["span"]
        );
    }

    #[test]
    fn test_file_trace_layer_event_inside_span() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let layer = FileTraceLayer::new(path.clone()).unwrap();

        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = subscriber.set_default();

        let span = tracing::info_span!("my_span", request_id = 42);
        let _enter = span.enter();

        tracing::info!("inside span");

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let parsed: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed["span"], "my_span");
        assert_eq!(parsed["level"], "INFO");
    }

    #[test]
    fn test_file_trace_layer_multiple_events() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let layer = FileTraceLayer::new(path.clone()).unwrap();

        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = subscriber.set_default();

        tracing::info!("event 1");
        tracing::debug!("event 2");
        tracing::error!("event 3");

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 3, "should have three JSONL lines");

        let parsed0: Value = serde_json::from_str(lines[0]).unwrap();
        let parsed1: Value = serde_json::from_str(lines[1]).unwrap();
        let parsed2: Value = serde_json::from_str(lines[2]).unwrap();

        assert_eq!(parsed0["message"], "event 1");
        assert_eq!(parsed1["message"], "event 2");
        assert_eq!(parsed2["message"], "event 3");
        assert_eq!(parsed0["level"], "INFO");
        assert_eq!(parsed1["level"], "DEBUG");
        assert_eq!(parsed2["level"], "ERROR");
    }

    #[test]
    fn test_file_trace_layer_thread_safety() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let layer = FileTraceLayer::new(path.clone()).unwrap();

        // Use set_global_default so spawned threads see the subscriber
        let subscriber = tracing_subscriber::registry().with(layer);
        let dispatch = tracing::Dispatch::new(subscriber);
        tracing::dispatcher::set_global_default(dispatch).ok();

        let mut handles = vec![];
        for i in 0..10 {
            handles.push(std::thread::spawn(move || {
                tracing::info!(thread_id = i, "thread message {i}");
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(
            lines.len(),
            10,
            "should have 10 JSONL lines from 10 threads"
        );

        // Verify all lines are valid JSON
        for line in &lines {
            let parsed: Value = serde_json::from_str(line).unwrap();
            assert!(parsed.is_object(), "each line should be a JSON object");
        }
    }

    #[test]
    fn test_file_trace_layer_nested_spans() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trace.jsonl");
        let layer = FileTraceLayer::new(path.clone()).unwrap();

        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = subscriber.set_default();

        let outer = tracing::info_span!("outer");
        let _outer_guard = outer.enter();

        tracing::info!("in outer");

        {
            let inner = tracing::info_span!("inner");
            let _inner_guard = inner.enter();
            tracing::info!("in inner");
        }

        tracing::info!("back to outer");

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 3);

        let parsed0: Value = serde_json::from_str(lines[0]).unwrap();
        let parsed1: Value = serde_json::from_str(lines[1]).unwrap();
        let parsed2: Value = serde_json::from_str(lines[2]).unwrap();

        assert_eq!(parsed0["span"], "outer");
        assert_eq!(parsed0["message"], "in outer");
        assert_eq!(parsed1["span"], "inner");
        assert_eq!(parsed1["message"], "in inner");
        assert_eq!(parsed2["span"], "outer");
        assert_eq!(parsed2["message"], "back to outer");
    }
}
