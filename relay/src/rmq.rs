//! RabbitMQ consumer: each queue → Topic → shards
use crate::types::{AppState, Shard, Triple, Topic};
use chrono::Utc;
use futures_lite::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use tracing::info;

/// Connect to RMQ and consume from all registered topics
pub async fn consume_topics(state: AppState, rmq_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect(rmq_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    let topics = state.topics.read().unwrap().clone();
    for topic in topics {
        let ch = conn.create_channel().await?;
        let state = state.clone();
        let topic = topic.clone();

        // Declare queue (idempotent)
        ch.queue_declare(&topic.queue, QueueDeclareOptions::default(), FieldTable::default()).await?;

        let consumer = ch.basic_consume(
            &topic.queue,
            &format!("erdfa-relay-{}", topic.name),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        ).await?;

        tokio::spawn(async move {
            info!("Consuming topic: {} (queue: {})", topic.name, topic.queue);
            let mut consumer = consumer;
            while let Some(Ok(delivery)) = consumer.next().await {
                if let Some(shard) = delivery_to_shard(&delivery.data, &topic) {
                    let is_new = state.ingest(shard);
                    if is_new {
                        info!("New shard from {}: {}", topic.name, 
                            std::str::from_utf8(&delivery.data).unwrap_or("(binary)").chars().take(80).collect::<String>());
                    }
                }
                let _ = delivery.ack(BasicAckOptions::default()).await;
            }
        });
    }
    Ok(())
}

/// Parse a RMQ message into a Shard
fn delivery_to_shard(data: &[u8], topic: &Topic) -> Option<Shard> {
    // Try JSON (git.commits format)
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(data) {
        let triples = json_to_triples(&json, &topic.name);
        let cid = crate::types::content_cid(data);
        return Some(Shard {
            cid,
            cbor: data.to_vec(), // store raw for relay
            triples,
            source: topic.name.clone(),
            ts: Utc::now(),
        });
    }
    // Try raw CBOR
    let cid = crate::types::content_cid(data);
    Some(Shard {
        cid,
        cbor: data.to_vec(),
        triples: vec![],
        source: topic.name.clone(),
        ts: Utc::now(),
    })
}

/// Flatten JSON object into triples: (topic:key, field, value)
fn json_to_triples(val: &serde_json::Value, topic: &str) -> Vec<Triple> {
    let mut triples = Vec::new();
    if let Some(obj) = val.as_object() {
        let subj = obj.get("hash")
            .or_else(|| obj.get("id"))
            .and_then(|v| v.as_str())
            .map(|s| format!("{}:{}", topic, s))
            .unwrap_or_else(|| format!("{}:anon", topic));

        for (k, v) in obj {
            let o = match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            triples.push(Triple { s: subj.clone(), p: k.clone(), o });
        }
    }
    triples
}
