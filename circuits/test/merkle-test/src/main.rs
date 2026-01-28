use soroban_poseidon::poseidon_hash;
use soroban_sdk::{
    crypto::bls12_381::Fr as BlsScalar,
    Env, U256, BytesN, Vec
};
use num_bigint::BigUint;

fn bls_scalar_to_decimal(scalar: BlsScalar) -> String {
    let u256_val = scalar.to_u256();
    let bytes = u256_val.to_be_bytes();
    let mut bytes_array = [0u8; 32];
    bytes.copy_into_slice(&mut bytes_array);
    let biguint = BigUint::from_bytes_be(&bytes_array);
    biguint.to_str_radix(10)
}

fn u64_to_bls_scalar(env: &Env, value: u64) -> BlsScalar {
    BlsScalar::from_u256(U256::from_u32(env, value as u32))
}

/// Hash two BlsScalars using Poseidon with t=3
fn hash_pair(env: &Env, left: BlsScalar, right: BlsScalar) -> BlsScalar {
    let left_u256 = BlsScalar::to_u256(&left);
    let right_u256 = BlsScalar::to_u256(&right);
    let inputs = Vec::from_array(env, [left_u256, right_u256]);
    let result_u256 = poseidon_hash::<3, BlsScalar>(env, &inputs);
    BlsScalar::from_u256(result_u256)
}

fn main() {
    let env = Env::default();

    // Test case: depth=2 tree with a single leaf at index 0
    // A depth-2 tree has capacity 2^2 = 4 leaves
    //
    //            root (level 2)
    //           /    \
    //      h(0,1)    h(2,3)   (level 1)
    //       / \       / \
    //     L0  0      0   0    (level 0)
    //
    // When we insert leaf L0 at index 0:
    // - sibling at level 0 is 0 (index 1)
    // - sibling at level 1 is h(0,0) (index 1 at level 1)

    let leaf = u64_to_bls_scalar(&env, 123);
    let zero = u64_to_bls_scalar(&env, 0);

    // Compute h(0,0) - the hash of two zeros (used for empty subtrees)
    let h_zero_zero = hash_pair(&env, zero.clone(), zero.clone());

    // For leaf index 0:
    // - Path bits: index 0 = binary 00, so bits are [0, 0]
    // - At level 0: sibling is at index 1 = 0
    // - At level 1: sibling is at index 1 = h(0,0)

    // Compute root step by step:
    // 1. h(leaf, 0) = h(123, 0)
    let level1_node0 = hash_pair(&env, leaf.clone(), zero.clone());
    // 2. h(h(123,0), h(0,0))
    let root = hash_pair(&env, level1_node0.clone(), h_zero_zero.clone());

    println!("=== Merkle Tree Test (depth=2, single leaf at index 0) ===");
    println!("");
    println!("Leaf value: {}", bls_scalar_to_decimal(leaf.clone()));
    println!("Zero: {}", bls_scalar_to_decimal(zero.clone()));
    println!("h(0,0): {}", bls_scalar_to_decimal(h_zero_zero.clone()));
    println!("");
    println!("Computing root for leaf at index 0:");
    println!("  Level 0 -> 1: h(leaf, 0) = {}", bls_scalar_to_decimal(level1_node0.clone()));
    println!("  Level 1 -> 2: h(h(leaf,0), h(0,0)) = {}", bls_scalar_to_decimal(root.clone()));
    println!("");
    println!("=== Circom Input JSON ===");
    println!("{{");
    println!("  \"leaf\": \"{}\",", bls_scalar_to_decimal(leaf.clone()));
    println!("  \"leafIndex\": \"0\",");
    println!("  \"siblings\": [\"{}\", \"{}\"]",
        bls_scalar_to_decimal(zero.clone()),
        bls_scalar_to_decimal(h_zero_zero.clone())
    );
    println!("}}");
    println!("");
    println!("Expected root: {}", bls_scalar_to_decimal(root.clone()));

    // Also test leaf at index 1
    println!("");
    println!("=== Test leaf at index 1 ===");
    // For leaf index 1:
    // - Path bits: index 1 = binary 01, so bits are [1, 0]
    // - At level 0: sibling is at index 0 = 0 (assuming no leaf there)
    // - At level 1: sibling is at index 1 = h(0,0)
    let level1_node0_from_idx1 = hash_pair(&env, zero.clone(), leaf.clone());
    let root_idx1 = hash_pair(&env, level1_node0_from_idx1.clone(), h_zero_zero.clone());

    println!("Computing root for leaf at index 1:");
    println!("  Level 0 -> 1: h(0, leaf) = {}", bls_scalar_to_decimal(level1_node0_from_idx1.clone()));
    println!("  Level 1 -> 2: h(h(0,leaf), h(0,0)) = {}", bls_scalar_to_decimal(root_idx1.clone()));
    println!("");
    println!("=== Circom Input JSON (index 1) ===");
    println!("{{");
    println!("  \"leaf\": \"{}\",", bls_scalar_to_decimal(leaf.clone()));
    println!("  \"leafIndex\": \"1\",");
    println!("  \"siblings\": [\"{}\", \"{}\"]",
        bls_scalar_to_decimal(zero.clone()),
        bls_scalar_to_decimal(h_zero_zero.clone())
    );
    println!("}}");
    println!("");
    println!("Expected root: {}", bls_scalar_to_decimal(root_idx1));
}
