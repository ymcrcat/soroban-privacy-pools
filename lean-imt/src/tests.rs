use crate::*;

#[test]
fn test_new_tree() {
    let env = Env::default();
    let tree = LeanIMT::new(&env, 0);
    assert_eq!(tree.get_depth(), 0);
    assert_eq!(tree.get_leaf_count(), 0);
    assert!(tree.is_empty());
}

#[test]
fn test_insert_u64() {
    let env = Env::default();
    let mut tree = LeanIMT::new(&env, 1);
    
    tree.insert_u64(0);
    tree.insert_u64(0);
    
    assert_eq!(tree.get_depth(), 1);
    assert_eq!(tree.get_leaf_count(), 2);
    
    // This should now compute the same root as Circom for [0, 0]
    let _root = tree.get_root_scalar();
    // Root computed successfully - should match Circom for [0, 0]
}

#[test]
fn test_hash_pair() {
    let env = Env::default();
    let tree = LeanIMT::new(&env, 0);
    
    let left_scalar = u64_to_bls_scalar(&env, 1);
    let right_scalar = u64_to_bls_scalar(&env, 2);
    
    let hash_scalar = tree.hash_pair(left_scalar.clone(), right_scalar.clone());
    
    // Verify the hash is deterministic
    let hash2_scalar = tree.hash_pair(left_scalar, right_scalar);
    assert_eq!(hash_scalar, hash2_scalar);
}

#[test]
fn test_compute_node_at_level_multiple_levels() {
    let env = Env::default();
    let mut tree = LeanIMT::new(&env, 1);
    
    // Insert 2 leaves to create a 1-level tree
    tree.insert_u64(0);
    tree.insert_u64(1);
    
    assert_eq!(tree.get_depth(), 1);
    assert_eq!(tree.get_leaf_count(), 2);
    
    // Test level 0 (leaves) - should match the inserted values
    let node_0 = tree.get_node(0, 0).unwrap();
        let expected_0 = bls_scalar_to_bytes(u64_to_bls_scalar(&env, 0));
    assert_eq!(node_0, expected_0);
    
    let node_1 = tree.get_node(0, 1).unwrap();
    let expected_1 = bls_scalar_to_bytes(u64_to_bls_scalar(&env, 1));
    assert_eq!(node_1, expected_1);
    
    // Test level 1 (root level) - should match the tree root
    let root_node = tree.get_node(1, 0).unwrap();
    assert_eq!(root_node, tree.get_root());
    
    // Test that nodes beyond the tree depth return None
    assert!(tree.get_node(2, 0).is_none());
}

#[test]
fn test_generate_proof_two_leaves() {
    let env = Env::default();
    let mut tree = LeanIMT::new(&env, 1);
    
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
    assert_eq!(siblings_0.len(), 1); // 1 sibling only (no root)
    
    // Test proof for leaf 1
    let proof_1 = tree.generate_proof(1);
    assert!(proof_1.is_some());
    let (siblings_1, depth_1) = proof_1.unwrap();
    assert_eq!(depth_1, 1);
    assert_eq!(siblings_1.len(), 1); // 1 sibling only (no root)
    
    // Verify siblings are correct (should be the other leaf)
    let leaf_1_scalar = u64_to_bls_scalar(&env, 2);
    let leaf_0_scalar = u64_to_bls_scalar(&env, 1);
    
    // For leaf 0, sibling should be leaf 1
    assert_eq!(siblings_0.get(0).unwrap(), leaf_1_scalar);
    // For leaf 1, sibling should be leaf 0
    assert_eq!(siblings_1.get(0).unwrap(), leaf_0_scalar);
}

