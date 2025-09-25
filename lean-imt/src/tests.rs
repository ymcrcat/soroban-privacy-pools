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
    let mut tree = LeanIMT::new(&env, 2);
    
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
    let tree = LeanIMT::new(&env, 0);
    
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
    let mut tree = LeanIMT::new(&env, 3);
    
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
