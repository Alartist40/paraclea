//! Off-Grid Reticulum Mesh Theological Bulletin Wire (`paraclea/bulletin/v1`).
//!
//! Provides asynchronous, signed store-and-forward dissemination of devotional
//! reading guides, verse notes, and prayer requests across off-grid LoRa mesh clusters.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BulletinCategory {
    Devotional,
    ScriptureNote,
    PrayerRequest,
    EmergencyNotice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletinPacket {
    pub id: String,
    pub author_node: String,
    pub timestamp: DateTime<Utc>,
    pub category: BulletinCategory,
    pub title: String,
    pub content: String,
    pub hop_limit: u8,
    pub signature_hash: String,
}

impl BulletinPacket {
    pub fn new(
        author_node: &str,
        category: BulletinCategory,
        title: &str,
        content: &str,
        hop_limit: u8,
    ) -> Self {
        let now = Utc::now();
        let payload = format!("{}:{}:{}:{}", author_node, now.to_rfc3339(), title, content);
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let hash_hex = format!("{:x}", hasher.finalize());
        let id = hash_hex[..16].to_string();

        Self {
            id,
            author_node: author_node.to_string(),
            timestamp: now,
            category,
            title: title.to_string(),
            content: content.to_string(),
            hop_limit,
            signature_hash: hash_hex,
        }
    }
}

pub struct BulletinRouter {
    seen_ids: HashSet<String>,
    local_store: Vec<BulletinPacket>,
    max_store: usize,
}

impl Default for BulletinRouter {
    fn default() -> Self {
        Self::new(200)
    }
}

impl BulletinRouter {
    pub fn new(max_store: usize) -> Self {
        Self {
            seen_ids: HashSet::new(),
            local_store: Vec::new(),
            max_store,
        }
    }

    /// Ingest and forward a received mesh packet. Returns Some(forward_packet) if viable.
    pub fn receive_packet(&mut self, packet: BulletinPacket) -> Result<Option<BulletinPacket>> {
        if self.seen_ids.contains(&packet.id) {
            return Ok(None); // Deduplicated duplicate packet
        }

        self.seen_ids.insert(packet.id.clone());
        self.local_store.push(packet.clone());

        if self.local_store.len() > self.max_store {
            self.local_store.remove(0);
        }

        if packet.hop_limit <= 1 {
            return Ok(None); // TTL/Hop exhausted
        }

        let mut forwarded = packet;
        forwarded.hop_limit -= 1;
        Ok(Some(forwarded))
    }

    /// Get all bulletins in local memory.
    pub fn list_bulletins(&self) -> &[BulletinPacket] {
        &self.local_store
    }

    /// Search local store by keyword.
    pub fn search(&self, keyword: &str) -> Vec<&BulletinPacket> {
        let kw = keyword.to_lowercase();
        self.local_store
            .iter()
            .filter(|b| b.title.to_lowercase().contains(&kw) || b.content.to_lowercase().contains(&kw))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulletin_deduplication_and_forwarding() -> Result<()> {
        let mut router = BulletinRouter::new(50);
        let packet = BulletinPacket::new(
            "node_alpha",
            BulletinCategory::Devotional,
            "Morning Sabbath Study",
            "Psalm 23: The Lord is my shepherd.",
            3,
        );

        let forwarded = router.receive_packet(packet.clone())?;
        assert!(forwarded.is_some());
        let f = forwarded.unwrap();
        assert_eq!(f.hop_limit, 2);

        // Immediate duplicate receive should yield None
        let duplicate = router.receive_packet(packet)?;
        assert!(duplicate.is_none());

        assert_eq!(router.list_bulletins().len(), 1);
        let hits = router.search("shepherd");
        assert_eq!(hits.len(), 1);
        Ok(())
    }
}