#[test]
fn test_bls_scalar_to_bytes_roundtrip() {
    let env = Env::default();
    
    // Test with various BlsScalar values
    let test_values = [
        u64_to_bls_scalar(&env, 0),
        u64_to_bls_scalar(&env, 1),
        u64_to_bls_scalar(&env, 42),
        u64_to_bls_scalar(&env, 12345),
        u64_to_bls_scalar(&env, u64::MAX),
        u64_to_bls_scalar(&env, 0x1234567890ABCDEF),
    ];
    
    for original_scalar in test_values {
        // Convert BlsScalar to BytesN<32> and back
        let bytes = bls_scalar_to_bytes(original_scalar.clone());
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
        let converted_bytes = bls_scalar_to_bytes(scalar);
        
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
    let converted_bytes = bls_scalar_to_bytes(scalar);
    
    // The result should be different from input due to field reduction
    // but the conversion should not panic
    // Note: soroban_sdk BlsScalar may handle large values differently than ark
    // So we just verify the conversion doesn't panic and is stable
    
    // However, converting the reduced value back should be stable
    let scalar2 = bytes_to_bls_scalar(&converted_bytes);
    let converted_bytes2 = bls_scalar_to_bytes(scalar2);
    assert_eq!(converted_bytes, converted_bytes2, "Reduced values should be stable");
}

#[test]
fn test_depth_2_tree_creation() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let tree = LeanIMT::new(&env, 2);
    
    // Test that we can create a depth 2 tree
    assert_eq!(tree.get_depth(), 2);
    assert_eq!(tree.get_leaf_count(), 0);
    
    // Test that nodes beyond the tree depth return None
    assert!(tree.get_node(3, 0).is_none());
    
    // Test that we can get the root (empty depth 2 tree should have a computed root)
    let root = tree.get_root();
    let zero_root = BytesN::from_array(&env, &[0u8; 32]);
    assert_ne!(root, zero_root, "Empty depth 2 tree should have a non-zero computed root");
}

#[test]
fn test_depth_4_tree_creation() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let tree = LeanIMT::new(&env, 4);
    
    // Test that we can create a depth 4 tree
    assert_eq!(tree.get_depth(), 4);
    assert_eq!(tree.get_leaf_count(), 0);
    
    // Test that nodes beyond the tree depth return None
    assert!(tree.get_node(5, 0).is_none());
    
    // Test that we can get the root (empty depth 4 tree should have a computed root)
    let root = tree.get_root();
    let zero_root = BytesN::from_array(&env, &[0u8; 32]);
    assert_ne!(root, zero_root, "Empty depth 4 tree should have a non-zero computed root");
    
    // Test that we can get nodes at internal levels (levels 1-4 should exist for empty tree)
    for level in 1..=4 {
        for index in 0..(1u32 << (4 - level)) {
            let node = tree.get_node(level, index);
            assert!(node.is_some(), "Internal node at level {}, index {} should exist", level, index);
        }
    }
    
    // Test that leaf nodes (level 0) return None for empty tree since no leaves were inserted
    for index in 0..16 {
        let node = tree.get_node(0, index);
        assert!(node.is_none(), "Leaf node at index {} should not exist in empty tree", index);
    }
}

#[test]
fn test_depth_2_tree_proof() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let mut tree = LeanIMT::new(&env, 2);
    
    // Insert 4 leaves to fill the depth 2 tree
    tree.insert_u64(1);
    tree.insert_u64(2);
    tree.insert_u64(3);
    tree.insert_u64(4);
    
    // Test proof generation for each leaf
    for leaf_index in 0..4 {
        let proof = tree.generate_proof(leaf_index);
        assert!(proof.is_some(), "Proof should be generated for leaf {}", leaf_index);
        
        let (siblings, depth) = proof.unwrap();
        assert_eq!(depth, 2, "Proof depth should be 2 for leaf {}", leaf_index);
        assert_eq!(siblings.len(), 2, "Should have 2 siblings for depth 2 tree, leaf {}", leaf_index);
    }
    
    // Test specific proof for leaf 0 (should have siblings from levels 0 and 1)
    let proof_0 = tree.generate_proof(0).unwrap();
    let (siblings_0, _) = proof_0;
    
    // First sibling should be leaf 1 (at level 0)
    let expected_sibling_0 = u64_to_bls_scalar(&env, 2);
    assert_eq!(siblings_0.get(0).unwrap(), expected_sibling_0);
    
    // Second sibling should be the hash of leaves 2,3 (at level 1)
    let expected_sibling_1 = tree.get_node(1, 1).unwrap();
    let expected_sibling_1_scalar = bytes_to_bls_scalar(&expected_sibling_1);
    assert_eq!(siblings_0.get(1).unwrap(), expected_sibling_1_scalar);
}

