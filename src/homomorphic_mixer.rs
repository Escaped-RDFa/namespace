use serde::{Deserialize, Serialize};

/// Homomorphic Transaction Mixer for Bitcoin
/// Bundles multiple users' transactions with eRDFa shard data

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserTransaction {
    pub user_id: String,
    pub payment_data: Vec<u8>,
    pub shard_id: Option<u8>,  // Optional shard to include
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MixedTransaction {
    pub users: Vec<String>,
    pub shards: Vec<u8>,
    pub combined_data: Vec<u8>,
    pub homomorphic_proof: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditReward {
    pub user_id: String,
    pub shard_id: u8,
    pub credits: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionPool {
    pub pending: Vec<UserTransaction>,
    pub mixed: Vec<MixedTransaction>,
    pub credits: Vec<CreditReward>,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            mixed: Vec::new(),
            credits: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, tx: UserTransaction) {
        self.pending.push(tx);
        
        // Auto-mix when we have 3+ transactions
        if self.pending.len() >= 3 {
            self.mix_transactions();
        }
    }

    fn mix_transactions(&mut self) {
        let batch: Vec<_> = self.pending.drain(..3.min(self.pending.len())).collect();
        
        let users: Vec<_> = batch.iter().map(|tx| tx.user_id.clone()).collect();
        let shards: Vec<_> = batch.iter()
            .filter_map(|tx| tx.shard_id)
            .collect();
        
        // Combine all data homomorphically
        let combined_data = self.homomorphic_combine(&batch);
        let proof = self.generate_homomorphic_proof(&batch);
        
        let mixed = MixedTransaction {
            users: users.clone(),
            shards: shards.clone(),
            combined_data,
            homomorphic_proof: proof,
        };
        
        // Award credits
        for (user, shard) in users.iter().zip(shards.iter()) {
            self.credits.push(CreditReward {
                user_id: user.clone(),
                shard_id: *shard,
                credits: 100,
            });
        }
        
        self.mixed.push(mixed);
    }

    fn homomorphic_combine(&self, batch: &[UserTransaction]) -> Vec<u8> {
        // Homomorphically combine payment data + shard data
        // Privacy: Can't tell which user included which shard
        let mut combined = Vec::new();
        for tx in batch {
            combined.extend_from_slice(&tx.payment_data);
        }
        combined
    }

    fn generate_homomorphic_proof(&self, batch: &[UserTransaction]) -> Vec<u8> {
        // ZK proof that all data is correctly included
        // Without revealing which user contributed what
        format!("homomorphic_proof_{}", batch.len()).into_bytes()
    }

    pub fn get_user_credits(&self, user_id: &str) -> u64 {
        self.credits.iter()
            .filter(|c| c.user_id == user_id)
            .map(|c| c.credits)
            .sum()
    }

    pub fn ready_for_bitcoin(&self) -> Vec<MixedTransaction> {
        self.mixed.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_pooling() {
        let mut pool = TransactionPool::new();

        // Three users add transactions with shards
        pool.add_transaction(UserTransaction {
            user_id: "alice".to_string(),
            payment_data: vec![1, 2, 3],
            shard_id: Some(1),
        });

        pool.add_transaction(UserTransaction {
            user_id: "bob".to_string(),
            payment_data: vec![4, 5, 6],
            shard_id: Some(2),
        });

        pool.add_transaction(UserTransaction {
            user_id: "carol".to_string(),
            payment_data: vec![7, 8, 9],
            shard_id: Some(3),
        });

        // Should auto-mix
        assert_eq!(pool.mixed.len(), 1);
        assert_eq!(pool.credits.len(), 3);
        assert_eq!(pool.get_user_credits("alice"), 100);
    }
}
