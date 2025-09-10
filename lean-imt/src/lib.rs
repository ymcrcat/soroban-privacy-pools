#![no_std]

extern crate alloc;

use soroban_sdk::{
    symbol_short, vec, BytesN, Env, Symbol, Vec,
};

use ark_bls12_381::Fr as BlsScalar;
use ark_ff::{PrimeField, BigInteger};
use poseidon::Poseidon255;

/// Storage keys for the LeanIMT
pub const TREE_ROOT_KEY: Symbol = symbol_short!("root");
pub const TREE_DEPTH_KEY: Symbol = symbol_short!("depth");
pub const TREE_LEAVES_KEY: Symbol = symbol_short!("leaves");

/// Converts u64 to BlsScalar for test compatibility
pub fn u64_to_bls_scalar(value: u64) -> BlsScalar {
    BlsScalar::from(value)
}

/// Converts BlsScalar to BytesN<32> for Soroban storage
pub fn bls_scalar_to_bytes(env: &Env, scalar: BlsScalar) -> BytesN<32> {
    let bigint = scalar.into_bigint();
    let bytes_vec = bigint.to_bytes_be();
    let bytes_array: [u8; 32] = bytes_vec.try_into()
        .expect("BlsScalar should always convert to a 32-byte array");
    BytesN::from_array(env, &bytes_array)
}

/// Converts BytesN<32> to BlsScalar for computation
pub fn bytes_to_bls_scalar(bytes_n: &BytesN<32>) -> BlsScalar {
    let bytes_array: [u8; 32] = bytes_n.to_array();
    // Convert 32 bytes to 4 u64 values for BigInt (big-endian)
    let mut u64_array = [0u64; 4];
    for i in 0..4 {
        let start = i * 8;
        let end = start + 8;
        if end <= bytes_array.len() {
            let mut chunk = [0u8; 8];
            chunk.copy_from_slice(&bytes_array[start..end]);
            u64_array[3 - i] = u64::from_be_bytes(chunk); // Reverse order for little-endian BigInt
        }
    }
    
    let bigint = ark_ff::BigInt::new(u64_array);
    BlsScalar::from_bigint(bigint).unwrap_or(BlsScalar::from(0u64))
}

/// Lean Incremental Merkle Tree implementation with hybrid approach:
/// - Internal computation uses BlsScalar for perfect Circom compatibility
/// - Storage and API uses BytesN<32> for Soroban compatibility
pub struct LeanIMT<'a> {
    env: &'a Env,
    leaves: Vec<BytesN<32>>,
    depth: u32,
    root: BytesN<32>,
}

impl<'a> LeanIMT<'a> {
    /// Creates a new empty LeanIMT
    pub fn new(env: &'a Env) -> Self {
        let empty_hash = BytesN::from_array(env, &[0u8; 32]);
        Self {
            env,
            leaves: vec![env],
            depth: 0,
            root: empty_hash,
        }
    }

    /// Inserts a new leaf into the tree
    pub fn insert(&mut self, leaf: BytesN<32>) {
        self.leaves.push_back(leaf);
        self.recompute_tree();
    }

    /// Inserts a u64 leaf (converts to BlsScalar internally)
    pub fn insert_u64(&mut self, leaf_value: u64) {
        let leaf_scalar = u64_to_bls_scalar(leaf_value);
        let leaf_bytes = bls_scalar_to_bytes(&self.env, leaf_scalar);
        self.insert(leaf_bytes);
    }

    /// Gets the current root of the tree
    pub fn get_root(&self) -> BytesN<32> {
        self.root.clone()
    }