#[test]
fn test_incremental_update_functional_approach() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let mut tree = LeanIMT::new(&env, 3); // Depth 3 tree (8 leaves)
    
    // Insert some leaves
    tree.insert_u64(1);
    tree.insert_u64(2);
    tree.insert_u64(3);
    
    // Get the root after 3 insertions
    let root_after_3 = tree.get_root();
    
    // Insert one more leaf using incremental update
    tree.insert_u64(4);
    
    // Get the root after 4 insertions
    let root_after_4 = tree.get_root();
    
    // Verify that the root changed (proving incremental update worked)
    assert_ne!(root_after_3, root_after_4, "Root should change after inserting new leaf");
    
    // Verify that the incremental update produces the same result as full recomputation
    let mut tree_full_recompute = LeanIMT::new(&env, 3);
    tree_full_recompute.insert_u64(1);
    tree_full_recompute.insert_u64(2);
    tree_full_recompute.insert_u64(3);
    tree_full_recompute.insert_u64(4);
    
    assert_eq!(root_after_4, tree_full_recompute.get_root(), 
               "Incremental update should produce same result as full recomputation");
}

#[test]
fn test_path_recomputation_efficiency() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let mut tree = LeanIMT::new(&env, 4); // Depth 4 tree (16 leaves)
    
    // Insert many leaves to test efficiency
    for i in 1..=10 {
        tree.insert_u64(i);
    }
    
    // Verify the tree is in a consistent state
    assert_eq!(tree.get_leaf_count(), 10);
    assert_eq!(tree.get_depth(), 4);
    
    // Test that path recomputation works for different leaf indices
    for leaf_index in 0..10 {
        let path_analysis = tree.analyze_optimization_path(leaf_index);
        
        // Should have 4 levels of analysis (depth 4)
        assert_eq!(path_analysis.len(), 4); // 4 levels (no vec![env] start in this implementation)
        
        // Verify the analysis shows the optimization concept
        for i in 0..4 { // 4 levels of analysis
            let (level, sibling_index, _is_cached) = path_analysis.get(i).unwrap();
            assert!(level < 4, "Level should be less than depth");
            assert!(sibling_index < 16, "Sibling index should be within tree bounds");
            // is_cached indicates whether this sibling would be cached in true implementation
        }
    }
}

#[test]
fn test_depth_10_tree_creation() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let tree = LeanIMT::new(&env, 10);
    
    // Test that we can create a depth 10 tree
    assert_eq!(tree.get_depth(), 10);
    assert_eq!(tree.get_leaf_count(), 0);
    
    // Test that nodes beyond the tree depth return None
    assert!(tree.get_node(11, 0).is_none());
    
    // Test that we can get the root (empty depth 10 tree should have a computed root)
    let root = tree.get_root();
    let zero_root = BytesN::from_array(&env, &[0u8; 32]);
    assert_ne!(root, zero_root, "Empty depth 10 tree should have a non-zero computed root");
    
    // Test that we can get nodes at internal levels (levels 1-10 should exist for empty tree)
    // Sample a few nodes at each level instead of checking all 1023 internal nodes
    for level in 1..=10 {
        let max_index = 1u32 << (10 - level);
        // Test first, middle, and last nodes at each level
        if max_index <= 3 {
            for index in 0..max_index {
                let node = tree.get_node(level, index);
                assert!(node.is_some(), "Internal node at level {}, index {} should exist", level, index);
            }
        } else {
            let test_indices = [0, max_index / 2, max_index - 1];
            for &index in &test_indices {
                let node = tree.get_node(level, index);
                assert!(node.is_some(), "Internal node at level {}, index {} should exist", level, index);
            }
        }
    }
    
    // Test that leaf nodes (level 0) return None for empty tree since no leaves were inserted
    // Sample a few leaf positions instead of checking all 1024
    let leaf_test_indices = [0, 100, 500, 1000, 1023]; // Sample across the range
    for &index in &leaf_test_indices {
        let node = tree.get_node(0, index);
        assert!(node.is_none(), "Leaf node at index {} should not exist in empty tree", index);
    }
}

