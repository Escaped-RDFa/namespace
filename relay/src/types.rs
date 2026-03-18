//! Core types: Instance, Shard, Lens, Topic
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// CID from raw bytes (wire-compatible with erdfa-publish)
pub fn content_cid(data: &[u8]) -> String {
    let h = Sha256::digest(data);
    format!("baf{}", hex::encode(&h[..16]))
}

/// A CID-addressed CBOR shard (the atomic unit of data)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shard {
    pub cid: String,
    pub cbor: Vec<u8>,           // raw CBOR bytes
    pub triples: Vec<Triple>,    // decoded triples
    pub source: String,          // topic/queue it came from
    pub ts: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Triple {
    pub s: String,
    pub p: String,
    pub o: String,
}

/// A Topic = a RabbitMQ queue = a feed source.
/// Each queue binding becomes a topic that accumulates shards.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,            // e.g. "git.commits", "clarifai.protos"
    pub queue: String,           // RMQ queue name
    pub routing_key: String,     // RMQ routing key pattern
    pub description: String,
}

/// A Lens = a saved query over the shard pool → produces an RSS feed.
/// "top 10 git repos" = Lens { query: match on rdf:type=git:Commit, aggregate by repo }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lens {
    pub id: String,              // URL-safe slug: "top-repos"
    pub title: String,           // "Top 10 Git Repos"
    pub description: String,
    pub topic: String,           // which topic to query
    pub predicate: Option<String>, // filter: match this predicate
    pub object_pattern: Option<String>, // filter: match object contains
    pub limit: usize,            // max items in feed
}

/// An Instance = a pastebin deployment = an ActivityPub Actor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub domain: String,          // e.g. "solana.solfunmeme.com"
    pub name: String,            // e.g. "eRDFa Pad"
    pub summary: String,
    pub pastebin_path: String,   // e.g. "/erdfa/"
}

impl Instance {
    pub fn actor_id(&self) -> String {
        format!("https://{}/relay/actor", self.domain)
    }
    pub fn inbox(&self) -> String {
        format!("https://{}/relay/inbox", self.domain)
    }
    pub fn outbox(&self) -> String {
        format!("https://{}/relay/outbox", self.domain)
    }
    pub fn feed_url(&self, lens_id: &str) -> String {
        format!("https://{}/feed/{}.xml", self.domain, lens_id)
    }
    pub fn shard_url(&self, cid: &str) -> String {
        format!("https://{}{}?op=decbor&cid={}", self.domain, self.pastebin_path, cid)
    }
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub instance: Instance,
    pub shards: Arc<RwLock<HashMap<String, Shard>>>,  // cid → shard
    pub topics: Arc<RwLock<Vec<Topic>>>,
    pub lenses: Arc<RwLock<Vec<Lens>>>,
}

impl AppState {
    pub fn new(instance: Instance) -> Self {
        Self {
            instance,
            shards: Arc::new(RwLock::new(HashMap::new())),
            topics: Arc::new(RwLock::new(Vec::new())),
            lenses: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Ingest a shard, dedup by CID
    pub fn ingest(&self, shard: Shard) -> bool {
        let mut store = self.shards.write().unwrap();
        if store.contains_key(&shard.cid) {
            return false; // already have it
        }
        store.insert(shard.cid.clone(), shard);
        true
    }

    /// Query shards matching a lens
    pub fn query_lens(&self, lens: &Lens) -> Vec<&Shard> {
        let store = self.shards.read().unwrap();
        let mut results: Vec<_> = store.values()
            .filter(|s| s.source == lens.topic || lens.topic == "*")
            .filter(|s| {
                if let Some(ref pred) = lens.predicate {
                    s.triples.iter().any(|t| t.p.contains(pred.as_str()))
                } else { true }
            })
            .filter(|s| {
                if let Some(ref pat) = lens.object_pattern {
                    s.triples.iter().any(|t| t.o.contains(pat.as_str()))
                } else { true }
            })
            .collect();
        results.sort_by(|a, b| b.ts.cmp(&a.ts));
        results.truncate(lens.limit);
        // Safety: we hold the read lock for the duration
        unsafe { std::mem::transmute(results) }
    }
}

// hex encoding (tiny, no extra dep)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
