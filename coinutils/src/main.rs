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
    let poseidon = Poseidon255::new(env);
    
    match inputs.len() {
        1 => poseidon.hash(&inputs[0]),
        2 => poseidon.hash_two(&inputs[0], &inputs[1]),
        _ => {
            // For more than 2 inputs, hash them sequentially
            let mut result = inputs[0].clone();
            for input in inputs.iter().skip(1) {
                result = poseidon.hash_two(&result, input);
            }
            result
        }
    }
}


fn hex_string_to_bls_scalar(env: &Env, hex_str: &str) -> Result<BlsScalar, String> {
    // Remove 0x prefix if present
    let hex_str = hex_str.trim_start_matches("0x");
    
    // Parse hex string to bytes
    let bytes = hex::decode(hex_str)
        .map_err(|e| format!("Invalid hex string: {:?}", e))?;
    
    if bytes.len() != 32 {
        return Err(format!("Hex string must be 32 bytes, got {}", bytes.len()));
    }
    
    let mut byte_array = [0u8; 32];
    byte_array.copy_from_slice(&bytes);
    
    Ok(BlsScalar::from_bytes(BytesN::from_array(env, &byte_array)))
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

    let value_hex = hex::encode(value.to_bytes().to_array());
    let nullifier_hex = hex::encode(nullifier.to_bytes().to_array());
    let secret_hex = hex::encode(secret.to_bytes().to_array());
    let label_hex = hex::encode(label.to_bytes().to_array());
    let commitment_hex = hex::encode(commitment.to_bytes().to_array());

    let coin_data = CoinData {
        value: value_hex,
        nullifier: nullifier_hex,
        secret: secret_hex,
        label: label_hex,
        commitment: commitment_hex.clone(),
    };

    GeneratedCoin {
        coin: coin_data,
        commitment_hex: format!("0x{}", commitment_hex),
    }
}

fn withdraw_coin(env: &Env, coin: &CoinData, state_file: &StateFile) -> Result<SnarkInput, String> {
    // Parse hex string values to BlsScalar
    let value = hex_string_to_bls_scalar(env, &coin.value)?;
    let nullifier = hex_string_to_bls_scalar(env, &coin.nullifier)?;
    let secret = hex_string_to_bls_scalar(env, &coin.secret)?;
    let label = hex_string_to_bls_scalar(env, &coin.label)?;

    // Reconstruct the commitment to verify it matches
    let commitment = generate_commitment(env, value.clone(), label.clone(), nullifier.clone(), secret.clone());
    
    // Build merkle tree from state file using lean-imt
    let mut tree = LeanIMT::new(env, TREE_DEPTH);
    let mut commitment_index = None;
    
    for (index, commitment_str) in state_file.commitments.iter().enumerate() {
        let commitment_fr = hex_string_to_bls_scalar(env, commitment_str)
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

    let label_hex = hex::encode(label.to_bytes().to_array());
    let value_hex = hex::encode(value.to_bytes().to_array());
    let nullifier_hex = hex::encode(nullifier.to_bytes().to_array());
    let secret_hex = hex::encode(secret.to_bytes().to_array());
    let state_root_hex = hex::encode(root_scalar.to_bytes().to_array());

    Ok(SnarkInput {
        withdrawn_value: COIN_VALUE.to_string(),
        label: label_hex,
        value: value_hex,
        nullifier: nullifier_hex,
        secret: secret_hex,
        state_root: state_root_hex,
        state_index: commitment_index.to_string(),
        state_siblings: siblings.into_iter()
            .map(|s| hex::encode(s.to_bytes().to_array()))
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