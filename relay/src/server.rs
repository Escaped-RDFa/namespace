//! Axum HTTP server wiring all endpoints together
use crate::types::{AppState, Instance, Lens, Topic};
use crate::feeds::lens_to_rss;
use crate::ap;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};

/// Build the router
pub fn router(state: AppState) -> Router {
    Router::new()
        // RSS feeds
        .route("/feed/{lens_id}.xml", get(feed_handler))
        .route("/feeds", get(list_feeds))
        // ActivityPub
        .route("/relay/actor", get(actor_handler))
        .route("/relay/inbox", post(inbox_handler))
        .route("/relay/outbox", get(outbox_handler))
        .route("/.well-known/webfinger", get(webfinger_handler))
        // Status
        .route("/status", get(status_handler))
        .with_state(state)
}

async fn feed_handler(State(state): State<AppState>, Path(lens_id): Path<String>) -> impl IntoResponse {
    let lenses = state.lenses.read().unwrap();
    if let Some(lens) = lenses.iter().find(|l| l.id == lens_id) {
        let xml = lens_to_rss(&state, lens);
        (
            [("content-type", "application/rss+xml; charset=utf-8")],
            xml,
        ).into_response()
    } else {
        (axum::http::StatusCode::NOT_FOUND, "lens not found").into_response()
    }
}

async fn list_feeds(State(state): State<AppState>) -> Json<Vec<serde_json::Value>> {
    let lenses = state.lenses.read().unwrap();
    Json(lenses.iter().map(|l| serde_json::json!({
        "id": l.id,
        "title": l.title,
        "url": state.instance.feed_url(&l.id),
        "topic": l.topic,
        "limit": l.limit,
    })).collect())
}

async fn actor_handler(State(state): State<AppState>) -> impl IntoResponse {
    ap::actor(state).await
}

async fn inbox_handler(State(state): State<AppState>, body: Json<serde_json::Value>) -> &'static str {
    ap::inbox(state, body).await
}

async fn outbox_handler(State(state): State<AppState>) -> impl IntoResponse {
    ap::outbox(state).await
}

async fn webfinger_handler(State(state): State<AppState>) -> impl IntoResponse {
    ap::webfinger(state).await
}

async fn status_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let shards = state.shards.read().unwrap().len();
    let topics = state.topics.read().unwrap().len();
    let lenses = state.lenses.read().unwrap().len();
    Json(serde_json::json!({
        "instance": state.instance.domain,
        "shards": shards,
        "topics": topics,
        "lenses": lenses,
        "actor": state.instance.actor_id(),
    }))
}
