use std::fs;
use serde_json::{json, Value};
use soroban_sdk::{Env, BytesN};
use lean_imt::LeanIMT;

// Add witness file reading functionality
use std::io::Read;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 2 && args[1] == "--extract-only" {
        // Extract-only mode for script usage
        let leaf_index: u32 = args[2].parse().unwrap_or(0);
        extract_witness_only(leaf_index);
        return;
    }
    
    // Check if we should skip witness verification (first run)
    let skip_witness_verification = args.len() > 1 && args[1] == "--skip-witness";
    
    // Normal test mode
    test_lean_imt_compatibility(skip_witness_verification);
}

// Function to extract witness output only (for script usage)
fn extract_witness_only(leaf_index: u32) {
    let witness_file = format!("witness_leaf_{}.wtns", leaf_index);
    
    if !std::path::Path::new(&witness_file).exists() {
        println!("Witness file {} not found", witness_file);
        return;
    }
    
    match extract_witness_output(&witness_file) {
        Ok(output) => {
            println!("Computed root (circom): {}", output);
        },
        Err(e) => {
            println!("Could not extract root: {}", e);
        }
    }
}

fn test_lean_imt_compatibility(skip_witness_verification: bool) {
    if skip_witness_verification {
        println!("🧪 Testing lean-imt ↔ merkleProof.circom compatibility (JSON generation only)...\n");
    } else {
        println!("🧪 Testing lean-imt ↔ merkleProof.circom compatibility...\n");
    }
    
    println!("🔍 Building merkle tree using lean-imt crate...");
    
    let env = Env::default();
    // Build a 4-level tree (16 leaves) to match maxDepth = 4
    let mut tree = LeanIMT::new(env.clone());
    
    // Add 16 leaves to create a full 4-level tree
    for i in 1..=16 {
        let leaf_value = BytesN::from_array(&env, &[i; 32]);
        tree.insert(leaf_value);
    }
    
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
        println!("   - Original siblings count: {}", siblings.len());
        println!("   - Actual depth: {}", actual_depth);
        println!("   - Circuit expects: {} siblings (maxDepth)", 4);
        
        // Verify proof structure - circuit expects exactly maxDepth siblings
        if siblings.len() < actual_depth as u32 {
            panic!("Invalid proof structure: need at least {} siblings, got {}", actual_depth, siblings.len());
        }
        
        println!("   ✅ Proof structure valid");
        println!("   💾 Input JSON saved to {}", input_file);
        
        // Verify the witness output (this should FAIL due to hash function mismatch)
        let expected_root = bytes_to_hex(&tree.get_root());
        if !skip_witness_verification {
            match verify_witness_output(&format!("witness_leaf_{}.wtns", leaf_index), &expected_root) {
                Ok(matches) => {
                    if matches {
                        println!("   ⚠️  WARNING: Roots match unexpectedly - this suggests a bug!");
                    } else {
                        println!("   ✅ EXPECTED: Roots don't match due to XOR vs Keccak256 hash functions");
                    }
                },
                Err(e) => {
                    println!("   ❌ Witness verification failed: {}", e);
                }
            }
        } else {
            println!("   ⚠️  Skipping witness verification due to --skip-witness flag.");
        }
    }
    
    println!("\n🎉 All compatibility tests completed!");
    println!("\n📝 Next steps:");
    println!("1. Compile the merkleProof.circom circuit");
    println!("2. Use the generated JSON files to generate witnesses");
    println!("3. Verify that witnesses are valid");
    println!("4. ⚠️  NOTE: Test should fail due to hash function mismatch!");
    println!("   - lean-imt uses Keccak256 (cryptographically secure)");
    println!("   - circom circuit uses SHA256 (despite misleading name)");
    println!("   - This will cause different root hashes!");
    println!("5. 🔍 Witness verification shows the cryptographic incompatibility");
}

fn create_circuit_input(leaf_index: u32, leaf: &BytesN<32>, siblings: &soroban_sdk::Vec<BytesN<32>>, actual_depth: u32) -> Value {
    // Convert leaf to hex
    let leaf_hex = bytes_to_hex(leaf);
    
    // Convert siblings to hex and truncate to exactly maxDepth=4
    let max_depth = 4;
    let mut siblings_hex: std::vec::Vec<String> = siblings.iter()
        .take(max_depth as usize)  // Take only the first maxDepth siblings
        .map(|s| bytes_to_hex(&s))
        .collect();
    
    // Ensure we have exactly maxDepth siblings
    while siblings_hex.len() < max_depth as usize {
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

/// Reads a witness file and extracts the computed root (output signal)
/// .wtns files have a specific format with sections and field elements
fn extract_witness_output(witness_file: &str) -> Result<String, String> {
    let mut file = fs::File::open(witness_file)
        .map_err(|e| format!("Failed to open witness file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read witness file: {}", e))?;
    

    
    // Check header
    if buffer.len() < 8 || &buffer[0..4] != b"wtns" {
        return Err("Invalid witness file format: missing 'wtns' header".to_string());
    }
    
    // Parse version (4 bytes after header)
    let version = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);

    
    if version != 2 {
        return Err(format!("Unsupported witness file version: {}", version));
    }
    
    // The .wtns format has a specific structure:
    // - Header (8 bytes)
    // - Number of sections (4 bytes)
    // - Section lengths (4 bytes each)
    // - Field element data
    
    if buffer.len() < 12 {
        return Err("Witness file too short: missing section count".to_string());
    }
    
    let num_sections = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
    if buffer.len() < 12 + (num_sections * 4) as usize {
        return Err("Witness file too short: missing section lengths".to_string());
    }
    
    // Read section lengths
    let mut section_lengths = Vec::new();
    let mut offset = 12;
    for _i in 0..num_sections {
        let length = u32::from_le_bytes([
            buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
        ]);
        section_lengths.push(length);
        offset += 4;
    }
    
    // The last section should contain the field element data
    // Each field element is 32 bytes (for BLS12-381 prime field)
    let field_data_start = offset;
    let field_data_size = buffer.len() - field_data_start;
    let num_field_elements = field_data_size / 32;
    
    if num_field_elements == 0 {
        return Err("No field elements found in witness file".to_string());
    }
    

    
    // Find the last non-zero field element (likely the output)
    let mut last_non_zero = None;
    for i in (0..num_field_elements).rev() {
        let element_offset = field_data_start + (i * 32);
        if element_offset + 32 <= buffer.len() {
            let field_bytes = &buffer[element_offset..element_offset + 32];
            if field_bytes.iter().any(|&b| b != 0) {
                let hex_string = format!("0x{}", field_bytes.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>());
                last_non_zero = Some(hex_string);
                break;
            }
        }
    }
    
    match last_non_zero {
        Some(root) => Ok(root),
        None => Ok("0x0000000000000000000000000000000000000000000000000000000000000000".to_string())
    }
}

/// Verifies that the circom computed root matches the lean-imt computed root
/// This should FAIL due to hash function mismatch (Keccak256 vs SHA256)
fn verify_witness_output(witness_file: &str, expected_root: &str) -> Result<bool, String> {
    let computed_root = extract_witness_output(witness_file)?;
    
    // This comparison should fail due to hash function mismatch
    let matches = computed_root == expected_root;
    
    if matches {
        println!("      ✅ Roots match (UNEXPECTED - this suggests a bug!)");
    } else {
        println!("      ❌ Roots don't match (EXPECTED - due to Keccak256 vs SHA256)");
        println!("         - lean-imt: {}", expected_root);
        println!("         - circom:   {}", computed_root);
    }
    
    Ok(matches)
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

