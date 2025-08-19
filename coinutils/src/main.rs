use ark_bls12_381::{Fr};
use ark_ff::PrimeField;
use rand::{thread_rng, Rng};
use dusk_poseidon::{Domain, Hash};
use dusk_bls12_381::BlsScalar;
use ark_ff::biginteger::BigInteger;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;

const COIN_VALUE: i128 = 1000000000; // 1 XLM in stroops

#[derive(Serialize)]
struct SnarkInput {
    #[serde(rename = "withdrawnValue")]
    withdrawn_value: String,
    label: String,
    #[serde(rename = "existingValue")]
    existing_value: String,
    #[serde(rename = "existingNullifier")]
    existing_nullifier: String,
    #[serde(rename = "existingSecret")]
    existing_secret: String,
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

fn random_fr() -> Fr {
    let mut rng = thread_rng();
    Fr::from(rng.gen::<u64>())
}

// Convert ark-ff Fr to dusk BlsScalar
fn fr_to_bls_scalar(fr: &Fr) -> BlsScalar {
    let bytes = fr.into_bigint().to_bytes_le();
    let mut padded_bytes = [0u8; 32];
    let copy_len = std::cmp::min(bytes.len(), 32);
    padded_bytes[..copy_len].copy_from_slice(&bytes[..copy_len]);
    BlsScalar::from_bytes(&padded_bytes).unwrap_or_else(|| {
        // Fallback: use wide reduction if canonical form fails
        let mut wide = [0u8; 64];
        wide[..copy_len].copy_from_slice(&bytes[..copy_len]);
        BlsScalar::from_bytes_wide(&wide)
    })
}

// Convert dusk BlsScalar to ark-ff Fr
fn bls_scalar_to_fr(scalar: &BlsScalar) -> Fr {
    let bytes = scalar.to_bytes();
    Fr::from_le_bytes_mod_order(&bytes)
}

// Poseidon-based hash for field elements
fn poseidon_hash(inputs: &[Fr]) -> Fr {
    let bls_inputs: Vec<BlsScalar> = inputs.iter().map(fr_to_bls_scalar).collect();
    
    // Use Domain::Other for general hashing (similar to the contract)
    let hash_result = Hash::digest(Domain::Other, &bls_inputs);
    
    // Convert back to ark-ff Fr
    bls_scalar_to_fr(&hash_result[0])
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

fn withdraw_coin(existing_coin: &CoinData, _scope: &[u8]) -> SnarkInput {
    
    let existing_value = Fr::from_str(&existing_coin.value).unwrap();
    let existing_nullifier = Fr::from_str(&existing_coin.nullifier).unwrap();
    let existing_secret = Fr::from_str(&existing_coin.secret).unwrap();
    let existing_label = Fr::from_str(&existing_coin.label).unwrap();

    let snark_input = SnarkInput {
        withdrawn_value: COIN_VALUE.to_string(),
        label: existing_label.into_bigint().to_string(),
        existing_value: existing_value.into_bigint().to_string(),
        existing_nullifier: existing_nullifier.into_bigint().to_string(),
        existing_secret: existing_secret.into_bigint().to_string()
    };

    snark_input
}

fn print_usage() {
    println!("Usage:");
    println!("  coinutils generate [scope] [output_file]  - Generate a new coin");
    println!("  coinutils withdraw <coin_file> [scope] [output_file]  - Withdraw a coin");
    println!();
    println!("Examples:");
    println!("  coinutils generate my_pool_scope coin.json");
    println!("  coinutils withdraw coin.json my_pool_scope withdrawal.json");
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
            if args.len() < 3 {
                println!("Error: withdraw command requires a coin file");
                print_usage();
                return;
            }
            
            let coin_file = &args[2];
            let scope_str = args.get(3).cloned().unwrap_or_else(|| "default_pool_scope".to_string());
            let output_file = args.get(4).cloned().unwrap_or_else(|| "withdrawal.json".to_string());
            
            // Read existing coin
            let coin_content = std::fs::read_to_string(coin_file)
                .expect(&format!("Failed to read coin file: {}", coin_file));
            let existing_coin: GeneratedCoin = serde_json::from_str(&coin_content)
                .expect(&format!("Failed to parse coin file: {}", coin_file));
            
            let snark_input = withdraw_coin(&existing_coin.coin, scope_str.as_bytes());
            
            // Save withdrawal data
            let withdrawal_json = serde_json::to_string_pretty(&snark_input).unwrap();
            let mut file = File::create(&output_file).unwrap();
            file.write_all(withdrawal_json.as_bytes()).unwrap();
            
            println!("Withdrawal created:");
            println!("  Withdrawn value: {}", snark_input.withdrawn_value);
            println!("  Label: {}", snark_input.label);
            println!("  Snark input saved to: {}", output_file);
        }
        
        _ => {
            println!("Unknown command: {}", args[1]);
            print_usage();
        }
    }
}