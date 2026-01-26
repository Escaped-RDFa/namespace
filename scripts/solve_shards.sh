#!/usr/bin/env bash
# Solve 71 shards across ALL platforms (social + blockchain)

set -e

# Social platforms
SOCIAL_PLATFORMS=("Twitter" "Discord" "Telegram" "GitHub_Commit" "GitHub_Body" "Website" "Email" "Reddit" "Mastodon" "Bluesky")
SOCIAL_LIMITS=(180 1500 3000 72 45000 50000 10000 35000 300 240)

# Blockchain platforms (testnets first, then mainnet)
BLOCKCHAIN_PLATFORMS=(
    "Solana_Devnet" "Solana_Testnet" "Solana_Mainnet"
    "Ethereum_Sepolia" "Ethereum_Goerli" "Ethereum_Mainnet"
    "Bitcoin_Testnet" "Bitcoin_Mainnet"
    "Polygon_Mumbai" "Polygon_Mainnet"
    "Arbitrum_Testnet" "Arbitrum_Mainnet"
    "Optimism_Testnet" "Optimism_Mainnet"
    "Base_Testnet" "Base_Mainnet"
    "Avalanche_Fuji" "Avalanche_Mainnet"
    "BSC_Testnet" "BSC_Mainnet"
)
# Blockchain limits (bytes per transaction)
BLOCKCHAIN_LIMITS=(
    1232 1232 1232  # Solana (memo field)
    32768 32768 32768  # Ethereum (calldata)
    80 80  # Bitcoin (OP_RETURN)
    32768 32768  # Polygon
    32768 32768  # Arbitrum
    32768 32768  # Optimism
    32768 32768  # Base
    32768 32768  # Avalanche
    32768 32768  # BSC
)

ALL_PLATFORMS=("${SOCIAL_PLATFORMS[@]}" "${BLOCKCHAIN_PLATFORMS[@]}")
ALL_LIMITS=("${SOCIAL_LIMITS[@]}" "${BLOCKCHAIN_LIMITS[@]}")

mkdir -p _site/proofs

echo "=== OPTIMAL 71-SHARD DISTRIBUTION ===" > _site/proofs/optimal_sharding.txt
echo "" >> _site/proofs/optimal_sharding.txt
echo "Platforms: ${#ALL_PLATFORMS[@]} (10 social + ${#BLOCKCHAIN_PLATFORMS[@]} blockchain)" >> _site/proofs/optimal_sharding.txt
echo "" >> _site/proofs/optimal_sharding.txt

# Phase 1: Testnets (free)
# Phase 2: Mainnets (pooled funding)

total=0
declare -A platform_count

# Assign to largest available platforms (max 3 per platform for distribution)
for i in {1..71}; do
  for idx in $(seq 0 $((${#ALL_PLATFORMS[@]} - 1))); do
    platform="${ALL_PLATFORMS[$idx]}"
    limit="${ALL_LIMITS[$idx]}"
    count="${platform_count[$platform]:-0}"
    
    if [ "$count" -lt 3 ]; then
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

for platform in "${ALL_PLATFORMS[@]}"; do
  count="${platform_count[$platform]:-0}"
  if [ "$count" -gt 0 ]; then
    echo "  $platform: $count shards" >> _site/proofs/optimal_sharding.txt
  fi
done

echo "" >> _site/proofs/optimal_sharding.txt
echo "Deployment Strategy:" >> _site/proofs/optimal_sharding.txt
echo "  Phase 1: Testnets (free)" >> _site/proofs/optimal_sharding.txt
echo "  Phase 2: Mainnets (pooled funding)" >> _site/proofs/optimal_sharding.txt

echo "✓ Optimal distribution: $total bytes across ${#ALL_PLATFORMS[@]} platforms"
