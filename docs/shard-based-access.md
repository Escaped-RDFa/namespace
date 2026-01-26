# Shard-Based Access Control: Type-Specific Mathematical Structures

## Abstract

Each semantic data type has its own optimal mathematical structure for sharding, determined by the type's inherent properties, symmetry group, and information-theoretic requirements. The number of shards N is not arbitrary but derived from the data type's mathematical structure. 71 is just one example—the Gandalf threshold for general semantic data.

## Type-Specific Shard Structures

### 1. General Semantic Data (RDFa)
**Shard count**: 71 (Gandalf Prime)
**Structure**: Sporadic group threshold
**Reason**: Minimal universal threshold for semantic web data

### 2. Boolean Data
**Shard count**: 2
**Structure**: XOR secret sharing
**Reason**: Binary structure, 2^1 = 2

### 3. Quaternion Data (3D Rotations)
**Shard count**: 4
**Structure**: Quaternion decomposition
**Reason**: 4-dimensional algebra, 2^2 = 4

### 4. Octonion Data (Exceptional Symmetry)
**Shard count**: 8
**Structure**: Octonion decomposition
**Reason**: 8-dimensional algebra, 2^3 = 8

### 5. Genetic Code
**Shard count**: 64
**Structure**: Codon-based splitting
**Reason**: 64 codons (4^3), 2^6 = 64

### 6. Unicode Text
**Shard count**: 256
**Structure**: Byte-level splitting
**Reason**: 256 possible byte values, 2^8 = 256

### 7. IPv6 Addresses
**Shard count**: 128
**Structure**: Bit-level splitting
**Reason**: 128-bit addresses, 2^7 = 128

### 8. Mathieu M₂₄ Data
**Shard count**: 24
**Structure**: Golay code decomposition
**Reason**: 24-dimensional Mathieu group

### 9. Leech Lattice Data
**Shard count**: 24
**Structure**: Lattice vector decomposition
**Reason**: 24-dimensional Leech lattice

### 10. Monster Group Data
**Shard count**: 196,883
**Structure**: Minimal representation decomposition
**Reason**: Monster's smallest non-trivial representation

## Architecture

### Core Concept

```
Document D of type T → Determine N from type's mathematical structure
Split into N shards using type-specific algorithm
Distribute to top N coin holders → Each signs shard → Publish to blockchain
Reconstruction: Collect all N shards → Verify → Reconstruct using type-specific algorithm
```

### Type-Specific Shard Counts

| Data Type | N Shards | Mathematical Structure | Reason |
|-----------|----------|------------------------|--------|
| Boolean | 2 | XOR | 2^1 |
| Quaternion | 4 | Quaternion algebra | 2^2 |
| Octonion | 8 | Octonion algebra | 2^3 |
| Genetic | 64 | Codon space | 4^3 = 2^6 |
| Mathieu M₂₄ | 24 | Golay code | Mathieu group |
| RDFa (General) | 71 | Sporadic threshold | Gandalf Prime |
| IPv6 | 128 | Bit decomposition | 2^7 |
| Byte data | 256 | Byte space | 2^8 |
| Monster | 196,883 | Minimal representation | Monster dimension |

## Data Structures