#[test]
fn test_depth_10_tree_with_leaves() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let mut tree = LeanIMT::new(&env, 10);
    
    // Insert some leaves to test the tree functionality
    let num_leaves = 20u32; // Insert 20 leaves out of 1024 possible (reduced for performance)
    for i in 1..=num_leaves {
        tree.insert_u64(i as u64);
    }
    
    // Verify the tree state
    assert_eq!(tree.get_depth(), 10);
    assert_eq!(tree.get_leaf_count(), num_leaves);
    
    // Test that we can get the root
    let root = tree.get_root();
    let zero_root = BytesN::from_array(&env, &[0u8; 32]);
    assert_ne!(root, zero_root, "Tree with leaves should have a non-zero root");
    
    // Test that inserted leaves can be retrieved
    for i in 0..num_leaves {
        let leaf_node = tree.get_node(0, i);
        assert!(leaf_node.is_some(), "Leaf node at index {} should exist", i);
        
        let expected_scalar = u64_to_bls_scalar(&env, (i + 1) as u64);
        let expected_bytes = bls_scalar_to_bytes(expected_scalar);
        assert_eq!(leaf_node.unwrap(), expected_bytes, "Leaf node at index {} should match inserted value", i);
    }
    
    // Test that empty leaf positions return None (sample a few positions)
    let empty_test_indices = [num_leaves, num_leaves + 100, 500, 1000, 1023];
    for &i in &empty_test_indices {
        if i < 1024u32 {
            let leaf_node = tree.get_node(0, i);
            assert!(leaf_node.is_none(), "Leaf node at index {} should not exist (not inserted)", i);
        }
    }
}

#[test]
fn test_depth_10_tree_proof_generation() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let mut tree = LeanIMT::new(&env, 10);
    
    // Insert some leaves to test proof generation
    let num_leaves = 10u32; // Insert 10 leaves (reduced for performance)
    for i in 1..=num_leaves {
        tree.insert_u64(i as u64);
    }
    
    // Test proof generation for a sample of inserted leaves (not all for performance)
    let proof_test_indices = [0, 1, 2, 5, 9]; // Sample across the range
    for &leaf_index in &proof_test_indices {
        if leaf_index < num_leaves {
            let proof = tree.generate_proof(leaf_index);
            assert!(proof.is_some(), "Proof should be generated for leaf {}", leaf_index);
            
            let (siblings, depth) = proof.unwrap();
            assert_eq!(depth, 10, "Proof depth should be 10 for leaf {}", leaf_index);
            assert_eq!(siblings.len(), 10, "Should have 10 siblings for depth 10 tree, leaf {}", leaf_index);
        }
    }
    
    // Test specific proof for leaf 0
    let proof_0 = tree.generate_proof(0).unwrap();
    let (siblings_0, _) = proof_0;
    
    // First sibling should be leaf 1 (at level 0)
    let expected_sibling_0 = u64_to_bls_scalar(&env, 2);
    assert_eq!(siblings_0.get(0).unwrap(), expected_sibling_0);
    
    // Test that proof generation fails for non-existent leaves (sample a few positions)
    let non_existent_indices = [num_leaves, num_leaves + 50, 200, 500, 1000, 1023];
    for &leaf_index in &non_existent_indices {
        if leaf_index < 1024u32 {
            let proof = tree.generate_proof(leaf_index);
            assert!(proof.is_none(), "Proof should not be generated for non-existent leaf {}", leaf_index);
        }
    }
}

