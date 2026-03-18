//! erdfa-relay binary: start the relay server + RMQ consumers
use erdfa_relay::types::*;
use erdfa_relay::server;
use erdfa_relay::rmq;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let instance = Instance {
        domain: std::env::var("ERDFA_DOMAIN").unwrap_or_else(|_| "solana.solfunmeme.com".into()),
        name: "eRDFa Pad".into(),
        summary: "Composable URL-driven encoding platform with federated shard relay".into(),
        pastebin_path: "/erdfa/".into(),
    };

    let state = AppState::new(instance);

    // Register default topics (RMQ queues)
    {
        let mut topics = state.topics.write().unwrap();
        topics.push(Topic {
            name: "git.commits".into(),
            queue: "git.commits".into(),
            routing_key: "git.commit".into(),
            description: "Git commits from 5521 repos".into(),
        });
    }

    // Register default lenses
    {
        let mut lenses = state.lenses.write().unwrap();
        lenses.push(Lens {
            id: "git-latest".into(),
            title: "Latest Git Commits".into(),
            description: "Most recent commits across all repos".into(),
            topic: "git.commits".into(),
            predicate: None,
            object_pattern: None,
            limit: 50,
        });
        lenses.push(Lens {
            id: "all-shards".into(),
            title: "All Shards".into(),
            description: "Every shard in the relay".into(),
            topic: "*".into(),
            predicate: None,
            object_pattern: None,
            limit: 100,
        });
    }

    // Start RMQ consumers
    let rmq_url = std::env::var("RMQ_URL")
        .unwrap_or_else(|_| "amqp://monster:gyroscope@localhost:5672/%2Fmonster".into());
    let rmq_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = rmq::consume_topics(rmq_state, &rmq_url).await {
            tracing::error!("RMQ consumer error: {}", e);
        }
    });

    // Start HTTP server
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3030);
    let app = server::router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    tracing::info!("erdfa-relay listening on :{}", port);
    tracing::info!("  Feeds: http://localhost:{}/feeds", port);
    tracing::info!("  Actor: http://localhost:{}/relay/actor", port);
    tracing::info!("  Status: http://localhost:{}/status", port);
    axum::serve(listener, app).await.unwrap();
}
