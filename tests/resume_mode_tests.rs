//! Resume mode tests.
//!
//! Exercises StateStore save/load roundtrip, ExportState public API,
//! resume with custom state dir, and corrupted state file handling.
//!
//! Uses tempfile for filesystem isolation and deterministic timestamps.

use std::fs;
use tempfile::tempdir;
use webfang_core::domain::ExportState;
use webfang_core::infrastructure::export::StateStore;

// ============================================================================
// 1. ExportState public API
// ============================================================================

#[test]
fn export_state_new_initializes_empty() {
    let state = ExportState::new("test.com");
    assert_eq!(state.domain, "test.com");
    assert!(state.processed_urls.is_empty());
    assert!(state.last_export.is_none());
    assert_eq!(state.total_exported, 0);
}

#[test]
fn export_state_mark_processed_adds_url() {
    let mut state = ExportState::new("test.com");
    state.mark_processed("https://test.com/page1");
    assert!(state.is_processed("https://test.com/page1"));
    assert_eq!(state.processed_urls.len(), 1);
    assert_eq!(state.total_exported, 1);
}

#[test]
fn export_state_mark_processed_deduplicates() {
    let mut state = ExportState::new("test.com");
    state.mark_processed("https://test.com/page1");
    state.mark_processed("https://test.com/page1");
    state.mark_processed("https://test.com/page1");
    assert_eq!(state.processed_urls.len(), 1, "should not duplicate");
    assert_eq!(
        state.total_exported, 1,
        "counter should not increment on duplicates"
    );
}

#[test]
fn export_state_is_processed_returns_false_for_unknown() {
    let state = ExportState::new("test.com");
    assert!(!state.is_processed("https://test.com/unknown"));
}

#[test]
fn export_state_multiple_urls() {
    let mut state = ExportState::new("test.com");
    state.mark_processed("https://test.com/a");
    state.mark_processed("https://test.com/b");
    state.mark_processed("https://test.com/c");
    assert!(state.is_processed("https://test.com/a"));
    assert!(state.is_processed("https://test.com/b"));
    assert!(state.is_processed("https://test.com/c"));
    assert_eq!(state.processed_urls.len(), 3);
    assert_eq!(state.total_exported, 3);
}

#[test]
fn export_state_clone_preserves_data() {
    let mut state = ExportState::new("test.com");
    state.mark_processed("https://test.com/page1");
    let cloned = state.clone();
    assert_eq!(cloned.domain, "test.com");
    assert_eq!(cloned.processed_urls.len(), 1);
    assert!(cloned.is_processed("https://test.com/page1"));
}

// ============================================================================
// 2. StateStore creation and path generation
// ============================================================================

#[test]
fn state_store_creation() {
    let store = StateStore::new("example.com");
    let path = store.get_state_path();
    let path_str = path.to_string_lossy();
    assert!(path_str.contains("example.com.json"));
}

#[test]
fn state_store_path_structure() {
    let store = StateStore::new("test.domain");
    let path = store.get_state_path();
    let path_str = path.to_string_lossy();
    assert!(path_str.contains("webfang/state/test.domain.json"));
}

// ============================================================================
// 3. StateStore save/load roundtrip with custom state dir
// ============================================================================

#[test]
fn state_store_save_load_roundtrip() {
    let dir = tempdir().unwrap();
    let mut cache_dir = dir.path().to_path_buf();
    cache_dir.push("webfang/state");

    let mut store = StateStore::new("roundtrip.com");
    store.set_cache_dir(cache_dir);

    // Create and save state
    let mut state = ExportState::new("roundtrip.com");
    state.mark_processed("https://roundtrip.com/p1");
    state.mark_processed("https://roundtrip.com/p2");
    state.mark_processed("https://roundtrip.com/p3");

    store.save(&state).unwrap();

    // Load and verify
    let loaded = store.load().unwrap();
    assert_eq!(loaded.domain, "roundtrip.com");
    assert_eq!(loaded.processed_urls.len(), 3);
    assert!(loaded.is_processed("https://roundtrip.com/p1"));
    assert!(loaded.is_processed("https://roundtrip.com/p2"));
    assert!(loaded.is_processed("https://roundtrip.com/p3"));
}

#[test]
fn state_store_mark_processed_via_store_api() {
    let dir = tempdir().unwrap();
    let mut cache_dir = dir.path().to_path_buf();
    cache_dir.push("webfang/state");

    let mut store = StateStore::new("mark.com");
    store.set_cache_dir(cache_dir);

    let mut state = ExportState::new("mark.com");
    store.mark_processed(&mut state, "https://mark.com/page1");

    assert!(store.is_processed(&state, "https://mark.com/page1"));
    assert!(!store.is_processed(&state, "https://mark.com/page2"));

    // Save and reload to verify persistence
    store.save(&state).unwrap();
    let loaded = store.load().unwrap();
    assert!(store.is_processed(&loaded, "https://mark.com/page1"));
}

#[test]
fn state_store_atomic_save_no_temp_file_remains() {
    let dir = tempdir().unwrap();
    let mut cache_dir = dir.path().to_path_buf();
    cache_dir.push("webfang/state");

    let mut store = StateStore::new("atomic.com");
    store.set_cache_dir(cache_dir);

    let state = ExportState::new("atomic.com");
    store.save(&state).unwrap();

    // Final file exists
    assert!(store.get_state_path().exists());

    // No .tmp file remains
    let mut temp_path = store.get_state_path();
    temp_path.set_extension("tmp");
    assert!(!temp_path.exists());
}

