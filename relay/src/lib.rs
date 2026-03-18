//! erdfa-relay: ActivityPub federation + RSS feeds for CID-addressed CBOR shards.
//!
//! Architecture:
//!   RabbitMQ queue → Topic → Lens (query) → RSS feed
//!   Pastebin instance = ActivityPub Actor
//!   Shard = ActivityPub Object (Note with CID)
//!   Create activity = new shard, Announce = relay from peer

pub mod types;
pub mod feeds;
pub mod rmq;
pub mod ap;
pub mod server;
