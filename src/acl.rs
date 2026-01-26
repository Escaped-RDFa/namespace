//! Multi-Layered ACL System for Nested Semantic Information

use std::collections::HashMap;
use crate::crypto::ExtractionWitness;

/// Access level for semantic data
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessLevel {
    Public = 0,
    Authenticated = 1,
    Subscriber = 2,
    Private = 3,
    Secret = 4,
}

/// Access Control Entry
#[derive(Debug, Clone)]
pub struct ACLEntry {
    pub level: AccessLevel,
    pub required_keys: Vec<Vec<u8>>,
    pub threshold: usize,
    pub encryption_key: Vec<u8>,
    pub parent_layer: Option<usize>,
}

/// Multi-layered ACL
#[derive(Debug, Clone)]
pub struct LayeredACL {
    pub layers: Vec<ACLEntry>,
    pub owner: Vec<u8>,
}

impl LayeredACL {
    pub fn new(owner: Vec<u8>) -> Self {
        Self {
            layers: vec![
                ACLEntry {
                    level: AccessLevel::Public,
                    required_keys: Vec::new(),
                    threshold: 0,
                    encryption_key: Vec::new(),
                    parent_layer: None,
                },
            ],
            owner,
        }
    }
    
    pub fn add_layer(&mut self, 
                     level: AccessLevel, 
                     required_keys: Vec<Vec<u8>>,
                     threshold: usize,
                     encryption_key: Vec<u8>) -> usize {
        let parent = self.layers.len() - 1;
        self.layers.push(ACLEntry {
            level,
            required_keys,
            threshold,
            encryption_key,
            parent_layer: Some(parent),
        });
        self.layers.len() - 1
    }
    
    pub fn can_access(&self, layer: usize, keys: &[Vec<u8>]) -> bool {
        if layer >= self.layers.len() {
            return false;
        }
        
        let entry = &self.layers[layer];
        
        if entry.level == AccessLevel::Public {
            return true;
        }
        
        let matching_keys = keys.iter()
            .filter(|k| entry.required_keys.contains(k))
            .count();
        
        matching_keys >= entry.threshold
    }
    
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}

/// Nested encryption for layered data
#[derive(Debug, Clone)]
pub struct NestedEncryption {
    pub layers: Vec<Vec<u8>>,
}

impl NestedEncryption {
    pub fn encrypt_nested(data: &str, acl: &LayeredACL) -> Self {
        let mut layers = Vec::new();
        let mut current_data = data.as_bytes().to_vec();
        
        for entry in acl.layers.iter().rev() {
            if entry.level != AccessLevel::Public {
                current_data = encrypt_layer(&current_data, &entry.encryption_key);
            }
            layers.push(current_data.clone());
        }
        
        layers.reverse();
        Self { layers }
    }
    
    pub fn decrypt_layer(&self, layer: usize, key: &[u8]) -> Option<Vec<u8>> {
        if layer >= self.layers.len() {
            return None;
        }
        
        Some(decrypt_layer(&self.layers[layer], key))
    }
    
    pub fn decrypt_to_layer(&self, target_layer: usize, keys: &[Vec<u8>]) -> Option<Vec<u8>> {
        if target_layer >= self.layers.len() {
            return None;
        }
        
        let mut data = self.layers[0].clone();
        
        for layer in 1..=target_layer {
            if layer < keys.len() {
                data = decrypt_layer(&data, &keys[layer]);
            } else {
                return None;
            }
        }
        
        Some(data)
    }
}

fn encrypt_layer(data: &[u8], key: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }
    data.iter()
        .zip(key.iter().cycle())
        .map(|(&d, &k)| d ^ k)
        .collect()
}

fn decrypt_layer(data: &[u8], key: &[u8]) -> Vec<u8> {
    encrypt_layer(data, key)
}

/// Layered semantic transaction
#[derive(Debug, Clone)]
pub struct LayeredSemanticTransaction {
    pub rdfa_data: Vec<u8>,
    pub nested_layers: NestedEncryption,
    pub acl: LayeredACL,
    pub witnesses: Vec<ExtractionWitness>,
    pub fee: u64,
    pub timestamp: u64,
}