// ============================================================================
// 4. load_or_default behavior
// ============================================================================

#[test]
fn state_store_load_or_default_creates_new_when_missing() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().to_path_buf();

    let mut store = StateStore::new("new.com");
    store.set_cache_dir(cache_dir);

    let state = store.load_or_default().unwrap();
    assert_eq!(state.domain, "new.com");
    assert!(state.processed_urls.is_empty());
}

#[test]
fn state_store_load_or_default_loads_existing() {
    let dir = tempdir().unwrap();
    let mut cache_dir = dir.path().to_path_buf();
    cache_dir.push("webfang/state");
    fs::create_dir_all(&cache_dir).unwrap();

    // Write a state file directly
    let state_path = cache_dir.join("existing.com.json");
    fs::write(
        &state_path,
        r#"{
            "domain": "existing.com",
            "processed_urls": ["https://existing.com/p1"],
            "last_export": null,
            "total_exported": 1
        }"#,
    )
    .unwrap();

    let mut store = StateStore::new("existing.com");
    store.set_cache_dir(cache_dir);

    let state = store.load_or_default().unwrap();
    assert_eq!(state.domain, "existing.com");
    assert_eq!(state.processed_urls.len(), 1);
    assert!(state.is_processed("https://existing.com/p1"));
}

// ============================================================================
// 5. Corrupted state file handling
// ============================================================================

#[test]
fn state_store_load_corrupted_json_returns_serialization_error() {
    let dir = tempdir().unwrap();
    let mut cache_dir = dir.path().to_path_buf();
    cache_dir.push("webfang/state");
    fs::create_dir_all(&cache_dir).unwrap();

    // Write invalid JSON
    let state_path = cache_dir.join("corrupt.com.json");
    fs::write(&state_path, "{ this is not valid json }").unwrap();

    let mut store = StateStore::new("corrupt.com");
    store.set_cache_dir(cache_dir);

    let result = store.load();
    assert!(result.is_err(), "loading corrupted file should fail");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("expected")
            || err_msg.contains("JSON")
            || err_msg.contains("json")
            || err_msg.contains("serializaci")
            || err_msg.contains("key must be a string"),
        "error should mention JSON/serialization issue, got: {err_msg}"
    );
}

#[test]
fn state_store_load_empty_file_returns_error() {
    let dir = tempdir().unwrap();
    let mut cache_dir = dir.path().to_path_buf();
    cache_dir.push("webfang/state");
    fs::create_dir_all(&cache_dir).unwrap();

    let state_path = cache_dir.join("empty.com.json");
    fs::write(&state_path, "").unwrap();

    let mut store = StateStore::new("empty.com");
    store.set_cache_dir(cache_dir);

    let result = store.load();
    assert!(result.is_err(), "loading empty file should fail");
}

#[test]
fn state_store_load_missing_file_returns_io_not_found() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().to_path_buf();

    let mut store = StateStore::new("missing.com");
    store.set_cache_dir(cache_dir);

    let result = store.load();
    assert!(result.is_err(), "loading missing file should fail");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found") || err.to_string().contains("No such file"),
        "error should indicate not found, got: {err}"
    );
}

#[test]
fn state_store_load_corrupted_partial_json_returns_error() {
    let dir = tempdir().unwrap();
    let mut cache_dir = dir.path().to_path_buf();
    cache_dir.push("webfang/state");
    fs::create_dir_all(&cache_dir).unwrap();

    let state_path = cache_dir.join("partial.com.json");
    fs::write(&state_path, r#"{"domain": "partial.com""#).unwrap();

    let mut store = StateStore::new("partial.com");
    store.set_cache_dir(cache_dir);

    let result = store.load();
    assert!(result.is_err(), "loading partial JSON should fail");
}

// ============================================================================
// 6. Integration: save → mark more → load → verify cumulative state
// ============================================================================

#[test]
fn state_store_cumulative_mark_and_save() {
    let dir = tempdir().unwrap();
    let mut cache_dir = dir.path().to_path_buf();
    cache_dir.push("webfang/state");

    let mut store = StateStore::new("cum.com");
    store.set_cache_dir(cache_dir);

    // First batch
    let mut state = ExportState::new("cum.com");
    store.mark_processed(&mut state, "https://cum.com/batch1/a");
    store.mark_processed(&mut state, "https://cum.com/batch1/b");
    store.save(&state).unwrap();

    // Second batch — load existing, mark more, save
    let mut state = store.load().unwrap();
    store.mark_processed(&mut state, "https://cum.com/batch2/c");
    store.mark_processed(&mut state, "https://cum.com/batch2/d");
    store.save(&state).unwrap();

    // Final load — should have all 4 URLs
    let final_state = store.load().unwrap();
    assert_eq!(final_state.processed_urls.len(), 4);
    assert!(final_state.is_processed("https://cum.com/batch1/a"));
    assert!(final_state.is_processed("https://cum.com/batch1/b"));
    assert!(final_state.is_processed("https://cum.com/batch2/c"));
    assert!(final_state.is_processed("https://cum.com/batch2/d"));
}
