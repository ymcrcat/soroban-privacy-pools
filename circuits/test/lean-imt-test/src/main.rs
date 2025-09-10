use lean_imt::{LeanIMT, bls_scalar_to_bytes};
use serde::{Deserialize, Serialize};
use soroban_sdk::Env;
use num_bigint::BigUint;
use ark_ff::PrimeField;

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct MerkleProofResult {
    leaf: String,
    leafIndex: u32,
    siblings: Vec<String>,
    actualDepth: u32,
    root: String,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() >= 6 {
        // Proof mode - compute merkle proof for specific leaf
        let mut leaves = Vec::new();
        for i in 1..5 {
            let leaf_value = args[i].parse().unwrap_or(0);
            leaves.push(leaf_value);
        }
        
        let leaf_index: u32 = args[5].parse().unwrap_or(0);
        
        println!("🧪 Computing Merkle Proof for Leaf Index {}", leaf_index);
        println!("================================================");
        println!("Testing merkle proof generation with lean-imt");
        println!("");
        
        let env = Env::default();
        let proof_result = compute_merkle_proof(&env, &leaves, leaf_index);
        
        println!("Leaf index: {}", proof_result.leafIndex);
        println!("Leaf value: {}", leaves[leaf_index as usize]);
        // Convert siblings to decimal for display
        let siblings_decimal_display: Vec<String> = proof_result.siblings.iter()
            .enumerate()
            .map(|(i, sibling_decimal)| {
                if i == 0 {
                    // First sibling is the other leaf at the same level
                    let sibling_leaf_index = if leaf_index < 2 {
                        if leaf_index == 0 { 1 } else { 0 }
                    } else {
                        if leaf_index == 2 { 3 } else { 2 }
                    };
                    leaves[sibling_leaf_index].to_string()
                } else {
                    // Second sibling is already a decimal string
                    sibling_decimal.to_string()
                }
            })
            .collect();
        println!("Siblings: {:?}", siblings_decimal_display);
        println!("Actual depth: {}", proof_result.actualDepth);
        println!("Merkle root: {}", proof_result.root);
        
        // Save circuit-compatible input with decimal string representations
        let leaf_decimal = leaves[leaf_index as usize].to_string();
        
        // The siblings are already in decimal format, so we can use them directly
        let siblings_decimal = proof_result.siblings.clone();
        
        let circuit_input = CircuitInput {
            leaf: leaf_decimal,
            leafIndex: proof_result.leafIndex,
            siblings: siblings_decimal,
            actualDepth: proof_result.actualDepth,
        };
        let circuit_json = serde_json::to_string_pretty(&circuit_input).unwrap();
        std::fs::write("circuit_input.json", circuit_json).unwrap();
        println!("📁 Circuit input saved to: circuit_input.json");
        
        return;
    }
    
    // Show usage if no valid arguments provided
    println!("🧪 Lean-IMT Test Suite");
    println!("======================");
    println!("Usage:");
    println!("   cargo run -- <leaf1> <leaf2> <leaf3> <leaf4> <leaf_index>");
    println!("\nExample:");
    println!("   cargo run -- 0 0 0 0 0");
}

fn compute_merkle_proof(env: &Env, leaves: &[u64], leaf_index: u32) -> MerkleProofResult {
    // Create a new LeanIMT instance
    let mut tree = LeanIMT::new(env);
    
    for &leaf in leaves {
        tree.insert_u64(leaf);
    }
    
    // Generate the merkle proof for the specified leaf index
    let proof = tree.generate_proof(leaf_index).expect("Failed to generate proof");
    let (siblings, actual_depth) = proof;
    
    // Get the leaf value using the new scalar-based method
    let leaf_scalar = tree.get_leaf_scalar(leaf_index as usize).expect("Leaf not found");
    let leaf_value_decimal = leaf_scalar.to_string();
    
    // Convert siblings to decimal strings, but filter out the root (last element)
    // The circuit should compute the root itself, not receive it as input
    let mut siblings_decimal = Vec::new();
    let siblings_count = siblings.len();
    let siblings_to_process = if siblings_count > actual_depth {
        // Remove the last element (root) if it was included
        actual_depth
    } else {
        siblings_count
    };
    
    for i in 0..siblings_to_process {
        let sibling = siblings.get(i).unwrap();
        if i == 0 {
            // First sibling is the other leaf at the same level
            // For leaf 0: sibling is leaf 1, for leaf 1: sibling is leaf 0
            // For leaf 2: sibling is leaf 3, for leaf 3: sibling is leaf 2
            let sibling_leaf_index = if leaf_index < 2 {
                if leaf_index == 0 { 1 } else { 0 }
            } else {
                if leaf_index == 2 { 3 } else { 2 }
            };
            siblings_decimal.push(leaves[sibling_leaf_index].to_string());
        } else {
            // Second sibling is a hash value - convert bytes directly to decimal
            let decimal_value = BigUint::from_bytes_be(&sibling.to_array());
            siblings_decimal.push(decimal_value.to_string());
        }
    }
    
    // Get the merkle root for display
    let root_scalar = tree.get_root_scalar();
    let root_decimal = root_scalar.to_string();
    
    MerkleProofResult {
        leaf: leaf_value_decimal,
        leafIndex: leaf_index,
        siblings: siblings_decimal,
        actualDepth: actual_depth,
        root: root_decimal,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct CircuitInput {
    leaf: String,
    leafIndex: u32,
    siblings: Vec<String>,
    actualDepth: u32,
}

