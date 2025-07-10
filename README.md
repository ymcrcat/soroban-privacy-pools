# Soroban Privacy Pools

A privacy-preserving transaction system built on Stellar using Soroban smart contracts and zero-knowledge proofs (zkSNARKs). This project implements privacy pools that allow users to deposit and withdraw tokens while maintaining transaction privacy through cryptographic commitments and Merkle tree inclusion proofs.

## Features

- **Privacy-preserving transactions**: Deposit and withdraw tokens without revealing transaction history
- **Zero-knowledge proofs**: Uses Groth16 zkSNARKs with BLS12-381 curve for cryptographic verification
- **Commitment scheme**: Cryptographic commitments using Keccak256 hashing
- **Merkle tree inclusion**: Efficient proof of commitment inclusion in the pool state
- **Soroban integration**: Native integration with Stellar's smart contract platform

## Project Structure

```text
.
├── circuits/                    # Circom circuits for zero-knowledge proofs
│   ├── commitment.circom        # Commitment hashing logic
│   ├── dummy.circom             # Simplified circuit for testing
│   ├── main.circom              # Main withdrawal verification circuit
│   ├── keccak256.circom         # Keccak256 hash implementation
│   ├── merkleProof.circom       # Merkle tree inclusion proof
│   ├── build/                   # Compiled circuit artifacts
│   ├── input/                   # Test input files
│   └── output/                  # Generated keys and proofs
├── contracts/                   # Soroban smart contracts
│   └── privacy-pools/
│       ├── src/
│       │   ├── lib.rs           # Main contract logic
│       │   ├── test.rs          # Contract tests
│       │   └── zk/              # Zero-knowledge verification
│       │       ├── mod.rs       # ZK proof verification logic
│       │       └── test.rs      # ZK verification tests
│       ├── Cargo.toml
│       └── Makefile
├── circom2soroban/              # Utility for converting circom artifacts
│   ├── src/main.rs              # Converts VK/proofs/public to Soroban format
│   └── Cargo.toml
├── coinutils/                   # Utility functions and helpers
│   ├── src/main.rs
│   └── Cargo.toml
├── Cargo.toml                   # Workspace configuration
├── Makefile                     # Circuit compilation commands
└── README.md
```

## Prerequisites

- **Rust** (latest stable version)
- **Soroban CLI** for contract development
- **Node.js** and **npm** for circuit tools
- **Circom** circuit compiler
- **snarkjs** for proof generation and BLS12-381 support

### Install Dependencies

```bash
# Install circom
npm install -g circom

# Install snarkjs (with BLS12-381 support)
npm install -g snarkjs

# Install Soroban CLI
cargo install --locked soroban-cli
```

## Quick Start

### 1. Compile Circuits

```bash
# Compile all circuits with BLS12-381 curve
make .circuits
```

### 2. Generate Trusted Setup (BLS12-381)

The project uses BLS12-381 curve to match Soroban's native cryptographic primitives:

```bash
cd circuits

# Generate powers of tau for BLS12-381 (if not already available)
# Note: Use existing ceremony files or generate new ones
snarkjs powersoftau new bls12-381 20 output/pot20_0000.ptau -v
snarkjs powersoftau contribute output/pot20_0000.ptau output/pot20_0001.ptau --name="First contribution" -v
snarkjs powersoftau prepare phase2 output/pot20_0001.ptau output/pot20_final.ptau -v

# Generate circuit-specific setup for dummy circuit
snarkjs groth16 setup build/dummy.r1cs output/pot20_final.ptau output/dummy_0000.zkey

# Contribute to ceremony
snarkjs zkey contribute output/dummy_0000.zkey output/dummy_final.zkey --name="Your contribution" -e="random entropy"

# Export verification key
snarkjs zkey export verificationkey output/dummy_final.zkey output/dummy_verification_key.json

# For main circuit (if needed)
snarkjs groth16 setup build/main.r1cs output/pot20_final.ptau output/main_0000.zkey
snarkjs zkey contribute output/main_0000.zkey output/main_final.zkey --name="Your contribution" -e="random entropy"
snarkjs zkey export verificationkey output/main_final.zkey output/main_verification_key.json
```

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run specific contract tests
cargo test -p privacy-pools

