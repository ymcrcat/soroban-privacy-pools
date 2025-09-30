use soroban_sdk::{
    crypto::bls12_381::Fr as BlsScalar,
    Env, BytesN, U256,
};
use rand::{thread_rng, Rng};
use poseidon::Poseidon255;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Write;
use lean_imt::LeanIMT;
use num_bigint::BigUint;

const COIN_VALUE: i128 = 1000000000; // 1 XLM in stroops
const TREE_DEPTH: u32 = 2;

#[derive(Serialize)]
struct SnarkInput {
    #[serde(rename = "withdrawnValue")]
    withdrawn_value: String,
    label: String,
    value: String,
    nullifier: String,
    secret: String,
    #[serde(rename = "stateRoot")]
    state_root: String,
    #[serde(rename = "stateIndex")]
    state_index: String,
    #[serde(rename = "stateSiblings")]
    state_siblings: std::vec::Vec<String>,
}

#[derive(Serialize, serde::Deserialize)]
struct CoinData {
    value: String,
    nullifier: String,
    secret: String,
    label: String,
    commitment: String,
}

#[derive(Serialize, serde::Deserialize)]
struct GeneratedCoin {
    coin: CoinData,
    commitment_hex: String,
}

#[derive(Serialize, Deserialize)]
struct StateFile {
    commitments: std::vec::Vec<String>,
    scope: String,
}

fn random_fr(env: &Env) -> BlsScalar {
    let mut rng = thread_rng();
    BlsScalar::from_u256(U256::from_u32(env, rng.gen::<u32>()))
}

// Poseidon-based hash for field elements
fn poseidon_hash(env: &Env, inputs: &[BlsScalar]) -> BlsScalar {
    let poseidon1 = Poseidon255::new(env);
    let poseidon2 = Poseidon255::new_with_t(env, 3);
    
    match inputs.len() {
        1 => poseidon1.hash(&inputs[0]),
        2 => poseidon2.hash_two(&inputs[0], &inputs[1]),
        _ => {
            // For more than 2 inputs, hash them sequentially
            let mut result = inputs[0].clone();
            for input in inputs.iter().skip(1) {
                result = poseidon2.hash_two(&result, input);
            }
            result
        }
    }
}

fn decimal_string_to_bls_scalar(env: &Env, decimal_str: &str) -> Result<BlsScalar, String> {
    // For now, let's use a simpler approach that works with the existing system
    // We'll convert the decimal to a u128 first, then to BlsScalar
    if let Ok(value) = decimal_str.parse::<u128>() {
        // Convert u128 to BlsScalar
        return Ok(BlsScalar::from_u256(U256::from_u32(env, value as u32)));
    }
    
    // For very large numbers, we need to handle them differently
    // Since the decimal numbers are too large for u128, we'll use a workaround
    // by converting through the existing hex conversion system
    
    // First, let's try to convert the decimal to hex manually
    let mut temp = decimal_str.to_string();
    let mut hex_digits = String::new();
    
    while !temp.is_empty() && temp != "0" {
        let mut carry = 0u32;
        let mut new_temp = String::new();
        
        for ch in temp.chars() {
            let digit = ch.to_digit(10).ok_or_else(|| "Invalid decimal character")? as u32;
            let value = carry * 10 + digit;
            new_temp.push((b'0' + (value / 16) as u8) as char);
            carry = value % 16;
        }
        
        // Remove leading zeros
        while new_temp.len() > 1 && new_temp.starts_with('0') {
            new_temp.remove(0);
        }
        
        if new_temp.is_empty() {
            new_temp = "0".to_string();
        }
        
        temp = new_temp;
        hex_digits.push_str(&format!("{:x}", carry));
    }
    
    // Reverse the hex string since we built it backwards
    let hex_str: String = hex_digits.chars().rev().collect();
    
    // Pad to 64 hex characters (32 bytes)
    let padded_hex = format!("{:0>64}", hex_str);
    
    // Convert hex to bytes
    let bytes = hex::decode(&padded_hex)
        .map_err(|e| format!("Hex conversion failed: {:?}", e))?;
    
    if bytes.len() != 32 {
        return Err("Invalid byte length".to_string());
    }
    
    let mut byte_array = [0u8; 32];
    byte_array.copy_from_slice(&bytes);
    
    Ok(BlsScalar::from_bytes(BytesN::from_array(env, &byte_array)))
}


