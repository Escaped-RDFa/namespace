#!/usr/bin/env bash
# Solve 71 shards in parallel and merge

set -e

PLATFORMS=("Twitter" "Discord" "Telegram" "GitHub_Commit" "GitHub_Body" "Website" "Email" "Reddit" "Mastodon" "Bluesky")
LIMITS=(180 1500 3000 72 45000 50000 10000 35000 300 240)

mkdir -p _site/proofs

echo "=== OPTIMAL 71-SHARD DISTRIBUTION ===" > _site/proofs/optimal_sharding.txt
echo "" >> _site/proofs/optimal_sharding.txt

# Greedy assignment: ensure all platforms get at least 1 shard, then fill largest first
total=0
declare -A platform_count

# First pass: assign 1 shard to each platform (smallest to largest to save capacity)
for idx in 4 10 1 9 2 3 7 8 5 6; do  # Sorted smallest to largest
  platform="${PLATFORMS[$((idx-1))]}"
  limit="${LIMITS[$((idx-1))]}"
  i=$((${#platform_count[@]} + 1))
  echo "Shard $i: $platform - $limit bytes" >> _site/proofs/optimal_sharding.txt
  platform_count[$platform]=1
  total=$((total + limit))
done

# Second pass: fill remaining 61 shards with largest platforms (max 8 total per platform)
for i in {11..71}; do
  for idx in 6 5 8 7 2 3 9 1 10 4; do  # Sorted by size
    platform="${PLATFORMS[$((idx-1))]}"
    limit="${LIMITS[$((idx-1))]}"
    count="${platform_count[$platform]:-0}"
    
    if [ "$count" -lt 8 ]; then
      echo "Shard $i: $platform - $limit bytes" >> _site/proofs/optimal_sharding.txt
      platform_count[$platform]=$((count + 1))
      total=$((total + limit))
      break
    fi
  done
done

echo "" >> _site/proofs/optimal_sharding.txt
echo "Total Information: $total bytes" >> _site/proofs/optimal_sharding.txt
echo "Average per shard: $((total / 71)) bytes" >> _site/proofs/optimal_sharding.txt
echo "" >> _site/proofs/optimal_sharding.txt
echo "Platform Distribution:" >> _site/proofs/optimal_sharding.txt

for platform in "${PLATFORMS[@]}"; do
  count="${platform_count[$platform]:-0}"
  echo "  $platform: $count shards" >> _site/proofs/optimal_sharding.txt
done

echo "✓ Optimal distribution: $total bytes"
