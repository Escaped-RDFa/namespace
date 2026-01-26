//! Semantic Blockchain Module
//! 
//! Blockchain for semantic web with proof-of-semantic-work

use crate::crypto::{ExtractionWitness, ChannelMatrix};

/// Semantic transaction
#[derive(Debug, Clone)]
pub struct SemanticTransaction {
    pub rdfa_data: Vec<u8>,
    pub witness: ExtractionWitness,
    pub channel_matrix: ChannelMatrix,
    pub fee: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Block header
#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub previous_hash: [u8; 32],
    pub timestamp: u64,
    pub nonce: u64,
    pub difficulty: u64,
}

/// Semantic block
#[derive(Debug, Clone)]
pub struct SemanticBlock {
    pub header: BlockHeader,
    pub transactions: Vec<SemanticTransaction>,
    pub merkle_root: [u8; 32],
    pub semantic_proof: Vec<u8>,
    pub miner_address: Vec<u8>,
    pub reward: u64,
}

/// Fee schedule
pub struct FeeSchedule {
    pub base_fee: u64,
    pub per_byte_fee: u64,
    pub per_channel_fee: u64,
    pub verification_fee: u64,
}

impl FeeSchedule {
    pub fn calculate_fee(&self, tx: &SemanticTransaction) -> u64 {
        self.base_fee
            + (tx.rdfa_data.len() as u64 * self.per_byte_fee)
            + (tx.channel_matrix.channels as u64 * self.per_channel_fee)
            + self.verification_fee
    }
}

/// Miner reward
pub struct MinerReward {
    pub block_reward: u64,
    pub transaction_fees: u64,
    pub storage_bonus: u64,
    pub verification_bonus: u64,
}

impl MinerReward {
    pub fn total(&self) -> u64 {
        self.block_reward 
            + self.transaction_fees 
            + self.storage_bonus 
            + self.verification_bonus
    }
}

/// Semantic blockchain
pub struct SemanticBlockchain {
    pub chain: Vec<SemanticBlock>,
    pub mempool: Vec<SemanticTransaction>,
    pub fee_schedule: FeeSchedule,
}

impl SemanticBlockchain {
    pub fn new() -> Self {
        Self {
            chain: vec![Self::genesis_block()],
            mempool: Vec::new(),
            fee_schedule: FeeSchedule {
                base_fee: 10,
                per_byte_fee: 1,
                per_channel_fee: 5,
                verification_fee: 20,
            },
        }
    }
    
    fn genesis_block() -> SemanticBlock {
        SemanticBlock {
            header: BlockHeader {
                previous_hash: [0; 32],
                timestamp: 0,
                nonce: 0,
                difficulty: 1,
            },
            transactions: Vec::new(),
            merkle_root: [0; 32],
            semantic_proof: Vec::new(),
            miner_address: Vec::new(),
            reward: 0,
        }
    }
    
    pub fn add_transaction(&mut self, tx: SemanticTransaction) -> bool {
        if self.validate_transaction(&tx) {
            self.mempool.push(tx);
            true
        } else {
            false
        }
    }
    
    pub fn validate_transaction(&self, tx: &SemanticTransaction) -> bool {
        // 1. Verify witness
        if !tx.witness.verify(&tx.rdfa_data) {
            return false;
        }
        
        // 2. Verify fee
        let required_fee = self.fee_schedule.calculate_fee(tx);
        if tx.fee < required_fee {
            return false;
        }
        
        true
    }
    
    pub fn mine_block(&mut self, miner_address: Vec<u8>) -> Option<SemanticBlock> {
        if self.mempool.is_empty() {
            return None;
        }
        
        let transactions = self.mempool.drain(..).collect::<Vec<_>>();
        let merkle_root = self.calculate_merkle_root(&transactions);
        let total_fees: u64 = transactions.iter().map(|tx| tx.fee).sum();
        
        let block = SemanticBlock {
            header: BlockHeader {
                previous_hash: self.get_last_block_hash(),
                timestamp: current_timestamp(),
                nonce: 0,
                difficulty: 1,
            },
            transactions,
            merkle_root,
            semantic_proof: Vec::new(),
            miner_address,
            reward: 50 + total_fees, // Block reward + fees
        };
        
        self.chain.push(block.clone());
        Some(block)
    }
    