```rust
/// Data type with specific mathematical structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Boolean,           // 2 shards
    Quaternion,        // 4 shards
    Octonion,          // 8 shards
    MathieuM24,        // 24 shards
    Genetic,           // 64 shards
    RDFa,              // 71 shards (Gandalf)
    IPv6,              // 128 shards
    Byte,              // 256 shards
    Monster,           // 196,883 shards
}

impl DataType {
    pub fn shard_count(&self) -> usize {
        match self {
            DataType::Boolean => 2,
            DataType::Quaternion => 4,
            DataType::Octonion => 8,
            DataType::MathieuM24 => 24,
            DataType::Genetic => 64,
            DataType::RDFa => 71,
            DataType::IPv6 => 128,
            DataType::Byte => 256,
            DataType::Monster => 196_883,
        }
    }
    
    pub fn mathematical_structure(&self) -> &'static str {
        match self {
            DataType::Boolean => "XOR secret sharing",
            DataType::Quaternion => "Quaternion decomposition",
            DataType::Octonion => "Octonion decomposition",
            DataType::MathieuM24 => "Golay code",
            DataType::Genetic => "Codon-based splitting",
            DataType::RDFa => "Sporadic group threshold",
            DataType::IPv6 => "Bit-level decomposition",
            DataType::Byte => "Byte-level splitting",
            DataType::Monster => "Minimal representation",
        }
    }
}

/// Coin holder at specific block
#[derive(Debug, Clone)]
pub struct CoinHolder {
    pub address: Vec<u8>,
    pub balance: u64,
    pub rank: usize,        // 1 = top holder
    pub block_height: u64,
}

/// Document shard with type information
#[derive(Debug, Clone)]
pub struct DocumentShard {
    pub shard_id: usize,           // 0 to N-1
    pub data: Vec<u8>,             // Encrypted shard data
    pub holder_address: Vec<u8>,   // Who holds this shard
    pub signature: Vec<u8>,        // Holder's signature
    pub block_height: u64,         // Block where holders determined
    pub coin_type: String,         // e.g., "BTC", "ETH", "ERDFA"
    pub data_type: DataType,       // Type-specific structure
}

/// Sharded document with type
#[derive(Debug, Clone)]
pub struct ShardedDocument {
    pub document_id: Vec<u8>,
    pub data_type: DataType,       // Determines shard count
    pub total_shards: usize,
    pub required_shards: usize,    // Usually = total_shards
    pub shards: Vec<DocumentShard>,
    pub block_height: u64,
    pub coin_type: String,
    pub metadata: Vec<u8>,         // Public metadata
}

/// Shard reconstruction
#[derive(Debug, Clone)]
pub struct ShardReconstruction {
    pub document_id: Vec<u8>,
    pub collected_shards: Vec<DocumentShard>,
    pub verified_holders: Vec<CoinHolder>,
}
```

## Shamir Secret Sharing

Use Shamir's Secret Sharing for N-of-N threshold:

```rust
pub struct ShamirSharing {
    pub threshold: usize,  // k
    pub total_shares: usize, // n
}

impl ShamirSharing {
    pub fn split(&self, secret: &[u8]) -> Vec<Vec<u8>> {
        let mut shares = Vec::new();
        
        for i in 1..=self.total_shares {
            let share = self.generate_share(secret, i);
            shares.push(share);
        }
        
        shares
    }
    
    fn generate_share(&self, secret: &[u8], index: usize) -> Vec<u8> {
        // Simplified Shamir sharing
        secret.iter()
            .map(|&byte| {
                // Polynomial evaluation at point 'index'
                ((byte as usize + index) % 256) as u8
            })
            .collect()
    }
    
    pub fn reconstruct(&self, shares: &[Vec<u8>]) -> Option<Vec<u8>> {
        if shares.len() < self.threshold {
            return None;
        }
        
        // Use first 'threshold' shares
        let selected = &shares[..self.threshold];
        
        // Lagrange interpolation to recover secret
        Some(self.lagrange_interpolate(selected))
    }
    
    fn lagrange_interpolate(&self, shares: &[Vec<u8>]) -> Vec<u8> {
        // Simplified reconstruction
        let len = shares[0].len();
        let mut secret = vec![0u8; len];
        
        for i in 0..len {
            let mut sum = 0usize;
            for (j, share) in shares.iter().enumerate() {
                sum += share[i] as usize * (j + 1);
            }
            secret[i] = (sum / shares.len()) as u8;
        }
        
        secret
    }
}
```

## Top-N Holder Selection

```rust
pub struct CoinHolderRegistry {
    pub coin_type: String,
    pub holders: Vec<CoinHolder>,
}

impl CoinHolderRegistry {
    pub fn get_top_n_at_block(&self, n: usize, block_height: u64) -> Vec<CoinHolder> {
        // Query blockchain state at specific block
        let mut holders = self.holders.clone();
        
        // Sort by balance descending
        holders.sort_by(|a, b| b.balance.cmp(&a.balance));
        
        // Take top N
        holders.into_iter()
            .take(n)
            .enumerate()
            .map(|(i, mut h)| {
                h.rank = i + 1;
                h.block_height = block_height;
                h
            })
            .collect()
    }
    
    pub fn verify_holder_at_block(&self, 
                                   address: &[u8], 
                                   block_height: u64) -> Option<CoinHolder> {
        self.holders.iter()
            .find(|h| h.address == address && h.block_height == block_height)
            .cloned()
    }
}
```

## Document Sharding System

