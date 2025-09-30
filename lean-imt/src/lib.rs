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
    // Memoization cache for all computed subtrees
    // Each level contains a map of node_index -> computed_hash
    // Using a flat structure: level * max_nodes_per_level + node_index -> hash
    subtree_cache: Vec<Option<BlsScalar>>,
}

impl<'a> LeanIMT<'a> {
    /// Creates a new LeanIMT with a fixed depth. Missing leaves are assumed zero.
    pub fn new(env: &'a Env, depth: u32) -> Self {
        let mut tree = Self {
            env,
            leaves: vec![env],
            depth,
            root: BytesN::from_array(env, &[0u8; 32]),
            poseidon: Poseidon255::new_with_t(env, 3),
            subtree_cache: vec![env],
        };
        tree.initialize_cache();
        tree.recompute_tree();
        tree
    }

    /// Inserts a new leaf into the tree (appends; missing leaves remain zero)
    /// Uses incremental path recomputation for efficiency (Clever shortcut 2)
    pub fn insert(&mut self, leaf: BytesN<32>) {
        self.leaves.push_back(leaf);
        self.incremental_update();
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
    /// Now uses memoization cache for efficiency
    fn compute_node_at_level_scalar(&self, node_index: u32, target_level: u32) -> BlsScalar {
        if target_level > self.depth {
            return BlsScalar::from_u256(U256::from_u32(self.env, 0));
        }
        
        // Check if we have this value cached
        let cache_index = self.get_cache_index(target_level, node_index);
        if let Some(cached_value) = self.subtree_cache.get(cache_index).unwrap() {
            return cached_value;
        }
        
        // If not cached, compute it
        if target_level == 0 {
            if node_index < self.leaves.len() as u32 {
                let leaf_bytes = self.leaves.get(node_index).unwrap();
                bytes_to_bls_scalar(&leaf_bytes)
            } else {
                BlsScalar::from_u256(U256::from_u32(self.env, 0))
            }
        } else {
            // For levels > 0, compute by hashing the two children from the level below
            let left_child_index = node_index * 2;
            let right_child_index = left_child_index + 1;
            
            let left_scalar = self.compute_node_at_level_scalar(left_child_index, target_level - 1);
            let right_scalar = self.compute_node_at_level_scalar(right_child_index, target_level - 1);
            
            self.hash_pair(left_scalar, right_scalar)
        }
    }

    /// Incremental update using path recomputation (Clever shortcut 2)
    /// Only recomputes the path from the new leaf to the root
    /// 
    /// This implements the optimization described in Tornado Cash:
    /// "all subtrees to the left of the newest member consist of subtrees 
    /// whose roots can be cached rather than recalculated"
    /// 
    /// Now with full memoization - we only recompute the specific path from the new leaf to root,
    /// and update the cache as we go.
    fn incremental_update(&mut self) {
        let leaf_index = (self.leaves.len() - 1) as u32;
        
        // Update the leaf in the cache
        let leaf_bytes = self.leaves.get(leaf_index).unwrap();
        let leaf_scalar = bytes_to_bls_scalar(&leaf_bytes);
        let cache_index = self.get_cache_index(0, leaf_index);
        self.subtree_cache.set(cache_index, Some(leaf_scalar));
        
        // Recompute the path to root and update cache
        self.root = self.recompute_path_to_root_with_cache_update(leaf_index);
    }


    /// Recomputes only the path from a specific leaf to the root with cache updates
    /// This is the optimized version that updates the cache as it goes
    fn recompute_path_to_root_with_cache_update(&mut self, leaf_index: u32) -> BytesN<32> {
        let leaf_bytes = self.leaves.get(leaf_index).unwrap();
        let leaf_scalar = bytes_to_bls_scalar(&leaf_bytes);
        
        // Start from the leaf and work our way up to the root
        let mut current_index = leaf_index;
        let mut current_level = 0;
        let mut current_scalar = leaf_scalar;
        
        while current_level < self.depth {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };
            
            // Get the sibling value (either from cache or compute if missing)
            let sibling_scalar = if current_level == 0 {
                // At leaf level, use actual leaves or zero if missing
                if sibling_index < self.leaves.len() as u32 {
                    let sibling_bytes = self.leaves.get(sibling_index).unwrap();
                    bytes_to_bls_scalar(&sibling_bytes)
                } else {
                    BlsScalar::from_u256(U256::from_u32(self.env, 0))
                }
            } else {
                // At internal levels, check cache first, then compute if needed
                let sibling_cache_index = self.get_cache_index(current_level, sibling_index);
                if let Some(cached_value) = self.subtree_cache.get(sibling_cache_index).unwrap() {
                    cached_value
                } else {
                    self.compute_node_at_level_scalar(sibling_index, current_level)
                }
            };
            
            // Compute the parent hash
            let parent_scalar = if current_index % 2 == 0 {
                self.hash_pair(current_scalar, sibling_scalar)
            } else {
                self.hash_pair(sibling_scalar, current_scalar)
            };
            
            // Cache the parent hash
            let parent_index = current_index / 2;
            let parent_level = current_level + 1;
            let parent_cache_index = self.get_cache_index(parent_level, parent_index);
            self.subtree_cache.set(parent_cache_index, Some(parent_scalar.clone()));
            
            // Move up to the parent level
            current_index = parent_index;
            current_level = parent_level;
            current_scalar = parent_scalar;
        }
        
        // Return the root
        bls_scalar_to_bytes(current_scalar)
    }