# Run ZK verification tests
cargo test test_coin_ownership
```

## Usage

### Circuit Development

The project includes several Circom circuits:

- **`commitment.circom`**: Implements the commitment scheme using Keccak256
- **`dummy.circom`**: Simplified circuit for testing without Merkle tree verification
- **`main.circom`**: Full withdrawal circuit with Merkle tree inclusion proof
- **`merkleProof.circom`**: Lean Incremental Merkle Tree (LeanIMT) verification

### Smart Contract

The Soroban contract (`contracts/privacy-pools/`) implements:

- Deposit functionality with commitment generation
- Withdrawal with zero-knowledge proof verification using BLS12-381
- Nullifier tracking to prevent double-spending
- Integration with Soroban's native BLS12-381 curve operations

### Proof Generation and Conversion

```bash
# Generate witness
node build/dummy_js/generate_witness.js build/dummy_js/dummy.wasm input.json witness.wtns

# Generate proof
snarkjs groth16 prove output/dummy_final.zkey witness.wtns proof.json public.json

# Verify proof (snarkjs)
snarkjs groth16 verify output/dummy_verification_key.json public.json proof.json

# Convert verification key for Soroban
cargo run --bin circom2soroban vk output/dummy_verification_key.json

# Convert proof for Soroban
cargo run --bin circom2soroban proof proof.json

# Convert public outputs for Soroban
cargo run --bin circom2soroban public public.json
```

### circom2soroban Tool

The `circom2soroban` utility converts snarkjs artifacts to Soroban-compatible format:

```bash
# Convert verification key
cargo run --bin circom2soroban vk <verification_key.json>
# Outputs: Rust code with verification key coordinates

# Convert proof
cargo run --bin circom2soroban proof <proof.json>
# Outputs: Rust code with proof coordinates

# Convert public outputs
cargo run --bin circom2soroban public <public.json>
# Outputs: Rust code with public inputs as U256 and Fr conversion
```

Example output for public conversion:
```rust
// Public output signals:
let public_0 = U256::from_be_bytes(&env, &Bytes::from_array(&env, &[0x07, 0xf5, ...]));
let public_1 = U256::from_be_bytes(&env, &Bytes::from_array(&env, &[0x00, 0x09, ...]));

// Create output vector for verification:
let output = Vec::from_array(&env, [Fr::from_u256(public_0), Fr::from_u256(public_1)]);
```

## Development Workflow

1. **Modify circuits** in `circuits/` directory
2. **Recompile circuits** using `make .circuits`
3. **Regenerate trusted setup** if circuit structure changed
4. **Generate new proofs** with test inputs
5. **Convert artifacts** using `circom2soroban`
6. **Update tests** with new verification keys/proofs/public outputs
7. **Run tests** to verify everything works

## Testing

The project includes comprehensive tests:

- **Circuit tests**: Generate and verify proofs using snarkjs with BLS12-381
- **Contract tests**: Test deposit/withdrawal functionality
- **ZK verification tests**: Test proof verification in Soroban environment
- **Integration tests**: End-to-end privacy pool functionality

## Security Considerations

- **Trusted Setup**: The project uses Groth16 which requires a trusted setup ceremony for BLS12-381
- **Curve Consistency**: All components use BLS12-381 to ensure compatibility with Soroban
- **Circuit Auditing**: Circuits should be audited before production use
- **Key Management**: Verification keys must be properly validated
- **Nullifier Uniqueness**: Contract ensures nullifiers cannot be reused

## BLS12-381 Integration

This project is specifically designed to work with Soroban's BLS12-381 implementation:

- **Circuit compilation**: Uses `--prime bls12381` flag in circom
- **Trusted setup**: Powers of tau ceremony for BLS12-381 curve
- **Proof verification**: Native BLS12-381 operations in Soroban contracts
- **Field arithmetic**: All field operations use BLS12-381 scalar field
- **Point serialization**: Consistent coordinate representation between snarkjs and arkworks

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is open source. Please see the license file for details.
