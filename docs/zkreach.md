# ZKReach - Reward System

## Concept

Reward accounts for expanding eRDFa distribution by copying shards to new platforms with ZK proofs.

## How It Works

1. **Copy Shard** - Take shard from any platform (testnet/mainnet)
2. **Deploy to New Platform** - Post to a platform not yet reached
3. **Submit ZK Proof** - Prove `source_hash == target_hash`
4. **Earn Reward** - Get tokens for expanding reach

## Rewards

- **100 tokens** per new platform reached
- **Reach Score** = number of new platforms
- **Leaderboard** tracks top contributors

## Example

```rust
ZKReachProof {
    shard_id: 1,
    source_platform: "Solana_Testnet",
    target_platform: "Ethereum_Mainnet",
    source_hash: "0xabc",
    target_hash: "0xabc",  // ZK proof of equivalence
    reacher: "alice",
}

// Alice earns 100 tokens for reaching Ethereum_Mainnet
```

## Incentive Structure

- **First to reach** a new platform gets the reward
- **Duplicate copies** to same platform don't earn rewards
- **71 shards × N platforms** = maximum reach potential
- **Organic growth** through community incentives

## ZK Proof

Proves content equivalence without revealing:
- Source location
- Target location  
- Content itself

Only proves: `hash(source) == hash(target)`

## Leaderboard

```
Rank  Reacher    Rewards    Reach
1     alice      10,000     100 platforms
2     bob        5,000      50 platforms
3     carol      2,000      20 platforms
```

## Integration

ZKReach proofs written to existing channels (block 2, social posts, etc.), expanding reach organically while rewarding contributors.