    /// Initializes the subtree cache for all levels
    fn initialize_cache(&mut self) {
        // Calculate total cache size needed
        let mut total_size = 0;
        for level in 0..=self.depth {
            let node_count = if level == 0 {
                if self.depth == 0 { 1 } else { 1usize << (self.depth as usize) }
            } else {
                1usize << ((self.depth - level) as usize)
            };
            total_size += node_count;
        }
        
        // Initialize flat cache with None values
        self.subtree_cache = vec![self.env];
        for _ in 0..total_size {
            self.subtree_cache.push_back(None);
        }
    }

    /// Gets the cache index for a given level and node index
    fn get_cache_index(&self, level: u32, node_index: u32) -> u32 {
        let mut index: u32 = 0;
        for l in 0..level {
            let node_count = if l == 0 {
                if self.depth == 0 { 1 } else { 1usize << (self.depth as usize) }
            } else {
                1usize << ((self.depth - l) as usize)
            };
            index += node_count as u32;
        }
        index + node_index
    }

    /// Recomputes the entire tree after insertion using fixed depth and zero padding
    /// Now with full memoization - all subtrees are cached as they're computed
    fn recompute_tree(&mut self) {
        let target_leaf_count: usize = if self.depth == 0 { 1 } else { 1usize << (self.depth as usize) };

        // Initialize level 0 cache with leaves and zeros
        for i in 0..target_leaf_count {
            let leaf_scalar = if i < (self.leaves.len() as usize) {
                let leaf_bytes = self.leaves.get(i as u32).unwrap();
                bytes_to_bls_scalar(&leaf_bytes)
            } else {
                BlsScalar::from_u256(U256::from_u32(self.env, 0))
            };
            let cache_index = self.get_cache_index(0, i as u32);
            self.subtree_cache.set(cache_index, Some(leaf_scalar));
        }
        
        // Compute up the tree for exactly self.depth levels using memoization
        for level in 1..=self.depth {
            let parent_count = 1usize << ((self.depth - level) as usize);
            
            for parent_index in 0..parent_count {
                let left_child_index = parent_index * 2;
                let right_child_index = left_child_index + 1;
                
                // Get cached values from the level below
                let left_cache_index = self.get_cache_index(level - 1, left_child_index as u32);
                let right_cache_index = self.get_cache_index(level - 1, right_child_index as u32);
                let left_scalar = self.subtree_cache.get(left_cache_index).unwrap().unwrap();
                let right_scalar = self.subtree_cache.get(right_cache_index).unwrap().unwrap();
                
                // Compute and cache the parent hash
                let parent_hash = self.hash_pair(left_scalar, right_scalar);
                let parent_cache_index = self.get_cache_index(level, parent_index as u32);
                self.subtree_cache.set(parent_cache_index, Some(parent_hash));
            }
        }

        // Set the root from the top level cache
        if self.depth == 0 {
            let root_cache_index = self.get_cache_index(0, 0);
            self.root = bls_scalar_to_bytes(self.subtree_cache.get(root_cache_index).unwrap().unwrap());
        } else {
            let root_cache_index = self.get_cache_index(self.depth, 0);
            self.root = bls_scalar_to_bytes(self.subtree_cache.get(root_cache_index).unwrap().unwrap());
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
        let mut tree = Self {
            env,
            leaves,
            depth,
            root,
            poseidon: Poseidon255::new_with_t(env, 3),
            subtree_cache: vec![env],
        };
        tree.initialize_cache();
        tree
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

    /// Demonstrates the "Clever shortcut 2" optimization concept
    /// Shows which subtrees would be reused vs recomputed for a new leaf
    /// 
    /// This method analyzes the path from a new leaf to the root and identifies
    /// which sibling subtrees could be cached (left of current position) vs
    /// which need to be computed (right of current position).
    pub fn analyze_optimization_path(&self, new_leaf_index: u32) -> Vec<(u32, u32, bool)> {
        let mut path_analysis = vec![self.env];
        let mut current_index = new_leaf_index;
        let mut current_level = 0;
        
        while current_level < self.depth {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };
            
            // Determine if this sibling subtree would be cached (left of current position)
            // In the true "Clever shortcut 2", subtrees to the left are cached
            let is_cached = sibling_index < current_index;
            
            path_analysis.push_back((current_level, sibling_index, is_cached));
            
            current_index = current_index / 2;
            current_level += 1;
        }
        
        path_analysis
    }
}

#[cfg(test)]
mod tests;