use ark_bls12_381::{Fq, Fq2};
use ark_serialize::CanonicalSerialize;
use core::str::FromStr;
use soroban_sdk::{
    crypto::bls12_381::{G1Affine, G2Affine, G1_SERIALIZED_SIZE, G2_SERIALIZED_SIZE}, 
    Env
};

fn g1_from_coords(env: &Env, x: &str, y: &str) -> G1Affine {
    let ark_g1 = ark_bls12_381::G1Affine::new(Fq::from_str(x).unwrap(), Fq::from_str(y).unwrap());
    let mut buf = [0u8; G1_SERIALIZED_SIZE];
    ark_g1.serialize_uncompressed(&mut buf[..]).unwrap();
    G1Affine::from_array(env, &buf)
}

fn g2_from_coords(env: &Env, x1: &str, x2: &str, y1: &str, y2: &str) -> G2Affine {
    let x = Fq2::new(Fq::from_str(x1).unwrap(), Fq::from_str(x2).unwrap());
    let y = Fq2::new(Fq::from_str(y1).unwrap(), Fq::from_str(y2).unwrap());
    let ark_g2 = ark_bls12_381::G2Affine::new(x, y);
    let mut buf = [0u8; G2_SERIALIZED_SIZE];
    ark_g2.serialize_uncompressed(&mut buf[..]).unwrap();
    G2Affine::from_array(env, &buf)
}

fn to_rust_bytes_literal(bytes: &[u8]) -> String {
    let inner = bytes.iter().map(|b| format!("0x{:02x}", b)).collect::<Vec<_>>().join(", ");
    format!("&[{}]", inner)
}

fn main() {
    let env = Env::default();

    // Example inputs
    let g1 = g1_from_coords(&env, "851850525556173310373115880154698084608631105506432893865500290442025919078535925294035153152030470398262539759609", 
         "2637289349983507610125993281171282870664683328789064436670091381805667870657250691837988574635646688089951719927247");
    let g2 = g2_from_coords(&env, "1312620381151154625549413690218290437739613987001512553647554932245743783919690104921577716179019375920325686841943",
        "1853421227732662200477195678252233549930451033531229987959164216695698667330234953033341200627605777603511819497457",
        "3215807833988244618006117550809420301978856703407297742347804415291049013404133666905173282837707341742014140541018",
        "812366606879346135498483310623227330050424196838294715759414425317592599094348477520229174120664109186562798527696");

    let g1_bytes = g1.to_array();
    let g2_bytes = g2.to_array();

    println!("let g1 = G1Affine::from_compressed(&env, &BytesN::from_array(&env, {}));", to_rust_bytes_literal(&g1_bytes));
    println!("let g2 = G2Affine::from_compressed(&env, &BytesN::from_array(&env, {}));", to_rust_bytes_literal(&g2_bytes));
}