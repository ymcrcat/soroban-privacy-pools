use ark_bls12_381::{Fr};
use ark_ff::PrimeField;
use rand::{thread_rng, Rng};
use sha3::{Digest, Keccak256};
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

// Simple keccak256-based hash for field elements
fn field_hash(inputs: &[Fr]) -> Fr {
    let mut hasher = Keccak256::new();
    for input in inputs {
        hasher.update(&to_bytesn32(input));
    }
    let hashed = hasher.finalize();
    
    // Convert the hash to a field element, which will automatically reduce modulo the field size
    Fr::from_le_bytes_mod_order(&hashed)
}

fn to_bytesn32(fr: &Fr) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let fr_bytes = fr.into_bigint().to_bytes_le();
    let offset = 32 - fr_bytes.len();
    bytes[offset..].copy_from_slice(&fr_bytes);
    bytes
}

fn generate_label(scope: &[u8], nonce: &[u8; 32]) -> Fr {
    let mut hasher = Keccak256::new();
    hasher.update(scope);
    hasher.update(nonce);
    let hashed = hasher.finalize();
    
    // Ensure the label fits in 254 bits by masking the highest 2 bits
    // This prevents potential issues with field arithmetic
    let mut truncated_hash = [0u8; 32];
    truncated_hash.copy_from_slice(&hashed);
    
    // Clear the highest 2 bits to ensure 254-bit compatibility
    truncated_hash[31] &= 0x3F; // Clear bits 254 and 255
    
    Fr::from_le_bytes_mod_order(&truncated_hash)
}

fn generate_commitment(value: Fr, label: Fr, nullifier: Fr, secret: Fr) -> Fr {
    let precommitment = field_hash(&[nullifier, secret]);
    field_hash(&[value, label, precommitment])
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

fn withdraw_coin(existing_coin: &CoinData, scope: &[u8]) -> SnarkInput {
    
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