use serde::Deserialize;
use std::fs;
use clap::Parser;
use num_bigint::BigUint;
use num_traits::Num;

#[derive(Parser)]
struct Args {
    filetype: String,
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

#[derive(Deserialize)]
struct ProofJson {
    pi_a: [String; 3],
    pi_b: [[String; 2]; 3],
    pi_c: [String; 3],
    #[serde(rename = "protocol")]
    _protocol: String,
    #[serde(rename = "curve")]
    _curve: String
}

// Remove the old PublicOutputJson struct and replace with type alias
type PublicOutputJson = Vec<String>;

fn print_vk(json_str: &String)
{
    let vk: VerificationKeyJson = serde_json::from_str(json_str).expect("Invalid JSON");

    println!("let alphax = \"{}\";", vk.vk_alpha_1[0]);
    println!("let alphay = \"{}\";", vk.vk_alpha_1[1]);
    println!("\n");
    println!("let betax1 = \"{}\";", vk.vk_beta_2[0][0]);
    println!("let betax2 = \"{}\";", vk.vk_beta_2[0][1]);
    println!("let betay1 = \"{}\";", vk.vk_beta_2[1][0]);
    println!("let betay2 = \"{}\";", vk.vk_beta_2[1][1]);
    println!("\n");
    println!("let gammax1 = \"{}\";", vk.vk_gamma_2[0][0]);
    println!("let gammax2 = \"{}\";", vk.vk_gamma_2[0][1]);
    println!("let gammay1 = \"{}\";", vk.vk_gamma_2[1][0]);
    println!("let gammay2 = \"{}\";", vk.vk_gamma_2[1][1]);
    println!("\n");
    println!("let deltax1 = \"{}\";", vk.vk_delta_2[0][0]);
    println!("let deltax2 = \"{}\";", vk.vk_delta_2[0][1]);
    println!("let deltay1 = \"{}\";", vk.vk_delta_2[1][0]);
    println!("let deltay2 = \"{}\";", vk.vk_delta_2[1][1]);
    println!("\n");
    println!("let ic0x = \"{}\";", vk.ic[0][0]);
    println!("let ic0y = \"{}\";", vk.ic[0][1]);
    println!("\n");
    println!("let ic1x = \"{}\";", vk.ic[1][0]);
    println!("let ic1y = \"{}\";", vk.ic[1][1]);
}

fn print_proof(json_str: &String) {
    let proof: ProofJson = serde_json::from_str(json_str).expect("Invalid JSON");

    println!("let pi_ax = \"{}\";", proof.pi_a[0]);
    println!("let pi_ay = \"{}\";", proof.pi_a[1]);
    println!("\n");
    println!("let pi_bx1 = \"{}\";", proof.pi_b[0][0]);
    println!("let pi_bx2 = \"{}\";", proof.pi_b[0][1]);
    println!("let pi_by1 = \"{}\";", proof.pi_b[1][0]);
    println!("let pi_by2 = \"{}\";", proof.pi_b[1][1]);
    println!("\n");
    println!("let pi_cx = \"{}\";", proof.pi_c[0]);
    println!("let pi_cy = \"{}\";", proof.pi_c[1]);
}

fn print_public_output(json_str: &String) {
    let public_output: PublicOutputJson = serde_json::from_str(json_str).expect("Invalid JSON");

    println!("// Public output signals:");
    for (i, signal) in public_output.iter().enumerate() {
        // Parse decimal string to BigUint
        let value = BigUint::from_str_radix(signal, 10).unwrap();
        let mut bytes = value.to_bytes_be();
        // Pad to 32 bytes
        if bytes.len() < 32 {
            let mut padded = vec![0u8; 32 - bytes.len()];
            padded.extend_from_slice(&bytes);
            bytes = padded;
        }
        // Format as hex for Rust array
        let bytes_str = bytes.iter().map(|b| format!("0x{:02x}", b)).collect::<Vec<_>>().join(", ");
        println!("let public_{} = U256::from_be_bytes(&env, &Bytes::from_array(&env, &[{}]));", i, bytes_str);
    }
    
    println!("\n// Create output vector for verification:");
    print!("let output = Vec::from_array(&env, [");
    for (i, _) in public_output.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("Fr::from_u256(public_{})", i);
    }
    println!("]);");
}

fn main() {
    let args = Args::parse();
    let json_str = fs::read_to_string(&args.filename).expect("Failed to read file");
    
    if args.filetype == "vk" {
        print_vk(&json_str);
    }

    if args.filetype == "proof" {
        print_proof(&json_str);
    }

    if args.filetype == "public" {
        print_public_output(&json_str);
    }
}