/// Helper function to convert BlsScalar to decimal string
fn bls_scalar_to_decimal_string(scalar: &BlsScalar) -> String {
    let array = scalar.to_bytes().to_array();
    bytes_to_decimal_string(&array)
}

/// Helper function to convert bytes to decimal string
/// Uses num-bigint for efficient conversion
fn bytes_to_decimal_string(bytes: &[u8; 32]) -> String {
    let biguint = BigUint::from_bytes_be(bytes);
    biguint.to_str_radix(10)
}

fn generate_label(env: &Env, scope: &[u8], nonce: &[u8; 32]) -> BlsScalar {
    // Convert scope and nonce to field elements for Poseidon hashing
    let scope_fr = BlsScalar::from_bytes(BytesN::from_array(env, &{
        let mut bytes = [0u8; 32];
        let len = scope.len().min(32);
        bytes[..len].copy_from_slice(&scope[..len]);
        bytes
    }));
    let nonce_fr = BlsScalar::from_bytes(BytesN::from_array(env, nonce));
    
    // Hash using Poseidon
    poseidon_hash(env, &[scope_fr, nonce_fr])
}

fn generate_commitment(env: &Env, value: BlsScalar, label: BlsScalar, nullifier: BlsScalar, secret: BlsScalar) -> BlsScalar {
    let precommitment = poseidon_hash(env, &[nullifier, secret]);
    poseidon_hash(env, &[value, label, precommitment])
}

fn generate_coin(env: &Env, scope: &[u8]) -> GeneratedCoin {
    let value = BlsScalar::from_u256(U256::from_u32(env, COIN_VALUE as u32));
    let nullifier = random_fr(env);
    let secret = random_fr(env);
    let nonce = thread_rng().gen::<[u8; 32]>();
    let label = generate_label(env, scope, &nonce);
    let commitment = generate_commitment(env, value.clone(), label.clone(), nullifier.clone(), secret.clone());

    let value_decimal = bls_scalar_to_decimal_string(&value);
    let nullifier_decimal = bls_scalar_to_decimal_string(&nullifier);
    let secret_decimal = bls_scalar_to_decimal_string(&secret);
    let label_decimal = bls_scalar_to_decimal_string(&label);
    let commitment_decimal = bls_scalar_to_decimal_string(&commitment);

    let coin_data = CoinData {
        value: value_decimal,
        nullifier: nullifier_decimal,
        secret: secret_decimal,
        label: label_decimal,
        commitment: commitment_decimal,
    };

    GeneratedCoin {
        coin: coin_data,
        commitment_hex: format!("0x{}", hex::encode(commitment.to_bytes().to_array())),
    }
}

fn withdraw_coin(env: &Env, coin: &CoinData, state_file: &StateFile) -> Result<SnarkInput, String> {
    // Parse decimal string values to BlsScalar
    let value = decimal_string_to_bls_scalar(env, &coin.value)?;
    let nullifier = decimal_string_to_bls_scalar(env, &coin.nullifier)?;
    let secret = decimal_string_to_bls_scalar(env, &coin.secret)?;
    let label = decimal_string_to_bls_scalar(env, &coin.label)?;

    // Reconstruct the commitment to verify it matches
    let commitment = generate_commitment(env, value.clone(), label.clone(), nullifier.clone(), secret.clone());
    
    // Build merkle tree from state file using lean-imt
    let mut tree = LeanIMT::new(env, TREE_DEPTH);
    let mut commitment_index = None;
    
    for (index, commitment_str) in state_file.commitments.iter().enumerate() {
        let commitment_fr = decimal_string_to_bls_scalar(env, commitment_str)
            .map_err(|e| format!("Invalid commitment at index {}: {}", index, e))?;
        
        // Convert BlsScalar to bytes and insert into lean-imt
        let commitment_bytes = lean_imt::bls_scalar_to_bytes(commitment_fr.clone());
        tree.insert(commitment_bytes);
        
        // Check if this is the commitment we're withdrawing
        if commitment_fr == commitment {
            commitment_index = Some(index);
        }
    }
    
    // Verify the commitment exists in the state
    let commitment_index = commitment_index.ok_or_else(|| {
        "The coin's commitment was not found in the state file".to_string()
    })?;
    
    // Generate merkle proof using lean-imt
    let proof = tree.generate_proof(commitment_index as u32)
        .ok_or_else(|| "Failed to generate merkle proof".to_string())?;
    let (siblings_scalars, _depth) = proof;
    
    // Convert siblings from BlsScalar to strings
    let siblings: std::vec::Vec<BlsScalar> = siblings_scalars.iter()
        .map(|s| s.clone())
        .collect();

    // Get the root from lean-imt
    let root_scalar = lean_imt::bytes_to_bls_scalar(&tree.get_root());

    let label_decimal = bls_scalar_to_decimal_string(&label);
    let value_decimal = bls_scalar_to_decimal_string(&value);
    let nullifier_decimal = bls_scalar_to_decimal_string(&nullifier);
    let secret_decimal = bls_scalar_to_decimal_string(&secret);
    let state_root_decimal = bls_scalar_to_decimal_string(&root_scalar);

    Ok(SnarkInput {
        withdrawn_value: COIN_VALUE.to_string(),
        label: label_decimal,
        value: value_decimal,
        nullifier: nullifier_decimal,
        secret: secret_decimal,
        state_root: state_root_decimal,
        state_index: commitment_index.to_string(),
        state_siblings: siblings.into_iter()
            .map(|s| bls_scalar_to_decimal_string(&s))
            .collect(),
    })
}

