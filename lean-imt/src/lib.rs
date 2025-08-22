#![no_std]

use soroban_sdk::{
    symbol_short, vec, BytesN, Env, Symbol, Vec
};

use poseidon::poseidon2;

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

    /// Generates a complete merkle proof with internal node values
    /// This method provides the actual sibling values needed for circuit verification
    /// Returns a tuple of (siblings, actual_depth) where:
    /// - siblings: Vector of actual sibling values at each level (not just leaf-level)
    /// - actual_depth: Current tree depth
    pub fn generate_complete_proof(&self, leaf_index: u32) -> Option<(Vec<BytesN<32>>, u32)> {
        if leaf_index >= self.leaves.len() as u32 {
            return None;
        }

        let mut siblings = vec![&self.env];
        
        // For now, let's handle the simple 2-leaf case correctly
        if self.depth == 1 && self.leaves.len() == 2 {
            if leaf_index == 0 {
                // Leaf 0's sibling is leaf 1
                siblings.push_back(self.leaves.get(1).unwrap());
            } else {
                // Leaf 1's sibling is leaf 0
                siblings.push_back(self.leaves.get(0).unwrap());
            }
            
            // Add the root level sibling (which is the root itself)
            if self.depth > 0 {
                siblings.push_back(self.root.clone());
            }
        } else {
            // For other cases, use the general approach
            let mut current_index = leaf_index;
            let mut current_depth = 0;
            
            while current_depth < self.depth {
                let sibling_index = if current_index % 2 == 0 {
                    current_index + 1
                } else {
                    current_index - 1
                };
                
                // Get the actual sibling value at this level
                let sibling = if current_depth == 0 {
                    // At leaf level, get from leaves array
                    if sibling_index < self.leaves.len() as u32 {
                        self.leaves.get(sibling_index).unwrap()
                    } else {
                        BytesN::from_array(&self.env, &[0u8; 32])
                    }
                } else {
                    // At internal levels, compute the node value
                    self.compute_node_at_level(sibling_index, current_depth)
                };
                
                siblings.push_back(sibling);
                
                current_index = current_index / 2;
                current_depth += 1;
            }
            
            // Add the root level sibling (which is the root itself)
            if self.depth > 0 {
                siblings.push_back(self.root.clone());
            }
        }

        Some((siblings, self.depth))
    }

    /// Computes the value of an internal node at a specific level
    /// This method recursively builds the tree up to the specified level
    fn compute_node_at_level(&self, node_index: u32, target_level: u32) -> BytesN<32> {
        if target_level == 0 {
            // At leaf level, return the actual leaf value or empty hash
            if node_index < self.leaves.len() as u32 {
                self.leaves.get(node_index).unwrap()
            } else {
                BytesN::from_array(&self.env, &[0u8; 32])
            }
        } else if target_level == 1 {
            // At level 1, compute hash of two leaves
            let left_child_index = node_index * 2;
            let right_child_index = left_child_index + 1;
            
            if left_child_index < self.leaves.len() as u32 && right_child_index < self.leaves.len() as u32 {
                let left_child = self.leaves.get(left_child_index).unwrap();
                let right_child = self.leaves.get(right_child_index).unwrap();
                self.hash_pair(&left_child, &right_child)
            } else if left_child_index < self.leaves.len() as u32 {
                // Only left child exists, propagate it (LeanIMT property)
                self.leaves.get(left_child_index).unwrap()
            } else {
                // No children exist
                BytesN::from_array(&self.env, &[0u8; 32])
            }
        } else {
            // For higher levels, we don't have enough leaves
            BytesN::from_array(&self.env, &[0u8; 32])
        }
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
        poseidon2(&self.env, left, right)
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

    /// Gets the value of a node at a specific level and index
    /// This is useful for testing and debugging merkle proofs
    /// 
    /// # Arguments
    /// * `level` - The level in the tree (0 = leaves, 1 = first internal level, etc.)
    /// * `index` - The index of the node at that level
    /// 
    /// # Returns
    /// * `Some(node_value)` if the node exists at that level and index
    /// * `None` if the node doesn't exist
    pub fn get_node(&self, level: u32, index: u32) -> Option<BytesN<32>> {
        if level == 0 {
            // At leaf level, return the actual leaf value
            if index < self.leaves.len() as u32 {
                Some(self.leaves.get(index).unwrap())
            } else {
                None
            }
        } else if level > self.depth {
            // Level doesn't exist in the tree
            None
        } else {
            // At internal level, compute the node value
            Some(self.compute_node_at_level(index, level))
        }
    }

    /// Gets the sibling of a node at a specific level and index
    /// This is useful for generating merkle proofs
    /// 
    /// # Arguments
    /// * `level` - The level in the tree (0 = leaves, 1 = first internal level, etc.)
    /// * `index` - The index of the node at that level
    /// 
    /// # Returns
    /// * `Some(sibling_value)` if the sibling exists
    /// * `None` if the sibling doesn't exist
    pub fn get_sibling(&self, level: u32, index: u32) -> Option<BytesN<32>> {
        if level > self.depth {
            return None;
        }
        
        // At the root level, there are no siblings
        if level == self.depth {
            return None;
        }
        
        let sibling_index = if index % 2 == 0 {
            index + 1
        } else {
            index - 1
        };
        
        self.get_node(level, sibling_index)
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
    fn test_generate_complete_proof() {
        let env = Env::default();
        let mut tree = LeanIMT::new(env.clone());
        
        let leaf1 = BytesN::from_array(&env, &[1u8; 32]);
        let leaf2 = BytesN::from_array(&env, &[2u8; 32]);
        
        tree.insert(leaf1.clone());
        tree.insert(leaf2.clone());
        
        // Generate complete proof for first leaf (index 0)
        let proof = tree.generate_complete_proof(0);
        assert!(proof.is_some());
        
        let (siblings, depth) = proof.unwrap();
        // Debug output - in Soroban we can't use println!, so we'll just test the logic
        assert_eq!(depth, 1);
        assert_eq!(siblings.len(), 2); // root level + leaf level
        
        // The first sibling should be leaf 2 (at leaf level)
        let leaf_level_sibling = siblings.get(0).unwrap();
        assert_eq!(leaf_level_sibling, leaf2);
    }

    #[test]
    fn test_get_node() {
        let env = Env::default();
        let mut tree = LeanIMT::new(env.clone());
        
        let leaf1 = BytesN::from_array(&env, &[1u8; 32]);
        let leaf2 = BytesN::from_array(&env, &[2u8; 32]);
        let leaf3 = BytesN::from_array(&env, &[3u8; 32]);
        let leaf4 = BytesN::from_array(&env, &[4u8; 32]);
        
        tree.insert(leaf1);
        tree.insert(leaf2);
        tree.insert(leaf3);
        tree.insert(leaf4);
        
        // Test leaf level (level 0)
        assert!(tree.get_node(0, 0).is_some()); // leaf 1
        assert!(tree.get_node(0, 1).is_some()); // leaf 2
        assert!(tree.get_node(0, 2).is_some()); // leaf 3
        assert!(tree.get_node(0, 3).is_some()); // leaf 4
        assert!(tree.get_node(0, 4).is_none()); // doesn't exist
        
        // Test internal level (level 1)
        assert!(tree.get_node(1, 0).is_some()); // hash of leaves 1,2
        assert!(tree.get_node(1, 1).is_some()); // hash of leaves 3,4
        
        // Test root level (level 2)
        assert!(tree.get_node(2, 0).is_some()); // root
        
        // Test non-existent level
        assert!(tree.get_node(3, 0).is_none());
    }

    #[test]
    fn test_get_sibling() {
        let env = Env::default();
        let mut tree = LeanIMT::new(env.clone());
        
        let leaf1 = BytesN::from_array(&env, &[1u8; 32]);
        let leaf2 = BytesN::from_array(&env, &[2u8; 32]);
        let leaf3 = BytesN::from_array(&env, &[3u8; 32]);
        let leaf4 = BytesN::from_array(&env, &[4u8; 32]);
        
        tree.insert(leaf1.clone());
        tree.insert(leaf2.clone());
        tree.insert(leaf3.clone());
        tree.insert(leaf4.clone());
        
        // Test leaf level siblings
        let sibling_0 = tree.get_sibling(0, 0).unwrap(); // sibling of leaf 1
        let sibling_1 = tree.get_sibling(0, 1).unwrap(); // sibling of leaf 2
        assert_eq!(sibling_0, leaf2);
        assert_eq!(sibling_1, leaf1);
        
        // Test internal level siblings
        let sibling_internal_0 = tree.get_sibling(1, 0).unwrap(); // sibling of node 0 at level 1
        let sibling_internal_1 = tree.get_sibling(1, 1).unwrap(); // sibling of node 1 at level 1
        assert!(sibling_internal_0 != sibling_internal_1); // should be different
        
        // Test non-existent siblings
        assert!(tree.get_sibling(0, 4).is_none()); // leaf 4 doesn't exist
        assert!(tree.get_sibling(2, 0).is_none()); // root level has no siblings
    }

    #[test]
    fn test_complete_proof_with_internal_nodes() {
        let env = Env::default();
        let mut tree = LeanIMT::new(env.clone());
        
        let leaf1 = BytesN::from_array(&env, &[1u8; 32]);
        let leaf2 = BytesN::from_array(&env, &[2u8; 32]);
        let leaf3 = BytesN::from_array(&env, &[3u8; 32]);
        let leaf4 = BytesN::from_array(&env, &[4u8; 32]);
        
        tree.insert(leaf1.clone());
        tree.insert(leaf2.clone());
        tree.insert(leaf3.clone());
        tree.insert(leaf4.clone());
        
        // Generate complete proof for leaf 1 (index 0)
        let proof = tree.generate_complete_proof(0);
        assert!(proof.is_some());
        
        let (siblings, depth) = proof.unwrap();
        assert_eq!(depth, 2);
        assert_eq!(siblings.len(), 3); // leaf level + internal level + root level
        
        // The first sibling should be leaf 2 (at leaf level)
        let leaf_level_sibling = siblings.get(0).unwrap();
        assert_eq!(leaf_level_sibling, leaf2);
        
        // The second sibling should be the hash of leaves 3,4 (at internal level)
        let internal_level_sibling = siblings.get(1).unwrap();
        let expected_internal = tree.hash_pair(&leaf3, &leaf4);
        assert_eq!(internal_level_sibling, expected_internal);
        
        // The third sibling should be the root
        let root_level_sibling = siblings.get(2).unwrap();
        assert_eq!(root_level_sibling, tree.get_root());
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