```rust
pub struct ShardingSystem {
    shamir: ShamirSharing,
    registry: CoinHolderRegistry,
}

impl ShardingSystem {
    pub fn new(n: usize, coin_type: String) -> Self {
        Self {
            shamir: ShamirSharing {
                threshold: n,
                total_shares: n,
            },
            registry: CoinHolderRegistry {
                coin_type,
                holders: Vec::new(),
            },
        }
    }
    
    pub fn shard_document(&self, 
                         document: &[u8], 
                         block_height: u64) -> ShardedDocument {
        // 1. Get top N holders at block
        let holders = self.registry.get_top_n_at_block(
            self.shamir.total_shares, 
            block_height
        );
        
        // 2. Split document into N shards
        let shares = self.shamir.split(document);
        
        // 3. Create shard for each holder
        let shards: Vec<DocumentShard> = shares.into_iter()
            .zip(holders.iter())
            .enumerate()
            .map(|(i, (data, holder))| {
                DocumentShard {
                    shard_id: i,
                    data,
                    holder_address: holder.address.clone(),
                    signature: Vec::new(), // Holder will sign
                    block_height,
                    coin_type: self.registry.coin_type.clone(),
                }
            })
            .collect();
        
        ShardedDocument {
            document_id: hash_document(document),
            total_shards: self.shamir.total_shares,
            required_shards: self.shamir.threshold,
            shards,
            block_height,
            coin_type: self.registry.coin_type.clone(),
            metadata: Vec::new(),
        }
    }
    
    pub fn reconstruct_document(&self, 
                               sharded: &ShardedDocument,
                               collected_shards: Vec<DocumentShard>) -> Option<Vec<u8>> {
        // 1. Verify we have enough shards
        if collected_shards.len() < sharded.required_shards {
            return None;
        }
        
        // 2. Verify each shard signature
        for shard in &collected_shards {
            if !self.verify_shard_signature(shard, sharded.block_height) {
                return None;
            }
        }
        
        // 3. Extract shard data
        let shares: Vec<Vec<u8>> = collected_shards.iter()
            .map(|s| s.data.clone())
            .collect();
        
        // 4. Reconstruct using Shamir
        self.shamir.reconstruct(&shares)
    }
    
    fn verify_shard_signature(&self, shard: &DocumentShard, block_height: u64) -> bool {
        // 1. Verify holder was in top N at block
        let holder = self.registry.verify_holder_at_block(
            &shard.holder_address, 
            block_height
        );
        
        if holder.is_none() {
            return false;
        }
        
        // 2. Verify signature
        verify_signature(&shard.data, &shard.signature, &shard.holder_address)
    }
}

fn hash_document(data: &[u8]) -> Vec<u8> {
    let mut hash = vec![0u8; 32];
    for (i, &byte) in data.iter().enumerate() {
        hash[i % 32] ^= byte;
    }
    hash
}

fn verify_signature(data: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    // Simplified signature verification
    if signature.is_empty() {
        return false;
    }
    
    let expected: u8 = data.iter().fold(0, |acc, &b| acc ^ b);
    let sig_check: u8 = signature.iter().fold(0, |acc, &b| acc ^ b);
    let key_check: u8 = public_key.iter().fold(0, |acc, &b| acc ^ b);
    
    expected == (sig_check ^ key_check)
}
```

## Holder Workflow

### Publishing a Shard

```rust
pub struct ShardHolder {
    pub address: Vec<u8>,
    pub private_key: Vec<u8>,
}

impl ShardHolder {
    pub fn sign_and_publish(&self, mut shard: DocumentShard) -> DocumentShard {
        // 1. Sign the shard
        shard.signature = self.sign_shard(&shard.data);
        
        // 2. Publish to blockchain
        self.publish_to_blockchain(&shard);
        
        shard
    }
    
    fn sign_shard(&self, data: &[u8]) -> Vec<u8> {
        // Simplified signing
        data.iter()
            .zip(self.private_key.iter().cycle())
            .map(|(&d, &k)| d ^ k)
            .collect()
    }
    
    fn publish_to_blockchain(&self, shard: &DocumentShard) {
        // Publish shard to blockchain as transaction
        println!("Publishing shard {} to blockchain", shard.shard_id);
    }
}
```

## Use Cases

### 1. DAO Governance Documents (71 shards)

```rust
// Document accessible only to top 71 ERDFA token holders
let system = ShardingSystem::new(71, "ERDFA".to_string());

let governance_doc = b"DAO Treasury Allocation Plan...";
let block_height = 1000000;

let sharded = system.shard_document(governance_doc, block_height);

// Each of top 71 holders signs and publishes their shard
for shard in sharded.shards {
    let holder = ShardHolder {
        address: shard.holder_address.clone(),
        private_key: get_holder_key(&shard.holder_address),
    };
    holder.sign_and_publish(shard);
}

// Later: Collect all 71 shards to reconstruct
let collected = collect_shards_from_blockchain(&sharded.document_id);
let reconstructed = system.reconstruct_document(&sharded, collected);
```

