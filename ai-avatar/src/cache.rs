//! In-memory LRU cache for LLM responses.
//!
//! Keyed by a hash of (prompt + context) to avoid repeated inference
//! for identical or near-identical queries.

use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use tracing::{debug, info};

pub struct ResponseCache {
    inner: Mutex<LruCache<u64, String>>,
    hits: std::sync::atomic::AtomicU64,
    misses: std::sync::atomic::AtomicU64,
}

impl ResponseCache {
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity.max(1)).unwrap();
        Self {
            inner: Mutex::new(LruCache::new(cap)),
            hits: std::sync::atomic::AtomicU64::new(0),
            misses: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Compute a cache key from prompt text.
    pub fn key(prompt: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get(&self, prompt: &str) -> Option<String> {
        let key = Self::key(prompt);
        let mut cache = self.inner.lock();
        if let Some(val) = cache.get(&key) {
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            debug!("Cache hit for key {}", key);
            Some(val.clone())
        } else {
            self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }

    pub fn put(&self, prompt: &str, response: String) {
        let key = Self::key(prompt);
        self.inner.lock().put(key, response);
    }

    pub fn stats(&self) -> (u64, u64) {
        (
            self.hits.load(std::sync::atomic::Ordering::Relaxed),
            self.misses.load(std::sync::atomic::Ordering::Relaxed),
        )
    }

    pub fn clear(&self) {
        self.inner.lock().clear();
        info!("Response cache cleared");
    }
}
