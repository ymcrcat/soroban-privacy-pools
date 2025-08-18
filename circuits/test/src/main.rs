use lean_imt::LeanIMT;
use serde::{Deserialize, Serialize};
use soroban_sdk::{BytesN, Env};

#[derive(Serialize, Deserialize, Debug)]
struct TestCase {
    leaf: u64,
    leaf_index: u32,
    siblings: Vec<u64>,
    actual_depth: u32,
}

#[derive(Serialize, Deserialize)]
struct TestResult {
    leaf: u64,
    leaf_index: u32,
    siblings: Vec<u64>,
    actual_depth: u32,
    expected_root: String,
    description: String,
}

fn main() {
    // Test cases that match the JavaScript test data
    let test_cases = vec![
        TestCase {
            leaf: 1,
            leaf_index: 0,
            siblings: vec![2],
            actual_depth: 1,
        },
        TestCase {
            leaf: 2,
            leaf_index: 1,
            siblings: vec![1, 3],
            actual_depth: 2,
        },
        TestCase {
            leaf: 5,
            leaf_index: 2,
            siblings: vec![4, 6, 7],
            actual_depth: 3,
        },
        TestCase {
            leaf: 100,
            leaf_index: 0,
            siblings: vec![],
            actual_depth: 0,
        },
        TestCase {
            leaf: 1,
            leaf_index: 0,
            siblings: vec![2, 4, 8],
            actual_depth: 3,
        },
    ];

    let env = Env::default();
    let mut results = Vec::new();

    println!("🧪 Testing lean-imt compatibility with merkleProof.circom");
    println!("=======================================================");

    for (i, test_case) in test_cases.iter().enumerate() {
        println!("\nTest Case {}: {:?}", i + 1, test_case);
        
        // Create a tree and insert the test data
        let mut tree = LeanIMT::new(env.clone());
        
        // Insert the leaf
        let leaf_bytes = u64_to_bytes32(&env, test_case.leaf);
        tree.insert(leaf_bytes);
        
        // Insert siblings if they exist
        for &sibling in &test_case.siblings {
            if sibling != 0 {
                let sibling_bytes = u64_to_bytes32(&env, sibling);
                tree.insert(sibling_bytes);
            }
        }
        
        // Get the root
        let root = tree.get_root();
        let root_u64 = bytes32_to_u64(&root);
        
        println!("  Leaf: {}", test_case.leaf);
        println!("  Leaf Index: {}", test_case.leaf_index);
        println!("  Siblings: {:?}", test_case.siblings);
        println!("  Actual Depth: {}", test_case.actual_depth);
        println!("  lean-imt Root: {}", root_u64);
        
        // Create test result
        let result = TestResult {
            leaf: test_case.leaf,
            leaf_index: test_case.leaf_index,
            siblings: test_case.siblings.clone(),
            actual_depth: test_case.actual_depth,
            expected_root: root_u64.to_string(),
            description: format!("Test Case {}: {:?}", i + 1, test_case),
        };
        
        results.push(result);
    }
    
    // Save results to JSON
    let json_output = serde_json::to_string_pretty(&results).unwrap();
    std::fs::write("lean_imt_test_results.json", json_output).unwrap();
    
    println!("\n✅ lean-imt compatibility testing complete!");
    println!("📁 Results saved to: lean_imt_test_results.json");
}

fn u64_to_bytes32(env: &Env, value: u64) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0..8].copy_from_slice(&value.to_le_bytes());
    BytesN::from_array(env, &bytes)
}

fn bytes32_to_u64(bytes: &BytesN<32>) -> u64 {
    let array = bytes.to_array();
    u64::from_le_bytes([
        array[0], array[1], array[2], array[3],
        array[4], array[5], array[6], array[7]
    ])
}
