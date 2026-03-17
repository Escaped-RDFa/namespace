# Integrating erdfa-publish into shem-hamephorash-72

## Dependency

```toml
# In shem-hamephorash-72/Cargo.toml
[dependencies]
erdfa-publish = { path = "../erdfa-clean/publish" }
```

## Usage

Don't edit main.rs. Create a new file `src/publish.rs`:

```rust
use erdfa_publish::ShardSet;

pub fn export_groups(
    groups: &[(String, Vec<[char; 3]>, /* Mv */ )],
    raw_rows: &[(Vec<char>, Vec<char>, Vec<char>)],
    labels: &[u8],
    english: &[[String; 3]],
) {
    let mut set = ShardSet::new("shem-hamephorash", groups.len());

    for (i, ((loc, trips, _blade), (r1, r2, r3))) in
        groups.iter().zip(raw_rows.iter()).enumerate()
    {
        let subj = format!("_:group{}", i);
        let mut t: Vec<(String, String, String)> = vec![
            (subj.clone(), "rdf:type".into(), "erdfa:VerseGroup".into()),
            (subj.clone(), "erdfa:location".into(), loc.clone()),
            (subj.clone(), "erdfa:row1".into(), r1.iter().collect()),
            (subj.clone(), "erdfa:row2".into(), r2.iter().collect()),
            (subj.clone(), "erdfa:row3".into(), r3.iter().collect()),
        ];
        if i < english.len() && !english[i][0].is_empty() {
            t.push((subj.clone(), "erdfa:english".into(), english[i][0].clone()));
        }
        for (j, &[a, b, c]) in trips.iter().enumerate() {
            let ns = format!("_:group{}:name{}", i, j);
            t.push((ns.clone(), "erdfa:hebrew".into(), format!("{}{}{}", a, b, c)));
            t.push((ns, "erdfa:index".into(), j.to_string()));
        }
        set.push_owned(t);
    }

    std::fs::create_dir_all("shards").ok();
    for s in &set.shards {
        std::fs::write(format!("shards/{}.cbor", s.cid), &s.cbor).ok();
    }
    let m = set.manifest();
    std::fs::write("shards/manifest.cbor", &m.cbor).ok();
    std::fs::write("shards/manifest.json", set.manifest_json()).ok();
    std::fs::write("shards/shards.tar", set.to_tar()).ok();
    std::fs::write("shards/urls.txt",
        set.to_urls("https://solana.solfunmeme.com/erdfa/").join("\n")).ok();
}
```

Then in main.rs, add at the end:
```rust
mod publish;
publish::export_groups(&all_groups, &all_raw_rows, &labels, &all_english);
```

## Output

```
shards/
├── baf*.cbor          # one per verse group
├── manifest.cbor      # lists all shard CIDs
├── manifest.json      # human-readable manifest
├── shards.tar         # everything in one archive
└── urls.txt           # composable eRDFa URLs, one per line
```

Each URL loads into the eRDFa pastebin via `?op=decbor&text=<base64>`.
Upload the tar: `pastebinit < shards/shards.tar`
