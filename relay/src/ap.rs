//! ActivityPub actor endpoints (stub — wire up with activitypub-federation)
use crate::types::AppState;
use axum::Json;
use serde_json::{json, Value};

/// WebFinger response for /.well-known/webfinger
pub async fn webfinger(state: AppState) -> Json<Value> {
    let actor_id = state.instance.actor_id();
    Json(json!({
        "subject": format!("acct:relay@{}", state.instance.domain),
        "links": [{
            "rel": "self",
            "type": "application/activity+json",
            "href": actor_id
        }]
    }))
}

/// Actor profile
pub async fn actor(state: AppState) -> Json<Value> {
    let inst = &state.instance;
    Json(json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "Application",
        "id": inst.actor_id(),
        "name": inst.name,
        "summary": inst.summary,
        "inbox": inst.inbox(),
        "outbox": inst.outbox(),
        "preferredUsername": "relay",
        "url": format!("https://{}{}", inst.domain, inst.pastebin_path),
    }))
}

/// Outbox: list of recent shard Create activities
pub async fn outbox(state: AppState) -> Json<Value> {
    let shards = state.shards.read().unwrap();
    let items: Vec<Value> = shards.values()
        .take(50)
        .map(|s| json!({
            "type": "Create",
            "actor": state.instance.actor_id(),
            "object": {
                "type": "Note",
                "id": state.instance.shard_url(&s.cid),
                "content": format!("{} triples from {}", s.triples.len(), s.source),
                "published": s.ts.to_rfc3339(),
                "tag": [{ "type": "Hashtag", "name": format!("#{}", s.source) }],
                "attachment": [{
                    "type": "Document",
                    "mediaType": "application/cbor",
                    "name": s.cid.clone(),
                }]
            }
        }))
        .collect();

    Json(json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "OrderedCollection",
        "totalItems": items.len(),
        "orderedItems": items
    }))
}

/// Inbox: receive activities from peers (Create = new shard, Announce = relay)
pub async fn inbox(state: AppState, Json(activity): Json<Value>) -> &'static str {
    let activity_type = activity.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match activity_type {
        "Create" | "Announce" => {
            // Extract shard CID from the object
            if let Some(obj) = activity.get("object") {
                let cid = obj.get("name")
                    .or_else(|| obj.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                tracing::info!("Received {} for shard: {}", activity_type, cid);
                // TODO: fetch CBOR from peer and ingest
            }
            "accepted"
        }
        _ => "ignored"
    }
}
