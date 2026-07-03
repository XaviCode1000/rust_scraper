//! Checkpoint persistence for the crawler engine
//!
//! Provides save/load of crawl state (visited URLs, queued URLs, pages crawled)
//! using bincode serialization with CRC32 integrity checks.
//!
//! # Rules Applied
//!
//! - **err-thiserror-lib**: Uses thiserror for error types
//! - **own-borrow-over-clone**: Accepts references for read operations
//! - **mem-with-capacity**: Pre-allocates collections when size is known

use std::collections::HashSet;
use std::path::Path;

use crc32fast::Hasher;
use thiserror::Error;

/// Errors that can occur during checkpoint operations
#[derive(Debug, Error)]
pub enum CheckpointError {
    /// I/O error during file operations
    #[error("checkpoint I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("checkpoint serialization error: {0}")]
    Serialization(String),

    /// CRC32 integrity check failed
    #[error("checkpoint integrity check failed: expected {expected:#010x}, got {actual:#010x}")]
    IntegrityMismatch {
        /// Expected CRC32 value
        expected: u32,
        /// Actual CRC32 value computed
        actual: u32,
    },
}

/// Crawl checkpoint state for persistence
///
/// Stores the crawl progress so it can be resumed after interruption.
/// Serialized with bincode and wrapped with CRC32 for integrity.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CrawlCheckpoint {
    /// URLs that have been visited
    pub visited: HashSet<String>,
    /// URLs that were queued but not yet visited
    pub queued: Vec<String>,
    /// Total pages successfully crawled
    pub pages_crawled: u64,
}

impl CrawlCheckpoint {
    /// Create a new empty checkpoint
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a checkpoint with pre-existing state
    pub fn with_state(visited: HashSet<String>, queued: Vec<String>, pages_crawled: u64) -> Self {
        Self {
            visited,
            queued,
            pages_crawled,
        }
    }

    /// Serialize the checkpoint to bytes with CRC32 integrity wrapper
    ///
    /// Format: [4 bytes CRC32][bincode bytes]
    ///
    /// # Errors
    ///
    /// Returns `CheckpointError::Serialization` if bincode encoding fails
    pub fn save_to_bytes(&self) -> Result<Vec<u8>, CheckpointError> {
        let data =
            bincode::serialize(self).map_err(|e| CheckpointError::Serialization(e.to_string()))?;

        let mut hasher = Hasher::new();
        hasher.update(&data);
        let crc = hasher.finalize();

        let mut output = Vec::with_capacity(4 + data.len());
        output.extend_from_slice(&crc.to_le_bytes());
        output.extend_from_slice(&data);

        Ok(output)
    }

    /// Deserialize a checkpoint from bytes, verifying CRC32 integrity
    ///
    /// # Errors
    ///
    /// Returns `CheckpointError::IntegrityMismatch` if CRC32 check fails,
    /// or `CheckpointError::Serialization` if bincode decoding fails
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() < 4 {
            return Err(CheckpointError::Serialization(
                "checkpoint data too short".to_string(),
            ));
        }

        let expected_crc = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let data = &bytes[4..];

        let mut hasher = Hasher::new();
        hasher.update(data);
        let actual_crc = hasher.finalize();

        if expected_crc != actual_crc {
            return Err(CheckpointError::IntegrityMismatch {
                expected: expected_crc,
                actual: actual_crc,
            });
        }

        bincode::deserialize(data).map_err(|e| CheckpointError::Serialization(e.to_string()))
    }

    /// Save checkpoint to a file
    ///
    /// # Errors
    ///
    /// Returns `CheckpointError` on I/O or serialization failure
    pub fn save_to_file(&self, path: &Path) -> Result<(), CheckpointError> {
        let bytes = self.save_to_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Load checkpoint from a file
    ///
    /// # Errors
    ///
    /// Returns `CheckpointError` on I/O, serialization, or integrity failure
    pub fn load_from_file(path: &Path) -> Result<Self, CheckpointError> {
        let bytes = std::fs::read(path)?;
        Self::load_from_bytes(&bytes)
    }

    /// Check if a URL has already been visited
    #[must_use]
    pub fn is_visited(&self, url: &str) -> bool {
        self.visited.contains(url)
    }

    /// Mark a URL as visited
    pub fn mark_visited(&mut self, url: &str) {
        self.visited.insert(url.to_string());
    }

    /// Get the number of visited URLs
    #[must_use]
    pub fn visited_count(&self) -> usize {
        self.visited.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_roundtrip() {
        let mut checkpoint = CrawlCheckpoint::new();
        checkpoint.mark_visited("https://example.com");
        checkpoint.mark_visited("https://example.com/page1");
        checkpoint
            .queued
            .push("https://example.com/page2".to_string());
        checkpoint.pages_crawled = 2;

        let bytes = checkpoint.save_to_bytes().unwrap();
        let loaded = CrawlCheckpoint::load_from_bytes(&bytes).unwrap();

        assert_eq!(checkpoint.visited, loaded.visited);
        assert_eq!(checkpoint.queued, loaded.queued);
        assert_eq!(checkpoint.pages_crawled, loaded.pages_crawled);
    }

    #[test]
    fn test_checkpoint_integrity_failure() {
        let checkpoint = CrawlCheckpoint::new();
        let mut bytes = checkpoint.save_to_bytes().unwrap();

        // Corrupt the data
        bytes[8] ^= 0xFF;

        let result = CrawlCheckpoint::load_from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("checkpoint.bin");

        let mut checkpoint = CrawlCheckpoint::new();
        checkpoint.mark_visited("https://example.com");
        checkpoint.pages_crawled = 1;

        checkpoint.save_to_file(&path).unwrap();
        let loaded = CrawlCheckpoint::load_from_file(&path).unwrap();

        assert_eq!(checkpoint.visited, loaded.visited);
        assert_eq!(checkpoint.pages_crawled, loaded.pages_crawled);
    }
}
