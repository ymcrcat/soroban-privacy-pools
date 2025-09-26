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
    /// Creates a new LeanIMT with a fixed depth. Missing leaves are assumed zero.
    pub fn new(env: &'a Env, depth: u32) -> Self {
        let mut tree = Self {
            env,
            leaves: vec![env],
            depth,
            root: BytesN::from_array(env, &[0u8; 32]),
        };
        tree.recompute_tree();
        tree
    }

    /// Inserts a new leaf into the tree (appends; missing leaves remain zero)
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

    /// Gets the number of leaves that have been explicitly inserted
    pub fn get_leaf_count(&self) -> u32 {
        self.leaves.len() as u32
    }

    /// Generates a merkle proof for a given leaf index
    pub fn generate_proof(&self, leaf_index: u32) -> Option<(alloc::vec::Vec<BlsScalar>, u32)> {
        if leaf_index >= self.leaves.len() as u32 {
            return None;
        }

        let mut siblings = alloc::vec::Vec::new();
        
        // Handle the simple 2-leaf case correctly
        if self.depth == 1 && self.leaves.len() == 2 {
            if leaf_index == 0 {
                let sibling_bytes = self.leaves.get(1).unwrap();
                siblings.push(bytes_to_bls_scalar(&sibling_bytes));
            } else {
                let sibling_bytes = self.leaves.get(0).unwrap();
                siblings.push(bytes_to_bls_scalar(&sibling_bytes));
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
                
                let sibling_scalar = if current_depth == 0 {
                    // At leaf level, use actual leaves or zero if missing
                    if sibling_index < self.leaves.len() as u32 {
                        let sibling_bytes = self.leaves.get(sibling_index).unwrap();
                        bytes_to_bls_scalar(&sibling_bytes)
                    } else {
                        BlsScalar::from(0u64)
                    }
                } else {
                    // At internal levels, compute the actual node value
                    self.compute_node_at_level_scalar(sibling_index, current_depth)
                };
                
                siblings.push(sibling_scalar);
                current_index = current_index / 2;
                current_depth += 1;
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

    /// Recomputes the entire tree after insertion using fixed depth and zero padding
    fn recompute_tree(&mut self) {
        let target_leaf_count: usize = if self.depth == 0 { 1 } else { 1usize << (self.depth as usize) };

        // Build leaf level with explicit leaves followed by zeros
        let mut current_level_scalars: alloc::vec::Vec<BlsScalar> = alloc::vec::Vec::with_capacity(target_leaf_count);
        for i in 0..target_leaf_count {
            if i < (self.leaves.len() as usize) {
                let leaf_bytes = self.leaves.get(i as u32).unwrap();
                let leaf_scalar = bytes_to_bls_scalar(&leaf_bytes);
                current_level_scalars.push(leaf_scalar);
            } else {
                current_level_scalars.push(BlsScalar::from(0u64));
            }
        }
        
        // Compute up the tree for exactly self.depth levels
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

        // Final root (if depth == 0, it's the single leaf or zero)
        if current_level_scalars.len() > 0 {
            self.root = bls_scalar_to_bytes(&self.env, current_level_scalars[0]);
        } else {
            self.root = BytesN::from_array(&self.env, &[0u8; 32]);
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