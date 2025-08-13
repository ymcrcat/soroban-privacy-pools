# LeanIMT ↔ MerkleProof.circom Compatibility Test

This directory contains tests to verify compatibility between the `lean-imt` Rust crate and the `merkleProof.circom` circuit.

## Overview

The test demonstrates that:
1. The `lean-imt` Rust crate can build a merkle tree with the same structure as expected by the circom circuit
2. Generated proofs from `lean-imt` can be used as input for the `merkleProof.circom` circuit
3. The circuit can successfully generate witnesses from the `lean-imt` proof data

## Files

- `test_lean_imt_compatibility.rs` - Rust test that uses the actual `lean-imt` crate
- `Cargo.toml` - Rust project configuration with `lean-imt` dependency
- `run_compatibility_test.sh` - Shell script to run the complete compatibility test
- `test_input_leaf_*.json` - Generated JSON input files for the circom circuit
- `witness_leaf_*.wtns` - Generated witness files (after running the script)

## Usage

### Quick Test

Run the Rust test to generate JSON input files:

```bash
cargo run --bin test_lean_imt_compatibility
```

This will:
- Build a merkle tree using the `lean-imt` crate
- Insert 4 test leaves
- Generate proofs for each leaf
- Create JSON input files for the circom circuit

### Complete Compatibility Test

Run the full compatibility test including circuit compilation and witness generation:

```bash
./run_compatibility_test.sh
```

This script will:
1. Generate JSON input files using `lean-imt`
2. Compile the `merkleProof.circom` circuit
3. Generate witnesses for each test case
4. Verify that witnesses are created successfully

### Manual Steps

If you prefer to run steps manually:

1. **Generate test data:**
   ```bash
   cargo run --bin test_lean_imt_compatibility
   ```

2. **Compile the circuit:**
   ```bash
   circom ../merkleProof.circom --r1cs --wasm --sym -o ../build -l /opt/homebrew/lib/node_modules/circomlib/circuits --prime bls12381
   ```

3. **Generate witnesses:**
   ```bash
   node ../build/merkleProof_js/generate_witness.js ../build/merkleProof_js/merkleProof.wasm test_input_leaf_0.json witness_leaf_0.wtns
   ```

## Test Data

The test creates a merkle tree with 4 leaves:
- Leaf 0: `0x0101010101010101010101010101010101010101010101010101010101010101`
- Leaf 1: `0x0202020202020202020202020202020202020202020202020202020202020202`
- Leaf 2: `0x0303030303030303030303030303030303030303030303030303030303030303`
- Leaf 3: `0x0404040404040404040404040404040404040404040404040404040404040404`

The tree has depth 2 and generates proofs with 3 siblings for each leaf.

## JSON Input Format

The generated JSON files have the following structure:

```json
{
  "leaf": "0x0101010101010101010101010101010101010101010101010101010101010101",
  "leafIndex": 0,
  "siblings": [
    "0x0202020202020202020202020202020202020202020202020202020202020202",
    "0x0202020202020202020202020202020202020202020202020202020202020202",
    "0x0404040404040404040404040404040404040404040404040404040404040404"
  ],
  "actualDepth": 2
}
```

## Verification

The test verifies:
- Tree structure matches expected depth and leaf count
- Proof generation produces the correct number of siblings
- JSON output format matches circom circuit expectations
- Witness generation succeeds without errors

## Integration with Privacy Pools

This test validates that the `lean-imt` crate used in the privacy pools contract (`contracts/privacy-pools/src/lib.rs`) is compatible with the zero-knowledge proof circuit (`circuits/merkleProof.circom`).

When a user deposits funds and gets a leaf index, they can:
1. Use the leaf index to generate a proof using the contract's merkle tree
2. Use the proof data to create a valid witness for the circom circuit
3. Generate a zero-knowledge proof for withdrawal

## Dependencies

- Rust with Cargo
- `lean-imt` crate (local dependency)
- `soroban-sdk` (workspace dependency)
- `serde_json` for JSON serialization
- `circom` for circuit compilation
- `node.js` for witness generation

## Troubleshooting

### Circom not found
Install circom: https://docs.circom.io/getting-started/installation/

### Compilation errors
Ensure all dependencies are installed:
```bash
cargo check
```

### Witness generation fails
Check that the JSON input format matches the circuit expectations and that the circuit compiled successfully.
