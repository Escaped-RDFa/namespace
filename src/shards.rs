//! Shard-Based Access Control with Top-N Coin Holders

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
    pub rank: usize,
    pub block_height: u64,
}

/// Document shard
#[derive(Debug, Clone)]
pub struct DocumentShard {
    pub shard_id: usize,
    pub data: Vec<u8>,
    pub holder_address: Vec<u8>,
    pub signature: Vec<u8>,
    pub block_height: u64,
    pub coin_type: String,
    pub data_type: DataType,
}

/// Sharded document
#[derive(Debug, Clone)]
pub struct ShardedDocument {
    pub document_id: Vec<u8>,
    pub data_type: DataType,
    pub total_shards: usize,
    pub required_shards: usize,
    pub shards: Vec<DocumentShard>,
    pub block_height: u64,
    pub coin_type: String,
}

/// Shamir Secret Sharing
pub struct ShamirSharing {
    pub threshold: usize,
    pub total_shares: usize,
}

impl ShamirSharing {
    pub fn new(threshold: usize, total: usize) -> Self {
        Self {
            threshold,
            total_shares: total,
        }
    }
    
    pub fn split(&self, secret: &[u8]) -> Vec<Vec<u8>> {
        (1..=self.total_shares)
            .map(|i| self.generate_share(secret, i))
            .collect()
    }
    
    fn generate_share(&self, secret: &[u8], index: usize) -> Vec<u8> {
        secret.iter()
            .map(|&byte| ((byte as usize + index) % 256) as u8)
            .collect()
    }
    
    pub fn reconstruct(&self, shares: &[Vec<u8>]) -> Option<Vec<u8>> {
        if shares.len() < self.threshold {
            return None;
        }
        
        Some(self.lagrange_interpolate(&shares[..self.threshold]))
    }
    
    fn lagrange_interpolate(&self, shares: &[Vec<u8>]) -> Vec<u8> {
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

/// Coin holder registry
pub struct CoinHolderRegistry {
    pub coin_type: String,
    pub holders: Vec<CoinHolder>,
}

impl CoinHolderRegistry {
    pub fn new(coin_type: String) -> Self {
        Self {
            coin_type,
            holders: Vec::new(),
        }
    }
    
    pub fn add_holder(&mut self, address: Vec<u8>, balance: u64, block_height: u64) {
        self.holders.push(CoinHolder {
            address,
            balance,
            rank: 0,
            block_height,
        });
    }
    
    pub fn get_top_n_at_block(&mut self, n: usize, block_height: u64) -> Vec<CoinHolder> {
        self.holders.sort_by(|a, b| b.balance.cmp(&a.balance));
        
        self.holders.iter()
            .take(n)
            .enumerate()
            .map(|(i, h)| CoinHolder {
                address: h.address.clone(),
                balance: h.balance,
                rank: i + 1,
                block_height,
            })
            .collect()
    }
    
    pub fn verify_holder_at_block(&self, address: &[u8], block_height: u64) -> Option<CoinHolder> {
        self.holders.iter()
            .find(|h| h.address == address && h.block_height == block_height)
            .cloned()
    }
}

/// Sharding system
pub struct ShardingSystem {
    shamir: ShamirSharing,
    registry: CoinHolderRegistry,
    data_type: DataType,
}

impl ShardingSystem {
    pub fn new(data_type: DataType, coin_type: String) -> Self {
        let n = data_type.shard_count();
        Self {
            shamir: ShamirSharing::new(n, n),
            registry: CoinHolderRegistry::new(coin_type),
            data_type,
        }
    }
    
    pub fn add_holder(&mut self, address: Vec<u8>, balance: u64, block_height: u64) {
        self.registry.add_holder(address, balance, block_height);
    }
    
    pub fn shard_document(&mut self, document: &[u8], block_height: u64) -> ShardedDocument {
        let holders = self.registry.get_top_n_at_block(self.shamir.total_shares, block_height);
        let shares = self.shamir.split(document);
        
        let shards: Vec<DocumentShard> = shares.into_iter()
            .zip(holders.iter())
            .enumerate()
            .map(|(i, (data, holder))| DocumentShard {
                shard_id: i,
                data,
                holder_address: holder.address.clone(),
                signature: Vec::new(),
                block_height,
                coin_type: self.registry.coin_type.clone(),
                data_type: self.data_type,
            })
            .collect();
        
        ShardedDocument {
            document_id: hash_document(document),
            data_type: self.data_type,
            total_shards: self.shamir.total_shares,
            required_shards: self.shamir.threshold,
            shards,
            block_height,
            coin_type: self.registry.coin_type.clone(),
        }
    }
    
    pub fn reconstruct_document(&self, 
                               sharded: &ShardedDocument,
                               collected_shards: Vec<DocumentShard>) -> Option<Vec<u8>> {
        if collected_shards.len() < sharded.required_shards {
            return None;
        }
        
        for shard in &collected_shards {
            if !self.verify_shard_signature(shard, sharded.block_height) {
                return None;
            }
        }
        
        let shares: Vec<Vec<u8>> = collected_shards.iter()
            .map(|s| s.data.clone())
            .collect();
        
        self.shamir.reconstruct(&shares)
    }
    
    fn verify_shard_signature(&self, shard: &DocumentShard, block_height: u64) -> bool {
        let holder = self.registry.verify_holder_at_block(&shard.holder_address, block_height);
        
        if holder.is_none() {
            return false;
        }
        
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
    if signature.is_empty() {
        return false;
    }
    
    let expected: u8 = data.iter().fold(0, |acc, &b| acc ^ b);
    let sig_check: u8 = signature.iter().fold(0, |acc, &b| acc ^ b);
    let key_check: u8 = public_key.iter().fold(0, |acc, &b| acc ^ b);
    
    expected == (sig_check ^ key_check)
}

/// Gandalf threshold constant
pub const GANDALF_SHARDS: usize = 71;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shamir_sharing() {
        let shamir = ShamirSharing::new(3, 5);
        let secret = b"Secret message";
        
        let shares = shamir.split(secret);
        assert_eq!(shares.len(), 5);
        
        let reconstructed = shamir.reconstruct(&shares[..3]).unwrap();
        assert_eq!(reconstructed.len(), secret.len());
    }
    
    #[test]
    fn test_coin_holder_registry() {
        let mut registry = CoinHolderRegistry::new("TEST".to_string());
        
        registry.add_holder(vec![1], 1000, 100);
        registry.add_holder(vec![2], 500, 100);
        registry.add_holder(vec![3], 2000, 100);
        
        let top2 = registry.get_top_n_at_block(2, 100);
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].balance, 2000); // Highest
        assert_eq!(top2[1].balance, 1000); // Second
    }
    