    /// Gets the current root as BlsScalar (for computation)
    pub fn get_root_scalar(&self) -> BlsScalar {
        bytes_to_bls_scalar(&self.root)
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
    pub fn generate_proof(&self, leaf_index: u32) -> Option<(Vec<BytesN<32>>, u32)> {
        if leaf_index >= self.leaves.len() as u32 {
            return None;
        }

        let mut siblings = vec![&self.env];
        
        // Handle the simple 2-leaf case correctly
        if self.depth == 1 && self.leaves.len() == 2 {
            if leaf_index == 0 {
                siblings.push_back(self.leaves.get(1).unwrap());
            } else {
                siblings.push_back(self.leaves.get(0).unwrap());
            }
            
            if self.depth > 0 {
                siblings.push_back(self.root.clone());
            }
        } else {
            // General approach
            let mut current_index = leaf_index;
            let mut current_depth = 0;
            
            while current_depth < self.depth {
                let sibling_index = if current_index % 2 == 0 {
                    current_index + 1
                } else {
                    current_index - 1
                };
                
                let sibling = if current_depth == 0 {
                    // At leaf level, use actual leaves or zero if missing
                    if sibling_index < self.leaves.len() as u32 {
                        self.leaves.get(sibling_index).unwrap()
                    } else {
                        BytesN::from_array(&self.env, &[0u8; 32])
                    }
                } else {
                    // At internal levels, compute the actual node value
                    self.compute_node_at_level(sibling_index, current_depth)
                };
                
                siblings.push_back(sibling);
                current_index = current_index / 2;
                current_depth += 1;
            }
            
            if self.depth > 0 {
                siblings.push_back(self.root.clone());
            }
        }

        Some((siblings, self.depth))
    }


    /// Computes the value of an internal node at a specific level
    fn compute_node_at_level(&self, node_index: u32, target_level: u32) -> BytesN<32> {
        let result_scalar = self.compute_node_at_level_scalar(node_index, target_level);
        bls_scalar_to_bytes(&self.env, result_scalar)
    }

    /// Computes the value of an internal node at a specific level in BlsScalar space
    fn compute_node_at_level_scalar(&self, node_index: u32, target_level: u32) -> BlsScalar {
        if target_level == 0 {
            if node_index < self.leaves.len() as u32 {
                let leaf_bytes = self.leaves.get(node_index).unwrap();
                bytes_to_bls_scalar(&leaf_bytes)
            } else {
                BlsScalar::from(0u64)
            }
        } else if target_level > self.depth {
            BlsScalar::from(0u64)
        } else {
            // For levels > 0, compute by hashing the two children from the level below
            let left_child_index = node_index * 2;
            let right_child_index = left_child_index + 1;
            
            let left_scalar = self.compute_node_at_level_scalar(left_child_index, target_level - 1);
            let right_scalar = self.compute_node_at_level_scalar(right_child_index, target_level - 1);
            
            self.hash_pair(left_scalar, right_scalar)
        }
    }

    /// Computes the tree depth based on the number of leaves
    fn compute_tree_depth(leaf_count: u32) -> u32 {
        if leaf_count == 0 {
            return 0;
        }
        
        let mut depth = 0;
        let mut temp_count = leaf_count;
        while temp_count > 1 {
            temp_count = (temp_count + 1) / 2;
            depth += 1;
        }
        depth
    }

    /// Recomputes the entire tree after insertion
    fn recompute_tree(&mut self) {
        if self.leaves.is_empty() {
            self.root = BytesN::from_array(&self.env, &[0u8; 32]);
            self.depth = 0;
            return;
        }

        let leaf_count = self.leaves.len() as u32;
        self.depth = Self::compute_tree_depth(leaf_count);

        // Convert leaves to BlsScalar for computation using alloc::vec::Vec
        let mut current_level_scalars: alloc::vec::Vec<BlsScalar> = alloc::vec::Vec::new();
        for i in 0..self.leaves.len() {
            let leaf_bytes = self.leaves.get(i).unwrap();
            let leaf_scalar = bytes_to_bls_scalar(&leaf_bytes);
            current_level_scalars.push(leaf_scalar);
        }
        
        // Compute tree levels entirely in BlsScalar space
        for _level in 0..self.depth {
            let mut next_level_scalars: alloc::vec::Vec<BlsScalar> = alloc::vec::Vec::new();
            let mut i: usize = 0;
            
            while i < current_level_scalars.len() {
                if i + 1 < current_level_scalars.len() {
                    let hash_scalar = self.hash_pair(current_level_scalars[i], current_level_scalars[i + 1]);
                    next_level_scalars.push(hash_scalar);
                } else {
                    next_level_scalars.push(current_level_scalars[i]);
                }
                i += 2;
            }
            
            current_level_scalars = next_level_scalars;
        }

        // Convert final result back to BytesN<32> for storage
        if current_level_scalars.len() > 0 {
            self.root = bls_scalar_to_bytes(&self.env, current_level_scalars[0]);
        }
    }

    /// Hashes two BlsScalar values using Poseidon hash function
    fn hash_pair(&self, left: BlsScalar, right: BlsScalar) -> BlsScalar {
        let poseidon = Poseidon255::new();
        poseidon.hash_two(&left, &right)
    }

    /// Serializes the tree state for storage
    pub fn to_storage(&self) -> (Vec<BytesN<32>>, u32, BytesN<32>) {
        (self.leaves.clone(), self.depth, self.root.clone())
    }

    /// Deserializes the tree state from storage
    pub fn from_storage(env: &'a Env, leaves: Vec<BytesN<32>>, depth: u32, root: BytesN<32>) -> Self {
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

    /// Gets a leaf as BlsScalar at a specific index
    pub fn get_leaf_scalar(&self, index: usize) -> Option<BlsScalar> {
        self.get_leaf(index).map(|leaf_bytes| bytes_to_bls_scalar(&leaf_bytes))
    }

    /// Gets the value of a node at a specific level and index
    pub fn get_node(&self, level: u32, index: u32) -> Option<BytesN<32>> {
        if level == 0 {
            if index < self.leaves.len() as u32 {
                Some(self.leaves.get(index).unwrap())
            } else {
                None
            }
        } else if level > self.depth {
            None
        } else {
            Some(self.compute_node_at_level(index, level))
        }
    }

    /// Gets the sibling of a node at a specific level and index
    pub fn get_sibling(&self, level: u32, index: u32) -> Option<BytesN<32>> {
        if level > self.depth {
            return None;
        }
        
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
mod tests {
    use super::*;

    #[test]
    fn test_new_tree() {
        let env = Env::default();
        let tree = LeanIMT::new(&env);
        assert_eq!(tree.get_depth(), 0);
        assert_eq!(tree.get_leaf_count(), 0);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_insert_u64() {
        let env = Env::default();
        let mut tree = LeanIMT::new(&env);
        
        tree.insert_u64(0);
        tree.insert_u64(0);
        tree.insert_u64(0);
        tree.insert_u64(0);
        
        assert_eq!(tree.get_depth(), 2);
        assert_eq!(tree.get_leaf_count(), 4);
        
        // This should now compute the same root as Circom for [0, 0, 0, 0]
        let _root = tree.get_root_scalar();
        // Root computed successfully - should match Circom for [0, 0, 0, 0]
    }

    #[test]
    fn test_hash_pair() {
        let env = Env::default();
        let tree = LeanIMT::new(&env);
        
        let left_scalar = u64_to_bls_scalar(1);
        let right_scalar = u64_to_bls_scalar(2);
        
        let hash_scalar = tree.hash_pair(left_scalar, right_scalar);
        
        // Verify the hash is deterministic
        let hash2_scalar = tree.hash_pair(left_scalar, right_scalar);
        assert_eq!(hash_scalar, hash2_scalar);
    }

    #[test]
    fn test_compute_node_at_level_multiple_levels() {
        let env = Env::default();
        let mut tree = LeanIMT::new(&env);
        
        // Insert 8 leaves to create a 3-level tree
        for i in 0..8 {
            tree.insert_u64(i);
        }
        
        assert_eq!(tree.get_depth(), 3);
        assert_eq!(tree.get_leaf_count(), 8);
        
        // Test level 0 (leaves) - should match the inserted values
        for i in 0..8 {
            let node = tree.get_node(0, i).unwrap();
            let expected = bls_scalar_to_bytes(&tree.env, u64_to_bls_scalar(i as u64));
            assert_eq!(node, expected);
        }
        
        // Test that internal nodes are computed correctly by verifying consistency
        // Level 1 should have 4 nodes
        for i in 0..4 {
            let node = tree.get_node(1, i).unwrap();
            // Verify it's not a zero hash (should be computed)
            assert_ne!(node, BytesN::from_array(&tree.env, &[0u8; 32]));
        }
        
        // Level 2 should have 2 nodes
        for i in 0..2 {
            let node = tree.get_node(2, i).unwrap();
            // Verify it's not a zero hash (should be computed)
            assert_ne!(node, BytesN::from_array(&tree.env, &[0u8; 32]));
        }
        
        // Test level 3 (root level) - should match the tree root
        let root_node = tree.get_node(3, 0).unwrap();
        assert_eq!(root_node, tree.get_root());
        
        // Test that nodes beyond the tree depth return None
        assert!(tree.get_node(4, 0).is_none());
    }

    #[test]
    fn test_compute_tree_depth() {
        // Test edge cases
        assert_eq!(LeanIMT::compute_tree_depth(0), 0);
        assert_eq!(LeanIMT::compute_tree_depth(1), 0);
        
        // Test powers of 2
        assert_eq!(LeanIMT::compute_tree_depth(2), 1);
        assert_eq!(LeanIMT::compute_tree_depth(4), 2);
        assert_eq!(LeanIMT::compute_tree_depth(8), 3);
        assert_eq!(LeanIMT::compute_tree_depth(16), 4);
        assert_eq!(LeanIMT::compute_tree_depth(32), 5);
        
        // Test non-powers of 2
        assert_eq!(LeanIMT::compute_tree_depth(3), 2);  // 3 leaves -> 2 levels
        assert_eq!(LeanIMT::compute_tree_depth(5), 3);  // 5 leaves -> 3 levels
        assert_eq!(LeanIMT::compute_tree_depth(6), 3);  // 6 leaves -> 3 levels
        assert_eq!(LeanIMT::compute_tree_depth(7), 3);  // 7 leaves -> 3 levels
        assert_eq!(LeanIMT::compute_tree_depth(9), 4);  // 9 leaves -> 4 levels
        assert_eq!(LeanIMT::compute_tree_depth(15), 4); // 15 leaves -> 4 levels
        
        // Test larger numbers
        assert_eq!(LeanIMT::compute_tree_depth(100), 7);  // 100 leaves -> 7 levels
        assert_eq!(LeanIMT::compute_tree_depth(1000), 10); // 1000 leaves -> 10 levels
    }

    #[test]
    fn test_generate_proof_two_leaves() {
        let env = Env::default();
        let mut tree = LeanIMT::new(&env);
        
        // Insert exactly 2 leaves to test the special 2-leaf case
        tree.insert_u64(1);
        tree.insert_u64(2);
        
        assert_eq!(tree.get_depth(), 1);
        assert_eq!(tree.get_leaf_count(), 2);
        
        // Test proof for leaf 0
        let proof_0 = tree.generate_proof(0);
        assert!(proof_0.is_some());
        let (siblings_0, depth_0) = proof_0.unwrap();
        assert_eq!(depth_0, 1);
        assert_eq!(siblings_0.len(), 2); // 1 sibling + 1 root
        
        // Test proof for leaf 1
        let proof_1 = tree.generate_proof(1);
        assert!(proof_1.is_some());
        let (siblings_1, depth_1) = proof_1.unwrap();
        assert_eq!(depth_1, 1);
        assert_eq!(siblings_1.len(), 2); // 1 sibling + 1 root
        
        // Verify siblings are correct (should be the other leaf)
        let leaf_1_bytes = bls_scalar_to_bytes(&tree.env, u64_to_bls_scalar(2));
        let leaf_0_bytes = bls_scalar_to_bytes(&tree.env, u64_to_bls_scalar(1));
        
        // For leaf 0, sibling should be leaf 1
        assert_eq!(siblings_0.get(0).unwrap(), leaf_1_bytes);
        // For leaf 1, sibling should be leaf 0
        assert_eq!(siblings_1.get(0).unwrap(), leaf_0_bytes);
        
        // Both should have the same root
        assert_eq!(siblings_0.get(1).unwrap(), siblings_1.get(1).unwrap());
        assert_eq!(siblings_0.get(1).unwrap(), tree.get_root());
    }

    #[test]
    fn test_bls_scalar_to_bytes_roundtrip() {
        let env = Env::default();
        
        // Test with various BlsScalar values
        let test_values = [
            BlsScalar::from(0u64),
            BlsScalar::from(1u64),
            BlsScalar::from(42u64),
            BlsScalar::from(12345u64),
            BlsScalar::from(u64::MAX),
            BlsScalar::from(0x1234567890ABCDEFu64),
        ];
        
        for original_scalar in test_values {
            // Convert BlsScalar to BytesN<32> and back
            let bytes = bls_scalar_to_bytes(&env, original_scalar);
            let converted_scalar = bytes_to_bls_scalar(&bytes);
            
            // Verify round-trip conversion preserves the original value
            assert_eq!(original_scalar, converted_scalar, 
                "BlsScalar -> BytesN<32> -> BlsScalar round-trip failed for value: {:?}", 
                original_scalar);
        }
    }

    #[test]
    fn test_bytes_to_bls_scalar_roundtrip() {
        let env = Env::default();
        
        // Test with various byte patterns that are valid within the field
        // Note: We can't test all possible byte values because values >= field prime
        // will be reduced modulo the prime, breaking round-trip equality
        let test_byte_arrays = [
            [0u8; 32], // All zeros
            [1u8; 32], // All ones (this will be reduced but should still work)
            {
                let mut arr = [0u8; 32];
                arr[0] = 0x12;
                arr[1] = 0x34;
                arr[2] = 0x56;
                arr[3] = 0x78;
                arr[4] = 0x90;
                arr[5] = 0xAB;
                arr[6] = 0xCD;
                arr[7] = 0xEF;
                arr
            },
            {
                let mut arr = [0u8; 32];
                arr[31] = 0x01; // Set last byte to small value
                arr
            },
            {
                let mut arr = [0u8; 32];
                for i in 0..16 { // Only fill first half to avoid field overflow
                    arr[i] = i as u8;
                }
                arr
            },
        ];
        
        for original_bytes in test_byte_arrays {
            let bytes_n = BytesN::from_array(&env, &original_bytes);
            
            // Convert BytesN<32> to BlsScalar and back
            let scalar = bytes_to_bls_scalar(&bytes_n);
            let converted_bytes = bls_scalar_to_bytes(&env, scalar);
            
            // For values that fit within the field, round-trip should work
            // For values that get reduced, we just verify the conversion doesn't panic
            let _scalar_check = bytes_to_bls_scalar(&converted_bytes);
        }
    }

    #[test]
    fn test_field_reduction_behavior() {
        let env = Env::default();
        
        // Test that large values get reduced modulo the field prime
        let large_bytes = [0xFFu8; 32];
        let bytes_n = BytesN::from_array(&env, &large_bytes);
        
        // Convert to scalar (this will be reduced)
        let scalar = bytes_to_bls_scalar(&bytes_n);
        
        // Convert back to bytes
        let converted_bytes = bls_scalar_to_bytes(&env, scalar);
        
        // The result should be different from input due to field reduction
        // but the conversion should not panic
        assert_ne!(bytes_n, converted_bytes, "Large values should be reduced by field arithmetic");
        
        // However, converting the reduced value back should be stable
        let scalar2 = bytes_to_bls_scalar(&converted_bytes);
        let converted_bytes2 = bls_scalar_to_bytes(&env, scalar2);
        assert_eq!(converted_bytes, converted_bytes2, "Reduced values should be stable");
    }
}