use ark_bls12_381::Fr as BlsScalar;
use ark_ff::PrimeField;
use poseidon::Poseidon255;
use serde::Deserialize;
use std::io::{self, Read};
use num_bigint::BigUint;

#[derive(Deserialize)]
struct Input {
    #[serde(rename = "in1")]
    in1_value: serde_json::Value,
    #[serde(rename = "in2")]
    in2_value: serde_json::Value,
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
    let input1_scalar = match input_data.in1_value {
        serde_json::Value::String(s) => {
            // Parse the large number as a BigUint first
            let big_num = BigUint::parse_bytes(s.as_bytes(), 10)
                .expect("Failed to parse string to BigUint");
            // Convert BigUint to BlsScalar
            BlsScalar::from_be_bytes_mod_order(&big_num.to_bytes_be())
        },
        serde_json::Value::Number(n) => {
            if let Some(u64_val) = n.as_u64() {
                BlsScalar::from(u64_val)
            } else {
                // For numbers too large for u64
                let s = n.to_string();
                let big_num = BigUint::parse_bytes(s.as_bytes(), 10)
                    .expect("Failed to parse number to BigUint");
                BlsScalar::from_be_bytes_mod_order(&big_num.to_bytes_be())
            }
        },
        _ => panic!("Expected string or number for 'in1' field"),
    };
    
    let input2_scalar = match input_data.in2_value {
        serde_json::Value::String(s) => {
            // Parse the large number as a BigUint first
            let big_num = BigUint::parse_bytes(s.as_bytes(), 10)
                .expect("Failed to parse string to BigUint");
            // Convert BigUint to BlsScalar
            BlsScalar::from_be_bytes_mod_order(&big_num.to_bytes_be())
        },
        serde_json::Value::Number(n) => {
            if let Some(u64_val) = n.as_u64() {
                BlsScalar::from(u64_val)
            } else {
                // For numbers too large for u64
                let s = n.to_string();
                let big_num = BigUint::parse_bytes(s.as_bytes(), 10)
                    .expect("Failed to parse number to BigUint");
                BlsScalar::from_be_bytes_mod_order(&big_num.to_bytes_be())
            }
        },
        _ => panic!("Expected string or number for 'in2' field"),
    };
    
    let poseidon1 = Poseidon255::new();
    let output1 = poseidon1.hash(&input1_scalar);
    let decimal_output1 = bls_scalar_to_decimal(output1);

    let poseidon2 = Poseidon255::new();
    let output2 = poseidon2.hash_two(&input1_scalar, &input2_scalar);
    let decimal_output2 = bls_scalar_to_decimal(output2);

    println!("{}", decimal_output1);
    println!("{}", decimal_output2);
}
