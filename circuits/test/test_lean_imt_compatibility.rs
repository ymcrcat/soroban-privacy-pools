use std::fs;
use serde_json::{json, Value};
use soroban_sdk::{Env, BytesN};
use lean_imt::LeanIMT;

fn main() {
    println!("🧪 Testing lean-imt ↔ merkleProof.circom compatibility...\n");
    
    test_lean_imt_compatibility();
}

fn test_lean_imt_compatibility() {
    println!("🔍 Building merkle tree using lean-imt crate...");
    
    let env = Env::default();
    let mut tree = LeanIMT::new(env.clone());
    
    // Create test leaves
    let leaf1 = BytesN::from_array(&env, &[1u8; 32]);
    let leaf2 = BytesN::from_array(&env, &[2u8; 32]);
    let leaf3 = BytesN::from_array(&env, &[3u8; 32]);
    let leaf4 = BytesN::from_array(&env, &[4u8; 32]);
    
    // Build the tree
    tree.insert(leaf1);
    tree.insert(leaf2);
    tree.insert(leaf3);
    tree.insert(leaf4);
    
    println!("✅ Tree built successfully:");
    println!("   - Depth: {}", tree.get_depth());
    println!("   - Leaf count: {}", tree.get_leaf_count());
    println!("   - Root: {}", bytes_to_hex(&tree.get_root()));
    
    // Generate test cases for each leaf
    let test_cases = vec![0, 1, 2, 3];
    
    for leaf_index in test_cases {
        println!("\n📋 Testing leaf index {}:", leaf_index);
        
        // Generate proof using lean-imt
        let proof = tree.generate_proof(leaf_index).unwrap();
        let (siblings, actual_depth) = proof;
        
        // Get the leaf
        let leaf = tree.get_leaf(leaf_index as usize).unwrap();
        
        // Create circuit input
        let circuit_input = create_circuit_input(leaf_index, &leaf, &siblings, actual_depth);
        
        // Save input JSON for witness generation
        let input_file = format!("test_input_leaf_{}.json", leaf_index);
        save_circuit_input(&circuit_input, &input_file);
        
        println!("   - Leaf: {}", bytes_to_hex(&leaf));
        println!("   - Siblings count: {}", siblings.len());
        println!("   - Actual depth: {}", actual_depth);
        println!("   - Expected siblings: {}", actual_depth + 1);
        
        // Verify proof structure
        if siblings.len() != (actual_depth + 1) as u32 {
            panic!("Invalid proof structure: expected {} siblings, got {}", actual_depth + 1, siblings.len());
        }
        
        println!("   ✅ Proof structure valid");
        println!("   💾 Input JSON saved to {}", input_file);
    }
    
    println!("\n🎉 All compatibility tests passed!");
    println!("\n📝 Next steps:");
    println!("1. Compile the merkleProof.circom circuit");
    println!("2. Use the generated JSON files to generate witnesses");
    println!("3. Verify that witnesses are valid");
}

fn create_circuit_input(leaf_index: u32, leaf: &BytesN<32>, siblings: &soroban_sdk::Vec<BytesN<32>>, actual_depth: u32) -> Value {
    // Convert leaf to hex
    let leaf_hex = bytes_to_hex(leaf);
    
    // Convert siblings to hex and pad to maxDepth=10
    let mut siblings_hex: std::vec::Vec<String> = siblings.iter()
        .map(|s| bytes_to_hex(&s))
        .collect();
    
    // Pad with zeros to match the circuit's maxDepth=10
    let max_depth = 10;
    while siblings_hex.len() < max_depth {
        siblings_hex.push("0x0000000000000000000000000000000000000000000000000000000000000000".to_string());
    }
    
    json!({
        "leaf": leaf_hex,
        "leafIndex": leaf_index,
        "siblings": siblings_hex,
        "actualDepth": actual_depth
    })
}

fn save_circuit_input(input: &Value, filename: &str) {
    let json_string = serde_json::to_string_pretty(input).unwrap();
    fs::write(filename, json_string).expect("Failed to write JSON file");
}

fn bytes_to_hex(bytes: &BytesN<32>) -> String {
    let byte_array = bytes.to_array();
    format!("0x{}", byte_array.iter().map(|b| format!("{:02x}", b)).collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lean_imt_integration() {
        let env = Env::default();
        let mut tree = LeanIMT::new(env.clone());
        
        // Insert a few leaves
        let leaf1 = BytesN::from_array(&env, &[1u8; 32]);
        let leaf2 = BytesN::from_array(&env, &[2u8; 32]);
        
        tree.insert(leaf1);
        tree.insert(leaf2);
        
        // Verify tree properties
        assert_eq!(tree.get_depth(), 1);
        assert_eq!(tree.get_leaf_count(), 2);
        
        // Generate proof for first leaf
        let proof = tree.generate_proof(0).unwrap();
        let (siblings, depth) = proof;
        
        assert_eq!(depth, 1);
        assert_eq!(siblings.len(), 2); // depth + 1
        
        // Verify the leaf exists
        let leaf = tree.get_leaf(0).unwrap();
        assert_eq!(leaf, BytesN::from_array(&env, &[1u8; 32]));
    }
}