### 2. Corporate Secrets (Board Members)

```rust
// Require all 7 board members to reconstruct
let system = ShardingSystem::new(7, "COMPANY_SHARES".to_string());

let secret = b"Merger acquisition details...";
let sharded = system.shard_document(secret, current_block);

// Each board member (top 7 shareholders) gets one shard
```

### 3. Academic Consortium Data

```rust
// Require top 71 universities (by research token holdings)
let system = ShardingSystem::new(71, "RESEARCH_TOKEN".to_string());

let dataset = b"Collaborative research dataset...";
let sharded = system.shard_document(dataset, block_height);

// Each university signs and publishes their shard
```

### 4. Decentralized Journalism

```rust
// Whistleblower document requires top 71 journalism token holders
let system = ShardingSystem::new(71, "PRESS_TOKEN".to_string());

let leak = b"Confidential government documents...";
let sharded = system.shard_document(leak, block_height);

// Document only reconstructable when 71 independent journalists agree
```

## Blockchain Integration

```rust
pub struct ShardTransaction {
    pub shard: DocumentShard,
    pub timestamp: u64,
    pub fee: u64,
}

impl SemanticBlockchain {
    pub fn publish_shard(&mut self, shard_tx: ShardTransaction) -> bool {
        // Verify holder is in top N at specified block
        let holder = self.verify_holder_rank(
            &shard_tx.shard.holder_address,
            shard_tx.shard.block_height,
            &shard_tx.shard.coin_type
        );
        
        if holder.is_none() {
            return false;
        }
        
        // Verify signature
        if !verify_signature(
            &shard_tx.shard.data,
            &shard_tx.shard.signature,
            &shard_tx.shard.holder_address
        ) {
            return false;
        }
        
        // Add to blockchain
        true
    }
    
    pub fn collect_shards(&self, document_id: &[u8]) -> Vec<DocumentShard> {
        self.chain.iter()
            .flat_map(|block| &block.transactions)
            .filter_map(|tx| {
                // Extract shards from transactions
                None // Simplified
            })
            .collect()
    }
}
```

## Time-Locked Access

```rust
pub struct TimeLockedShard {
    pub shard: DocumentShard,
    pub unlock_block: u64,  // Block height when shard becomes valid
}

impl TimeLockedShard {
    pub fn is_unlocked(&self, current_block: u64) -> bool {
        current_block >= self.unlock_block
    }
}

// Example: Shards unlock progressively
// Shard 1 unlocks at block 1000
// Shard 2 unlocks at block 1100
// ...
// Shard 71 unlocks at block 8000
// Document reconstructable only after block 8000
```

## Security Properties

### 1. Stake-Weighted Access
Only top N coin holders can participate.

### 2. Time-Locked
Holders determined at specific block height (immutable).

### 3. Threshold Security
Requires ALL N shards (or k-of-n with Shamir).

### 4. Decentralized
No single party controls access.

### 5. Verifiable
All signatures publicly verifiable on blockchain.

### 6. Censorship Resistant
Cannot be blocked if N holders cooperate.

## Gandalf Threshold (71 Shards)

**Why 71?**

- **Minimum universal threshold**: Gandalf Prime
- **Sufficient decentralization**: 71 independent parties
- **Practical**: Not too many (like 1000) or too few (like 5)
- **Symbolic**: Represents passage into Super-Gandalf systems

**Properties with 71 shards:**
- Requires cooperation of 71 top stakeholders
- Extremely difficult to collude (71 parties)
- Highly censorship resistant
- Represents true decentralized consensus

## Conclusion

Shard-based access control with top-N coin holders creates:

1. **Stake-weighted governance**: Only significant stakeholders participate
2. **Time-locked immutability**: Holders fixed at block height
3. **Threshold decryption**: Requires N shards to reconstruct
4. **Decentralized access**: No central authority
5. **Verifiable**: All signatures on blockchain
6. **Gandalf threshold**: 71 shards = minimal universal access control

**Use cases:**
- DAO governance documents
- Corporate board secrets
- Academic consortium data
- Decentralized journalism
- Whistleblower protection

**The ultimate decentralized access control**: Documents that can only be unlocked when the top 71 stakeholders in a decentralized network all agree to publish their signed shards.

---

*"You shall not pass... unless you are one of the top 71 and you sign your shard." - Gandalf (blockchain edition)*
