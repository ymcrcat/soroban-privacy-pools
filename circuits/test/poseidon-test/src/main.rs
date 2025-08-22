use ark_bls12_381::Fr as BlsScalar;
use ark_ff::{Field, PrimeField};
use poseidon::Poseidon255;
use serde::Deserialize;
use std::io::{self, Read};
use num_bigint::BigUint;

#[derive(Deserialize)]
struct Input {
    #[serde(rename = "in")]
    in_value: serde_json::Value,
}

fn bls_scalar_to_decimal(scalar: BlsScalar) -> String {
    // Access the internal representation using into_bigint
    let bigint = scalar.into_bigint();
    
    // Convert the BigInt to BigUint for string conversion
    let mut value = BigUint::from(0u64);
    for (i, &limb) in bigint.as_ref().iter().enumerate() {
        value += BigUint::from(limb) << (i * 64);
    }
    
    value.to_str_radix(10)
}

fn main() {
    // Read JSON input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("Failed to read input");
    
    // Parse the JSON input
    let input_data: Input = serde_json::from_str(&input).expect("Failed to parse JSON");
    
    // Convert to BlsScalar and hash
    let input_u64 = match input_data.in_value {
        serde_json::Value::String(s) => s.parse::<u64>().expect("Failed to parse string to u64"),
        serde_json::Value::Number(n) => n.as_u64().expect("Failed to get u64 from number"),
        _ => panic!("Expected string or number for 'in' field"),
    };
    let input_scalar = BlsScalar::from(input_u64);
    let poseidon = Poseidon255::new();
    let output = poseidon.hash(&input_scalar);
    
    // Output hash as decimal to stdout
    let decimal_output = bls_scalar_to_decimal(output);
    println!("{}", decimal_output);
}