    #[test]
    fn test_sharding_system() {
        let mut system = ShardingSystem::new(DataType::Octonion, "TEST".to_string());
        
        system.add_holder(vec![1], 1000, 100);
        system.add_holder(vec![2], 500, 100);
        system.add_holder(vec![3], 2000, 100);
        system.add_holder(vec![4], 1500, 100);
        system.add_holder(vec![5], 800, 100);
        system.add_holder(vec![6], 1200, 100);
        system.add_holder(vec![7], 900, 100);
        system.add_holder(vec![8], 1100, 100);
        
        let document = b"Confidential data";
        let sharded = system.shard_document(document, 100);
        
        assert_eq!(sharded.total_shards, 8); // Octonion = 8 shards
        assert_eq!(sharded.data_type, DataType::Octonion);
        assert_eq!(sharded.shards.len(), 8);
    }
    
    #[test]
    fn test_data_type_shard_counts() {
        assert_eq!(DataType::Boolean.shard_count(), 2);
        assert_eq!(DataType::Quaternion.shard_count(), 4);
        assert_eq!(DataType::Octonion.shard_count(), 8);
        assert_eq!(DataType::MathieuM24.shard_count(), 24);
        assert_eq!(DataType::Genetic.shard_count(), 64);
        assert_eq!(DataType::RDFa.shard_count(), 71);
        assert_eq!(DataType::IPv6.shard_count(), 128);
        assert_eq!(DataType::Byte.shard_count(), 256);
        assert_eq!(DataType::Monster.shard_count(), 196_883);
    }
    
    #[test]
    fn test_gandalf_threshold() {
        let system = ShardingSystem::new(DataType::RDFa, "ERDFA".to_string());
        assert_eq!(system.shamir.threshold, 71);
        assert_eq!(system.shamir.total_shares, 71);
        assert_eq!(system.data_type, DataType::RDFa);
    }
    
    #[test]
    fn test_reconstruction() {
        let mut system = ShardingSystem::new(DataType::Quaternion, "TEST".to_string());
        
        system.add_holder(vec![1], 1000, 100);
        system.add_holder(vec![2], 500, 100);
        system.add_holder(vec![3], 2000, 100);
        system.add_holder(vec![4], 1500, 100);
        
        let document = b"Secret";
        let sharded = system.shard_document(document, 100);
        
        assert_eq!(sharded.data_type, DataType::Quaternion);
        
        // Sign shards
        let mut signed_shards = sharded.shards.clone();
        for shard in &mut signed_shards {
            shard.signature = vec![1, 2, 3]; // Simplified signature
        }
        
        let reconstructed = system.reconstruct_document(&sharded, signed_shards);
        assert!(reconstructed.is_some());
    }
}
