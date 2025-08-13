#![no_std]

use soroban_sdk::{
    symbol_short, vec, Bytes, BytesN, Env, Symbol, Vec
};

/// Storage keys for the LeanIMT
pub const TREE_ROOT_KEY: Symbol = symbol_short!("root");
pub const TREE_DEPTH_KEY: Symbol = symbol_short!("depth");
pub const TREE_LEAVES_KEY: Symbol = symbol_short!("leaves");

/// Lean Incremental Merkle Tree implementation compatible with merkleProof.circom
/// 
/// This implementation follows the LeanIMT design where:
/// 1. Every node with two children is the hash of its left and right nodes
/// 2. Every node with one child has the same value as its child node
/// 3. Tree is always built from leaves to root
/// 4. Tree is always balanced by construction
/// 5. Tree depth is dynamic and can increase with insertion of new leaves
pub struct LeanIMT {
    env: Env,
    leaves: Vec<BytesN<32>>,
    depth: u32,
    root: BytesN<32>,
}

impl LeanIMT {
    /// Creates a new empty LeanIMT
    pub fn new(env: Env) -> Self {
        let empty_hash = BytesN::from_array(&env, &[0u8; 32]);
        Self {
            env: env.clone(),
            leaves: vec![&env],
            depth: 0,
            root: empty_hash,
        }
    }

    /// Inserts a new leaf into the tree
    pub fn insert(&mut self, leaf: BytesN<32>) {
        self.leaves.push_back(leaf);
        self.recompute_tree();
    }

    /// Gets the current root of the tree
    pub fn get_root(&self) -> BytesN<32> {
        self.root.clone()
    }

    /// Gets the current depth of the tree
    pub fn get_depth(&self) -> u32 {
        self.depth
    }

    /// Gets the number of leaves in the tree
    pub fn get_leaf_count(&self) -> u32 {
        self.leaves.len() as u32
    }

    /// Generates a merkle proof for a given leaf index
    /// Returns a tuple of (siblings, actual_depth) where:
    /// - siblings: Vector of sibling values along the path to the root
    /// - actual_depth: Current tree depth
    pub fn generate_proof(&self, leaf_index: u32) -> Option<(Vec<BytesN<32>>, u32)> {
        if leaf_index >= self.leaves.len() as u32 {
            return None;
        }

        let mut siblings = vec![&self.env];
        let mut current_index = leaf_index;
        let mut current_depth = 0;
        
        // Build the proof by traversing up the tree
        while current_depth < self.depth {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };
            
            // Get sibling value (0 if sibling doesn't exist)
            let sibling = if sibling_index < self.leaves.len() as u32 {
                self.leaves.get(sibling_index).unwrap()
            } else {
                BytesN::from_array(&self.env, &[0u8; 32])
            };
            
            siblings.push_back(sibling);
            current_index = current_index / 2;
            current_depth += 1;
        }

        // Add the root level sibling (which is the root itself)
        if self.depth > 0 {
            siblings.push_back(self.root.clone());
        }

