use lean_imt::LeanIMT;
use serde::{Deserialize, Serialize};
use soroban_sdk::{BytesN, Env};

#[derive(Serialize, Deserialize, Debug)]
struct PoseidonTestResult {
    input: u64,
    input_hex: String,
    lean_imt_hash: String,
    description: String,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "poseidon" {
        // Poseidon hash mode - testing single input hashing
        if args.len() < 3 {
            println!("Usage: cargo run -- poseidon <input_value>");
            println!("Example: cargo run -- poseidon 123456789");
            return;
        }
        
        let input_value = args[2].parse().unwrap_or(123456789);
        println!("🧪 Testing Poseidon single input hash compatibility");
        println!("================================================");
        println!("Testing compatibility between lean-imt and test_poseidon.circom");
        println!("");
        
        let env = Env::default();
        let input_bytes = u64_to_bytes32(&env, input_value);
        
        // Test lean-imt poseidon hash functionality for single input
        println!("Input value: {}", input_value);
        println!("Input bytes: 0x{}", hex::encode(&input_bytes.to_array()));
        
        // Create a single leaf tree to test the poseidon hash
        let mut test_tree = LeanIMT::new(env.clone());
        test_tree.insert(input_bytes.clone());
        
        let root = test_tree.get_root();
        let root_hex = format!("0x{}", hex::encode(&root.to_array()));
        
        println!("lean-imt Poseidon hash result: {}", root_hex);
        println!("Tree depth: {}", test_tree.get_depth());
        println!("Leaf count: {}", test_tree.get_leaf_count());
        
        // Save result for circuit compatibility testing
        let result = PoseidonTestResult {
            input: input_value,
            input_hex: format!("0x{}", hex::encode(&input_bytes.to_array())),
            lean_imt_hash: root_hex.clone(),
            description: "Single input Poseidon hash test".to_string(),
        };
        
        let json_result = serde_json::to_string_pretty(&result).unwrap();
        std::fs::write("poseidon_single_result.json", json_result).unwrap();
        println!("📁 Result saved to: poseidon_single_result.json");
        
        println!("");
        println!("🎯 This result can now be used to test compatibility with test_poseidon.circom");
        println!("   The circuit should produce the same hash output for the same input value.");
        println!("   Input: {} -> Expected Hash: {}", input_value, root_hex);
        
        return;
    }
    
    // Default mode - run comprehensive Poseidon single input tests
    println!("🧪 Testing Poseidon single input hash compatibility");
    println!("================================================");
    println!("Testing compatibility between lean-imt and test_poseidon.circom");
    println!("");

    let env = Env::default();
    let mut results = Vec::new();

    // Test Case 1: Simple single input
    println!("Test Case 1: Single input value 42");
    let mut tree1 = LeanIMT::new(env.clone());
    tree1.insert(u64_to_bytes32(&env, 42));
    
    let root1 = tree1.get_root();
    let root1_hex = format!("0x{}", hex::encode(&root1.to_array()));
    
    println!("  Input: 42");
    println!("  lean-imt Hash: {}", root1_hex);
    
    let result1 = PoseidonTestResult {
        input: 42,
        input_hex: format!("0x{}", hex::encode(&u64_to_bytes32(&env, 42).to_array())),
        lean_imt_hash: root1_hex.clone(),
        description: "Test Case 1: Single input value 42".to_string(),
    };
    results.push(result1);

    // Test Case 2: Another single input
    println!("\nTest Case 2: Single input value 123456789");
    let mut tree2 = LeanIMT::new(env.clone());
    tree2.insert(u64_to_bytes32(&env, 123456789));
    
    let root2 = tree2.get_root();
    let root2_hex = format!("0x{}", hex::encode(&root2.to_array()));
    
    println!("  Input: 123456789");
    println!("  lean-imt Hash: {}", root2_hex);
    
    let result2 = PoseidonTestResult {
        input: 123456789,
        input_hex: format!("0x{}", hex::encode(&u64_to_bytes32(&env, 123456789).to_array())),
        lean_imt_hash: root2_hex.clone(),
        description: "Test Case 2: Single input value 123456789".to_string(),
    };
    results.push(result2);

    // Test Case 3: Zero input
    println!("\nTest Case 3: Single input value 0");
    let mut tree3 = LeanIMT::new(env.clone());
    tree3.insert(u64_to_bytes32(&env, 0));
    
    let root3 = tree3.get_root();
    let root3_hex = format!("0x{}", hex::encode(&root3.to_array()));
    
    println!("  Input: 0");
    println!("  lean-imt Hash: {}", root3_hex);
    
    let result3 = PoseidonTestResult {
        input: 0,
        input_hex: format!("0x{}", hex::encode(&u64_to_bytes32(&env, 0).to_array())),
        lean_imt_hash: root3_hex.clone(),
        description: "Test Case 3: Single input value 0".to_string(),
    };
    results.push(result3);
    
    // Save results to JSON
    let json_output = serde_json::to_string_pretty(&results).unwrap();
    std::fs::write("poseidon_single_result.json", json_output).unwrap();
    
    println!("\n✅ Poseidon single input hash testing complete!");
    println!("📁 Results saved to: poseidon_single_result.json");
    println!("\n🎯 These results can now be used to test compatibility with test_poseidon.circom");
    println!("   Each test case provides:");
    println!("   - Input value (decimal)");
    println!("   - Input value (hex bytes)");
    println!("   - Expected hash output from lean-imt");
    println!("   - The circuit should produce matching hash outputs");
}

fn u64_to_bytes32(env: &Env, value: u64) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0..8].copy_from_slice(&value.to_le_bytes());
    BytesN::from_array(env, &bytes)
}


