#![no_std]

use soroban_sdk::{
    symbol_short, vec, BytesN, Env, Symbol, Vec, U256,
    crypto::bls12_381::Fr as BlsScalar,
};
use poseidon::Poseidon255;

/// Storage keys for the LeanIMT
pub const TREE_ROOT_KEY: Symbol = symbol_short!("root");
pub const TREE_DEPTH_KEY: Symbol = symbol_short!("depth");
pub const TREE_LEAVES_KEY: Symbol = symbol_short!("leaves");

/// Converts u64 to BlsScalar for test compatibility
pub fn u64_to_bls_scalar(env: &Env, value: u64) -> BlsScalar {
    BlsScalar::from_u256(U256::from_u32(env, value as u32))
}

/// Converts BlsScalar to BytesN<32> for Soroban storage
pub fn bls_scalar_to_bytes(scalar: BlsScalar) -> BytesN<32> {
    scalar.to_bytes()
}

/// Converts BytesN<32> to BlsScalar for computation
pub fn bytes_to_bls_scalar(bytes_n: &BytesN<32>) -> BlsScalar {
    BlsScalar::from_bytes(bytes_n.clone())
}

/// Lean Incremental Merkle Tree implementation with hybrid approach:
/// - Internal computation uses BlsScalar for perfect Circom compatibility
/// - Storage and API uses BytesN<32> for Soroban compatibility
pub struct LeanIMT<'a> {
    env: &'a Env,
    leaves: Vec<BytesN<32>>,
    depth: u32,
    root: BytesN<32>,
    poseidon: Poseidon255<'a>,
}

impl<'a> LeanIMT<'a> {
    /// Creates a new LeanIMT with a fixed depth. Missing leaves are assumed zero.
    pub fn new(env: &'a Env, depth: u32) -> Self {
        let mut tree = Self {
            env,
            leaves: vec![env],
            depth,
            root: BytesN::from_array(env, &[0u8; 32]),
            poseidon: Poseidon255::new(env),
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
        let leaf_scalar = u64_to_bls_scalar(self.env, leaf_value);
        let leaf_bytes = bls_scalar_to_bytes(leaf_scalar);
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
    pub fn generate_proof(&self, leaf_index: u32) -> Option<(Vec<BlsScalar>, u32)> {
        if leaf_index >= self.leaves.len() as u32 {
            return None;
        }

        let mut siblings = vec![self.env];
        
        // Handle the simple 2-leaf case correctly
        if self.depth == 1 && self.leaves.len() == 2 {
            if leaf_index == 0 {
                let sibling_bytes = self.leaves.get(1).unwrap();
                siblings.push_back(bytes_to_bls_scalar(&sibling_bytes));
            } else {
                let sibling_bytes = self.leaves.get(0).unwrap();
                siblings.push_back(bytes_to_bls_scalar(&sibling_bytes));
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
                        BlsScalar::from_u256(U256::from_u32(self.env, 0))
                    }
                } else {
                    // At internal levels, compute the actual node value
                    self.compute_node_at_level_scalar(sibling_index, current_depth)
                };
                
                siblings.push_back(sibling_scalar);
                current_index = current_index / 2;
                current_depth += 1;
            }
        }

        Some((siblings, self.depth))
    }

    /// Computes the value of an internal node at a specific level
    fn compute_node_at_level(&self, node_index: u32, target_level: u32) -> BytesN<32> {
        let result_scalar = self.compute_node_at_level_scalar(node_index, target_level);
        bls_scalar_to_bytes(result_scalar)
    }

    /// Computes the value of an internal node at a specific level in BlsScalar space
    fn compute_node_at_level_scalar(&self, node_index: u32, target_level: u32) -> BlsScalar {
        if target_level == 0 {
            if node_index < self.leaves.len() as u32 {
                let leaf_bytes = self.leaves.get(node_index).unwrap();
                bytes_to_bls_scalar(&leaf_bytes)
            } else {
                BlsScalar::from_u256(U256::from_u32(self.env, 0))
            }
        } else if target_level > self.depth {
            BlsScalar::from_u256(U256::from_u32(self.env, 0))
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
        let mut current_level_scalars = vec![self.env];
        for i in 0..target_leaf_count {
            if i < (self.leaves.len() as usize) {
                let leaf_bytes = self.leaves.get(i as u32).unwrap();
                let leaf_scalar = bytes_to_bls_scalar(&leaf_bytes);
                current_level_scalars.push_back(leaf_scalar);
            } else {
                current_level_scalars.push_back(BlsScalar::from_u256(U256::from_u32(self.env, 0)));
            }
        }
        
        // Compute up the tree for exactly self.depth levels
        for _level in 0..self.depth {
            let mut next_level_scalars = vec![self.env];
            let mut i: usize = 0;
            
            while i < current_level_scalars.len() as usize {
                if i + 1 < current_level_scalars.len() as usize {
                    let hash_scalar = self.hash_pair(current_level_scalars.get(i as u32).unwrap(), current_level_scalars.get((i + 1) as u32).unwrap());
                    next_level_scalars.push_back(hash_scalar);
                } else {
                    next_level_scalars.push_back(current_level_scalars.get(i as u32).unwrap());
                }
                i += 2;
            }
            
            current_level_scalars = next_level_scalars;
        }

        // Final root (if depth == 0, it's the single leaf or zero)
        if current_level_scalars.len() > 0 {
            self.root = bls_scalar_to_bytes(current_level_scalars.get(0).unwrap());
        } else {
            self.root = BytesN::from_array(self.env, &[0u8; 32]);
        }
    }

    /// Hashes two BlsScalar values using Poseidon hash function
    fn hash_pair(&self, left: BlsScalar, right: BlsScalar) -> BlsScalar {
        self.poseidon.hash_two(&left, &right)
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
            poseidon: Poseidon255::new(env),
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