impl LayeredSemanticTransaction {
    pub fn new(data: &str, acl: LayeredACL) -> Self {
        let nested = NestedEncryption::encrypt_nested(data, &acl);
        let witnesses = (0..acl.layers.len())
            .map(|i| ExtractionWitness::generate(&nested.layers[i], &[i as u8]))
            .collect();
        
        Self {
            rdfa_data: nested.layers[0].clone(),
            nested_layers: nested,
            acl,
            witnesses,
            fee: 0,
            timestamp: 0,
        }
    }
    
    pub fn access_layer(&self, layer: usize, keys: &[Vec<u8>]) -> Option<Vec<u8>> {
        if !self.acl.can_access(layer, keys) {
            return None;
        }
        
        self.nested_layers.decrypt_to_layer(layer, keys)
    }
    
    pub fn verify_layer(&self, layer: usize) -> bool {
        if layer >= self.witnesses.len() {
            return false;
        }
        
        self.witnesses[layer].verify(&self.nested_layers.layers[layer])
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
    fn test_acl_creation() {
        let acl = LayeredACL::new(vec![1, 2, 3]);
        assert_eq!(acl.layer_count(), 1); // Public layer
    }
    
    #[test]
    fn test_add_layers() {
        let mut acl = LayeredACL::new(vec![1, 2, 3]);
        
        acl.add_layer(
            AccessLevel::Authenticated,
            vec![vec![4, 5, 6]],
            1,
            vec![7, 8, 9]
        );
        
        assert_eq!(acl.layer_count(), 2);
    }
    
    #[test]
    fn test_access_control() {
        let mut acl = LayeredACL::new(vec![1, 2, 3]);
        
        let auth_key = vec![4, 5, 6];
        acl.add_layer(
            AccessLevel::Authenticated,
            vec![auth_key.clone()],
            1,
            vec![7, 8, 9]
        );
        
        // Public layer accessible to all
        assert!(acl.can_access(0, &[]));
        
        // Auth layer requires key
        assert!(!acl.can_access(1, &[]));
        assert!(acl.can_access(1, &[auth_key]));
    }
    
    #[test]
    fn test_nested_encryption() {
        let mut acl = LayeredACL::new(vec![1, 2, 3]);
        acl.add_layer(
            AccessLevel::Authenticated,
            vec![vec![4, 5, 6]],
            1,
            vec![10, 20, 30]
        );
        
        let data = "Secret message";
        let nested = NestedEncryption::encrypt_nested(data, &acl);
        
        assert_eq!(nested.layers.len(), 2);
    }
    
    #[test]
    fn test_layered_transaction() {
        let mut acl = LayeredACL::new(vec![1, 2, 3]);
        let key1 = vec![4, 5, 6];
        let enc_key1 = vec![10, 20, 30];
        
        acl.add_layer(
            AccessLevel::Authenticated,
            vec![key1.clone()],
            1,
            enc_key1.clone()
        );
        
        let data = "Layered data";
        let tx = LayeredSemanticTransaction::new(data, acl);
        
        // Public layer accessible
        let public_data = tx.access_layer(0, &[]);
        assert!(public_data.is_some());
        
        // Auth layer requires key
        let auth_data = tx.access_layer(1, &[vec![], enc_key1]);
        assert!(auth_data.is_some());
    }
    
    #[test]
    fn test_threshold_access() {
        let mut acl = LayeredACL::new(vec![1, 2, 3]);
        
        let key1 = vec![4, 5, 6];
        let key2 = vec![7, 8, 9];
        let key3 = vec![10, 11, 12];
        
        // Require 2-of-3 keys
        acl.add_layer(
            AccessLevel::Secret,
            vec![key1.clone(), key2.clone(), key3.clone()],
            2,
            vec![13, 14, 15]
        );
        
        // 1 key not enough
        assert!(!acl.can_access(1, &[key1.clone()]));
        
        // 2 keys sufficient
        assert!(acl.can_access(1, &[key1.clone(), key2.clone()]));
        
        // 3 keys also work
        assert!(acl.can_access(1, &[key1, key2, key3]));
    }
}
