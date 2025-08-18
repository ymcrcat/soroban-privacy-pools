#![allow(clippy::needless_borrow)]

use dusk_bls12_381::BlsScalar;
use dusk_poseidon::{Domain, Hash};
use soroban_sdk::{BytesN, Env};

/// Convert 32 bytes (little-endian) ↔ BlsScalar using Dusk’s canonical encoding.
/// If your existing BytesN<32> are big-endian, flip the arrays accordingly.
#[inline]
fn scalar_from_le_bytes(bytes: &BytesN<32>) -> BlsScalar {
    // Dusk expects LE; returns CtOption<BlsScalar>
    match BlsScalar::from_bytes(&bytes.to_array()) {
        // Safe unwrap for canonical inputs; if you feed random bytes, ensure reduction first.
        ct if bool::from(ct.is_some()) => ct.unwrap(),
        _ => {
            // Fallback: reduce a 512-bit little-endian integer.
            // (Only hit if input wasn’t canonical.)
            let mut wide = [0u8; 64];
            wide[..32].copy_from_slice(&bytes.to_array());
            BlsScalar::from_bytes_wide(&wide)
        }
    }
}

#[inline]
fn scalar_to_le_bytes(env: &Env, x: &BlsScalar) -> BytesN<32> {
    BytesN::from_array(env, &x.to_bytes()) // Dusk yields LE bytes
}

/// Poseidon parent for a binary Merkle tree (t=3 permutation; 2 inputs + capacity).
/// Input/output as BytesN<32> so it fits your LeanIMT storage.
pub fn poseidon2_bytes(env: &Env, left: &BytesN<32>, right: &BytesN<32>) -> BytesN<32> {
    let a = scalar_from_le_bytes(left);
    let b = scalar_from_le_bytes(right);

    // Prefer a Merkle-specific domain if present in your crate version; otherwise use Other.
    // If `Domain::Merkle2` doesn't exist for your version, swap to Domain::Other.
    let h = Hash::digest(Domain::Other, &[a, b]);

    scalar_to_le_bytes(env, &h[0])
}