fn print_usage() {
    println!("Usage:");
    println!("  coinutils generate [scope] [output_file]  - Generate a new coin");
    println!("  coinutils withdraw <coin_file> <state_file> [output_file]  - Withdraw a coin");
    println!();
    println!("Examples:");
    println!("  coinutils generate my_pool_scope coin.json");
    println!("  coinutils withdraw coin.json state.json withdrawal.json");
    println!();
    println!("State file format:");
    println!("  {{");
    println!("    \"commitments\": [\"commitment1\", \"commitment2\", ...],");
    println!("    \"scope\": \"pool_scope\"");
    println!("  }}");
}

fn main() {
    let args: std::vec::Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }

    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    match args[1].as_str() {
        "generate" => {
            if args.len() < 3 {
                println!("Error: generate command requires a scope");
                print_usage();
                return;
            }
            
            let scope = args[2].as_bytes();
            let output_file = args.get(3).map(|s| s.clone()).unwrap_or_else(|| "coin.json".to_string());
            
            let generated_coin = generate_coin(&env, scope);
            
            // Save coin data
            let coin_json = serde_json::to_string_pretty(&generated_coin).unwrap();
            let mut file = File::create(&output_file).unwrap();
            file.write_all(coin_json.as_bytes()).unwrap();
            
            println!("Generated coin:");
            println!("  Value: {}", COIN_VALUE);
            println!("  Nullifier: {}", generated_coin.coin.nullifier);
            println!("  Secret: {}", generated_coin.coin.secret);
            println!("  Label: {}", generated_coin.coin.label);
            println!("  Commitment: {}", generated_coin.commitment_hex);
            println!("  Saved to: {}", output_file);
        }
        
        "withdraw" => {
            if args.len() < 4 {
                println!("Error: withdraw command requires both coin file and state file");
                print_usage();
                return;
            }
            
            let coin_file = &args[2];
            let state_file = &args[3];
            let output_file = args.get(4).map(|s| s.clone()).unwrap_or_else(|| "withdrawal.json".to_string());
            
            // Read existing coin
            let coin_content = std::fs::read_to_string(coin_file)
                .expect(&format!("Failed to read coin file: {}", coin_file));
            let existing_coin: GeneratedCoin = serde_json::from_str(&coin_content)
                .expect(&format!("Failed to parse coin file: {}", coin_file));
            
            // Read state file
            let state_content = std::fs::read_to_string(state_file)
                .expect(&format!("Failed to read state file: {}", state_file));
            let state_data: StateFile = serde_json::from_str(&state_content)
                .expect(&format!("Failed to parse state file: {}", state_file));
            
            match withdraw_coin(&env, &existing_coin.coin, &state_data) {
                Ok(snark_input) => {
                    // Save withdrawal data
                    let withdrawal_json = serde_json::to_string_pretty(&snark_input).unwrap();
                    let mut file = File::create(&output_file).unwrap();
                    file.write_all(withdrawal_json.as_bytes()).unwrap();
                    
                    println!("Withdrawal created:");
                    println!("  Withdrawn value: {}", snark_input.withdrawn_value);
                    println!("  State root: {}", snark_input.state_root);
                    println!("  Commitment index: {}", snark_input.state_index);
                    println!("  Snark input saved to: {}", output_file);
                }
                Err(e) => {
                    println!("Error creating withdrawal: {}", e);
                    return;
                }
            }
        }
        
        _ => {
            println!("Unknown command: {}", args[1]);
            print_usage();
        }
    }
}