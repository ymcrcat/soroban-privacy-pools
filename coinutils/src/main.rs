use ark_bls12_381::Fr;
use ark_ff::PrimeField;
use rand::{thread_rng, Rng};
use poseidon::Poseidon255;
use ark_ff::biginteger::BigInteger;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use lean_imt::LeanIMT;
use soroban_sdk::Env;

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
    state_siblings: Vec<String>,
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
    commitments: Vec<String>,
    scope: String,
}

fn random_fr() -> Fr {
    let mut rng = thread_rng();
    Fr::from(rng.gen::<u64>())
}

// Poseidon-based hash for field elements
fn poseidon_hash(inputs: &[Fr]) -> Fr {
    let poseidon = Poseidon255::new();
    
    match inputs.len() {
        1 => poseidon.hash(&inputs[0]),
        2 => poseidon.hash_two(&inputs[0], &inputs[1]),
        _ => {
            // For more than 2 inputs, hash them sequentially
            let mut result = inputs[0];
            for input in inputs.iter().skip(1) {
                result = poseidon.hash_two(&result, input);
            }
            result
        }
    }
}

fn to_bytesn32(fr: &Fr) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let fr_bytes = fr.into_bigint().to_bytes_le();
    let offset = 32 - fr_bytes.len();
    bytes[offset..].copy_from_slice(&fr_bytes);
    bytes
}

fn generate_label(scope: &[u8], nonce: &[u8; 32]) -> Fr {
    // Convert scope and nonce to field elements for Poseidon hashing
    let scope_fr = Fr::from_le_bytes_mod_order(scope);
    let nonce_fr = Fr::from_le_bytes_mod_order(nonce);
    
    // Hash using Poseidon
    poseidon_hash(&[scope_fr, nonce_fr])
}

fn generate_commitment(value: Fr, label: Fr, nullifier: Fr, secret: Fr) -> Fr {
    let precommitment = poseidon_hash(&[nullifier, secret]);
    poseidon_hash(&[value, label, precommitment])
}

fn generate_coin(scope: &[u8]) -> GeneratedCoin {
    let value = Fr::from(COIN_VALUE as u64);
    let nullifier = random_fr();
    let secret = random_fr();
    let nonce = thread_rng().gen::<[u8; 32]>();
    let label = generate_label(scope, &nonce);
    let commitment = generate_commitment(value, label, nullifier, secret);

    let coin_data = CoinData {
        value: value.into_bigint().to_string(),
        nullifier: nullifier.into_bigint().to_string(),
        secret: secret.into_bigint().to_string(),
        label: label.into_bigint().to_string(),
        commitment: commitment.into_bigint().to_string(),
    };

    GeneratedCoin {
        coin: coin_data,
        commitment_hex: format!("0x{}", hex::encode(to_bytesn32(&commitment))),
    }
}

fn withdraw_coin(coin: &CoinData, state_file: &StateFile) -> Result<SnarkInput, String> {
    let value = Fr::from_str(&coin.value).map_err(|e| format!("Invalid coin value: {:?}", e))?;
    let nullifier = Fr::from_str(&coin.nullifier).map_err(|e| format!("Invalid coin nullifier: {:?}", e))?;
    let secret = Fr::from_str(&coin.secret).map_err(|e| format!("Invalid coin secret: {:?}", e))?;
    let label = Fr::from_str(&coin.label).map_err(|e| format!("Invalid coin label: {:?}", e))?;

    // Reconstruct the commitment to verify it matches
    let commitment = generate_commitment(value, label, nullifier, secret);
    
    // Build merkle tree from state file using lean-imt
    let env = Env::default();
    let mut tree = LeanIMT::new(&env, TREE_DEPTH);
    let mut commitment_index = None;
    
    for (index, commitment_str) in state_file.commitments.iter().enumerate() {
        let commitment_fr = Fr::from_str(commitment_str)
            .map_err(|e| format!("Invalid commitment at index {}: {:?}", index, e))?;
        
        // Convert Fr to bytes and insert into lean-imt
        let commitment_bytes = lean_imt::bls_scalar_to_bytes(&env, commitment_fr);
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
    
    // Convert siblings from BlsScalar to Fr and then to strings
    let siblings: Vec<Fr> = siblings_scalars.iter()
        .map(|s| *s)
        .collect();

    // Get the root from lean-imt
    let root_scalar = lean_imt::bytes_to_bls_scalar(&tree.get_root());

    Ok(SnarkInput {
        withdrawn_value: COIN_VALUE.to_string(),
        label: label.into_bigint().to_string(),
        value: value.into_bigint().to_string(),
        nullifier: nullifier.into_bigint().to_string(),
        secret: secret.into_bigint().to_string(),
        state_root: root_scalar.into_bigint().to_string(),
        state_index: commitment_index.to_string(),
        state_siblings: siblings.into_iter()
            .map(|s| s.into_bigint().to_string())
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
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "generate" => {
            if args.len() < 3 {
                println!("Error: generate command requires a scope");
                print_usage();
                return;
            }
            
            let scope = args[2].as_bytes();
            let output_file = args.get(3).cloned().unwrap_or_else(|| "coin.json".to_string());
            
            let generated_coin = generate_coin(scope);
            
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
            let output_file = args.get(4).cloned().unwrap_or_else(|| "withdrawal.json".to_string());
            
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
            
            match withdraw_coin(&existing_coin.coin, &state_data) {
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