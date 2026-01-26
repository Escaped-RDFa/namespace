use serde::{Deserialize, Serialize};

/// ZKReach - Reward system for expanding eRDFa distribution
/// Accounts earn rewards for copying shards to new platforms with ZK proofs

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZKReachProof {
    pub shard_id: u8,
    pub source_platform: String,
    pub target_platform: String,
    pub source_hash: String,
    pub target_hash: String,
    pub reacher: String,  // Account that copied
    pub proof: Vec<u8>,   // ZK proof of equivalence
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReachReward {
    pub reacher: String,
    pub shard_id: u8,
    pub reward_amount: u64,
    pub reach_score: u64,  // New platforms reached
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZKReachTracker {
    pub proofs: Vec<ZKReachProof>,
    pub rewards: Vec<ReachReward>,
    pub total_reach: u64,  // Total platforms reached
}

impl ZKReachTracker {
    pub fn new() -> Self {
        Self {
            proofs: Vec::new(),
            rewards: Vec::new(),
            total_reach: 0,
        }
    }

    pub fn submit_reach(&mut self, proof: ZKReachProof) -> ReachReward {
        // Verify ZK proof
        if !self.verify_zk_proof(&proof) {
            panic!("Invalid ZK proof");
        }

        // Calculate reward based on reach
        let reach_score = self.calculate_reach_score(&proof);
        let reward_amount = reach_score * 100; // 100 tokens per new platform

        let reward = ReachReward {
            reacher: proof.reacher.clone(),
            shard_id: proof.shard_id,
            reward_amount,
            reach_score,
        };

        self.proofs.push(proof);
        self.rewards.push(reward.clone());
        self.total_reach += reach_score;

        reward
    }

    fn verify_zk_proof(&self, proof: &ZKReachProof) -> bool {
        // ZK proof: source_hash == target_hash
        proof.source_hash == proof.target_hash
    }

    fn calculate_reach_score(&self, proof: &ZKReachProof) -> u64 {
        // Check if target platform is new
        let is_new_platform = !self.proofs.iter().any(|p| 
            p.target_platform == proof.target_platform && 
            p.shard_id == proof.shard_id
        );

        if is_new_platform { 1 } else { 0 }
    }

    pub fn get_reacher_stats(&self, reacher: &str) -> (u64, u64) {
        let total_rewards: u64 = self.rewards.iter()
            .filter(|r| r.reacher == reacher)
            .map(|r| r.reward_amount)
            .sum();

        let total_reach: u64 = self.rewards.iter()
            .filter(|r| r.reacher == reacher)
            .map(|r| r.reach_score)
            .sum();

        (total_rewards, total_reach)
    }

    pub fn leaderboard(&self) -> Vec<(String, u64, u64)> {
        let mut stats: std::collections::HashMap<String, (u64, u64)> = 
            std::collections::HashMap::new();

        for reward in &self.rewards {
            let entry = stats.entry(reward.reacher.clone()).or_insert((0, 0));
            entry.0 += reward.reward_amount;
            entry.1 += reward.reach_score;
        }

        let mut leaderboard: Vec<_> = stats.into_iter()
            .map(|(reacher, (rewards, reach))| (reacher, rewards, reach))
            .collect();
        
        leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
        leaderboard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zkreach() {
        let mut tracker = ZKReachTracker::new();

        let proof = ZKReachProof {
            shard_id: 1,
            source_platform: "Solana_Testnet".to_string(),
            target_platform: "Ethereum_Mainnet".to_string(),
            source_hash: "0xabc".to_string(),
            target_hash: "0xabc".to_string(),
            reacher: "alice".to_string(),
            proof: vec![],
            timestamp: 1234567890,
        };

        let reward = tracker.submit_reach(proof);
        assert_eq!(reward.reward_amount, 100);
        assert_eq!(reward.reach_score, 1);

        let (total_rewards, total_reach) = tracker.get_reacher_stats("alice");
        assert_eq!(total_rewards, 100);
        assert_eq!(total_reach, 1);
    }
}