        Some((siblings, self.depth))
    }

    /// Recomputes the entire tree after insertion
    fn recompute_tree(&mut self) {
        if self.leaves.is_empty() {
            self.root = BytesN::from_array(&self.env, &[0u8; 32]);
            self.depth = 0;
            return;
        }

        // Calculate required depth
        let leaf_count = self.leaves.len() as u32;
        let mut new_depth = 0;
        let mut temp_count = leaf_count;
        while temp_count > 1 {
            temp_count = (temp_count + 1) / 2;
            new_depth += 1;
        }
        self.depth = new_depth;

        // Build tree levels
        let mut current_level = self.leaves.clone();
        
        for _level in 0..self.depth {
            let mut next_level = vec![&self.env];
            let mut i: u32 = 0;
            
            while i < current_level.len() as u32 {
                if i + 1 < current_level.len() as u32 {
                    // Hash two children
                    let hash = self.hash_pair(&current_level.get(i).unwrap(), &current_level.get(i + 1).unwrap());
                    next_level.push_back(hash);
                } else {
                    // Propagate single child (LeanIMT property)
                    next_level.push_back(current_level.get(i).unwrap());
                }
                i += 2;
            }
            
            current_level = next_level;
        }

        // Set root
        if current_level.len() > 0 {
            self.root = current_level.get(0).unwrap();
        }
    }

    /// Hashes two values using Keccak256 hash function
    /// This provides a cryptographically secure hash for the merkle tree
    fn hash_pair(&self, left: &BytesN<32>, right: &BytesN<32>) -> BytesN<32> {
        // Concatenate left and right bytes
        let left_bytes = left.to_array();
        let right_bytes = right.to_array();
        
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&left_bytes);
        combined[32..].copy_from_slice(&right_bytes);
        
        // Convert to Bytes and use Keccak256 from the crypto module
        let combined_bytes = Bytes::from_slice(&self.env, &combined);
        let hash_result = self.env.crypto().keccak256(&combined_bytes);
        
        // Convert Hash<32> back to BytesN<32>
        BytesN::from_array(&self.env, &hash_result.to_array())
    }

    /// Serializes the tree state for storage
    pub fn to_storage(&self) -> (Vec<BytesN<32>>, u32, BytesN<32>) {
        (self.leaves.clone(), self.depth, self.root.clone())
    }

    /// Deserializes the tree state from storage
    pub fn from_storage(env: Env, leaves: Vec<BytesN<32>>, depth: u32, root: BytesN<32>) -> Self {
        Self {
            env,
            leaves,
            depth,
            root,
        }
    }

    /// Gets all leaves in the tree
    pub fn get_leaves(&self) -> &Vec<BytesN<32>> {
        &self.leaves
    }

    /// Checks if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Gets a leaf at a specific index
    pub fn get_leaf(&self, index: usize) -> Option<BytesN<32>> {
        match self.leaves.get(index.try_into().unwrap()) {
            Some(leaf) => Some(leaf.clone()),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_tree() {
        let env = Env::default();
        let tree = LeanIMT::new(env.clone());
        
        assert_eq!(tree.get_depth(), 0);
        assert_eq!(tree.get_leaf_count(), 0);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_insert_single_leaf() {
        let env = Env::default();
        let mut tree = LeanIMT::new(env.clone());
        
        let leaf = BytesN::from_array(&env, &[1u8; 32]);
        tree.insert(leaf);
        
        assert_eq!(tree.get_depth(), 0);
        assert_eq!(tree.get_leaf_count(), 1);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_insert_two_leaves() {
        let env = Env::default();
        let mut tree = LeanIMT::new(env.clone());
        
        let leaf1 = BytesN::from_array(&env, &[1u8; 32]);
        let leaf2 = BytesN::from_array(&env, &[2u8; 32]);
        
        tree.insert(leaf1);
        tree.insert(leaf2);
        
        assert_eq!(tree.get_depth(), 1);
        assert_eq!(tree.get_leaf_count(), 2);
    }

    #[test]
    fn test_generate_proof() {
        let env = Env::default();
        let mut tree = LeanIMT::new(env.clone());
        
        let leaf1 = BytesN::from_array(&env, &[1u8; 32]);
        let leaf2 = BytesN::from_array(&env, &[2u8; 32]);
        
        tree.insert(leaf1);
        tree.insert(leaf2);
        
        // Generate proof for first leaf
        let proof = tree.generate_proof(0);
        assert!(proof.is_some());
        
        let (siblings, depth) = proof.unwrap();
        assert_eq!(depth, 1);
        assert_eq!(siblings.len(), 2); // root level + leaf level
    }

    #[test]
    fn test_storage_serialization() {
        let env = Env::default();
        let mut tree = LeanIMT::new(env.clone());
        
        let leaf = BytesN::from_array(&env, &[1u8; 32]);
        tree.insert(leaf);
        
        let (leaves, depth, root) = tree.to_storage();
        let restored_tree = LeanIMT::from_storage(env.clone(), leaves, depth, root);
        
        assert_eq!(tree.get_depth(), restored_tree.get_depth());
        assert_eq!(tree.get_leaf_count(), restored_tree.get_leaf_count());
        assert_eq!(tree.get_root(), restored_tree.get_root());
    }
}
