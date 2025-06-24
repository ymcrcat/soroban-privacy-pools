use ark_bls12_381::{G1Affine, G2Affine, Fq, Fq2};
use ark_serialize::CanonicalSerialize;
use core::str::FromStr;
use serde::Deserialize;
use std::fs;
use clap::Parser;

#[derive(Parser)]
struct Args {
    filename: String,
}

#[derive(Deserialize)]
struct VerificationKeyJson {
    vk_alpha_1: [String; 3],
    vk_beta_2: [[String; 2]; 3],
    vk_gamma_2: [[String; 2]; 3],
    vk_delta_2: [[String; 2]; 3],
    #[serde(rename = "IC")]
    ic: Vec<[String; 3]>,
}

fn g1_bytes(x: &str, y: &str) -> [u8; 48] {
    let p = G1Affine::new(Fq::from_str(x).unwrap(), Fq::from_str(y).unwrap());
    let mut buf = [0u8; 48];
    p.serialize_compressed(&mut buf[..]).unwrap();
    buf
}

fn g2_bytes(x1: &str, x2: &str, y1: &str, y2: &str) -> [u8; 96] {
    let x = Fq2::new(Fq::from_str(x1).unwrap(), Fq::from_str(x2).unwrap());
    let y = Fq2::new(Fq::from_str(y1).unwrap(), Fq::from_str(y2).unwrap());
    let p = G2Affine::new(x, y);
    let mut buf = [0u8; 96];
    p.serialize_compressed(&mut buf[..]).unwrap();
    buf
}

fn print_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("0x{:02x}", b)).collect::<Vec<_>>().join(", ")
}

fn main() {
    let args = Args::parse();
    let json_str = fs::read_to_string(&args.filename).expect("Failed to read file");
    let vk: VerificationKeyJson = serde_json::from_str(&json_str).expect("Invalid JSON");

    // Alpha
    let alpha_bytes = g1_bytes(&vk.vk_alpha_1[0], &vk.vk_alpha_1[1]);
    // Beta
    let beta_bytes = g2_bytes(
        &vk.vk_beta_2[0][0], &vk.vk_beta_2[0][1],
        &vk.vk_beta_2[1][0], &vk.vk_beta_2[1][1]
    );
    // Gamma
    let gamma_bytes = g2_bytes(
        &vk.vk_gamma_2[0][0], &vk.vk_gamma_2[0][1],
        &vk.vk_gamma_2[1][0], &vk.vk_gamma_2[1][1]
    );
    // Delta
    let delta_bytes = g2_bytes(
        &vk.vk_delta_2[0][0], &vk.vk_delta_2[0][1],
        &vk.vk_delta_2[1][0], &vk.vk_delta_2[1][1]
    );

    println!("VerificationKey {{");
    println!("    alpha: G1Affine::from_compressed(&env, &BytesN::from_array(&env, [{}])),", print_bytes(&alpha_bytes));
    println!("    beta: G2Affine::from_compressed(&env, &BytesN::from_array(&env, [{}])),", print_bytes(&beta_bytes));
    println!("    gamma: G2Affine::from_compressed(&env, &BytesN::from_array(&env, [{}])),", print_bytes(&gamma_bytes));
    println!("    delta: G2Affine::from_compressed(&env, &BytesN::from_array(&env, [{}])),", print_bytes(&delta_bytes));
    println!("    ic: Vec::from_array(&env, [");
    for ic in &vk.ic {
        let ic_bytes = g1_bytes(&ic[0], &ic[1]);
        println!("        G1Affine::from_compressed(&env, &BytesN::from_array(&env, [{}])),", print_bytes(&ic_bytes));
    }
    println!("    ]),");
    println!("}}");
}