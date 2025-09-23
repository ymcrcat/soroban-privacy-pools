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
mod tests;