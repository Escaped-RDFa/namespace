//! RSS/Atom feed generation from Lens queries
use crate::types::{AppState, Lens};
use rss::{ChannelBuilder, ItemBuilder, GuidBuilder};

/// Generate RSS XML for a lens
pub fn lens_to_rss(state: &AppState, lens: &Lens) -> String {
    let shards = state.query_lens(lens);
    let items: Vec<_> = shards.iter().map(|s| {
        let title = s.triples.iter()
            .find(|t| t.p == "rdf:type" || t.p == "erdfa:name" || t.p.contains("message"))
            .map(|t| format!("{}: {}", t.s.split(':').last().unwrap_or(&t.s), t.o.chars().take(80).collect::<String>()))
            .unwrap_or_else(|| s.cid.clone());

        let link = state.instance.shard_url(&s.cid);

        let desc = s.triples.iter().take(5)
            .map(|t| format!("<{}>  <{}>  \"{}\"", t.s, t.p, t.o.chars().take(100).collect::<String>()))
            .collect::<Vec<_>>()
            .join("\n");

        ItemBuilder::default()
            .title(Some(title))
            .link(Some(link.clone()))
            .guid(Some(GuidBuilder::default()
                .value(s.cid.clone())
                .permalink(false)
                .build()))
            .pub_date(Some(s.ts.to_rfc2822()))
            .description(Some(desc))
            .build()
    }).collect();

    ChannelBuilder::default()
        .title(&lens.title)
        .link(state.instance.feed_url(&lens.id))
        .description(&lens.description)
        .items(items)
        .build()
        .to_string()
}