    fn calculate_merkle_root(&self, transactions: &[SemanticTransaction]) -> [u8; 32] {
        let mut root = [0u8; 32];
        for tx in transactions {
            for (i, &byte) in tx.rdfa_data.iter().enumerate() {
                root[i % 32] ^= byte;
            }
        }
        root
    }
    
    fn get_last_block_hash(&self) -> [u8; 32] {
        if let Some(last_block) = self.chain.last() {
            last_block.merkle_root
        } else {
            [0; 32]
        }
    }
    
    pub fn query_rdfa(&self, predicate: &str) -> Vec<String> {
        self.chain.iter()
            .flat_map(|block| &block.transactions)
            .filter_map(|tx| {
                let rdfa = String::from_utf8_lossy(&tx.rdfa_data);
                if rdfa.contains(predicate) {
                    Some(rdfa.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
    
    pub fn get_block_count(&self) -> usize {
        self.chain.len()
    }
    
    pub fn get_total_transactions(&self) -> usize {
        self.chain.iter()
            .map(|block| block.transactions.len())
            .sum()
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blockchain_creation() {
        let blockchain = SemanticBlockchain::new();
        assert_eq!(blockchain.get_block_count(), 1); // Genesis block
    }
    
    #[test]
    fn test_transaction_validation() {
        let blockchain = SemanticBlockchain::new();
        let data = b"<div property=\"name\">Test</div>";
        
        let tx = SemanticTransaction {
            rdfa_data: data.to_vec(),
            witness: ExtractionWitness::generate(data, &[0, 1, 2]),
            channel_matrix: ChannelMatrix::new(8),
            fee: 100,
            timestamp: current_timestamp(),
            signature: Vec::new(),
        };
        
        assert!(blockchain.validate_transaction(&tx));
    }
    
    #[test]
    fn test_mining() {
        let mut blockchain = SemanticBlockchain::new();
        let data = b"<div property=\"name\">Alice</div>";
        
        let tx = SemanticTransaction {
            rdfa_data: data.to_vec(),
            witness: ExtractionWitness::generate(data, &[0, 1, 2]),
            channel_matrix: ChannelMatrix::new(8),
            fee: 100,
            timestamp: current_timestamp(),
            signature: Vec::new(),
        };
        
        blockchain.add_transaction(tx);
        let block = blockchain.mine_block(vec![1, 2, 3, 4]);
        
        assert!(block.is_some());
        assert_eq!(blockchain.get_block_count(), 2);
    }
    
    #[test]
    fn test_query() {
        let mut blockchain = SemanticBlockchain::new();
        let data = b"<div property=\"foaf:name\">Bob</div>";
        
        let tx = SemanticTransaction {
            rdfa_data: data.to_vec(),
            witness: ExtractionWitness::generate(data, &[0, 1, 2]),
            channel_matrix: ChannelMatrix::new(8),
            fee: 100,
            timestamp: current_timestamp(),
            signature: Vec::new(),
        };
        
        blockchain.add_transaction(tx);
        blockchain.mine_block(vec![1, 2, 3, 4]);
        
        let results = blockchain.query_rdfa("foaf:name");
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("Bob"));
    }
    
    #[test]
    fn test_fee_calculation() {
        let schedule = FeeSchedule {
            base_fee: 10,
            per_byte_fee: 1,
            per_channel_fee: 5,
            verification_fee: 20,
        };
        
        let tx = SemanticTransaction {
            rdfa_data: vec![0; 100],
            witness: ExtractionWitness::generate(&[0; 100], &[0, 1, 2]),
            channel_matrix: ChannelMatrix::new(8),
            fee: 0,
            timestamp: 0,
            signature: Vec::new(),
        };
        
        let fee = schedule.calculate_fee(&tx);
        assert_eq!(fee, 10 + 100 + 40 + 20); // base + bytes + channels + verification
    }
}
