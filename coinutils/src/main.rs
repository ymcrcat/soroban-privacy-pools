use ark_bls12_381::{Fr};
use ark_ff::PrimeField;
use rand::{thread_rng, Rng};
use sha3::{Digest, Keccak256};
use ark_ff::biginteger::BigInteger;

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
    Fr::from_be_bytes_mod_order(&hashed)
}

fn to_bytesn32(fr: &Fr) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let fr_bytes = fr.into_bigint().to_bytes_le();
    let offset = 32 - fr_bytes.len();
    bytes[offset..].copy_from_slice(&fr_bytes);
    bytes
}

fn main() {
    let value = Fr::from(1u64);  // fixed amount
    let nullifier = random_fr();
    let secret = random_fr();

    let scope = b"example_pool_scope";
    let nonce = thread_rng().gen::<[u8; 32]>();

    // label = keccak256(scope || nonce)
    let mut hasher = Keccak256::new();
    hasher.update(scope);
    hasher.update(&nonce);
    let hashed = hasher.finalize();
    let label = Fr::from_be_bytes_mod_order(&hashed);

    // commitment = Hash(value, label, Hash(nullifier, secret))
    let precommitment = field_hash(&[nullifier, secret]);
    let commitment = field_hash(&[value, label, precommitment]);

    println!("Nullifier: {}", nullifier);
    println!("Secret: {}", secret);
    println!("Label: {}", label);
    println!("Commitment: 0x{}", hex::encode(to_bytesn32(&commitment)));
    println!("Use this 32-byte commitment in your deposit call.");
}