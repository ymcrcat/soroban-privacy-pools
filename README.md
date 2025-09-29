# Soroban Privacy Pools

> **Warning**: This project is currently a work in progress and is under active development. It does not yet constitute a secure or functional system. Features and APIs may change.



A privacy-preserving transaction system built on Stellar using Soroban smart contracts and zero-knowledge proofs (zkSNARKs). This project implements privacy pools that allow users to deposit and withdraw tokens while maintaining transaction privacy through cryptographic commitments and Merkle tree inclusion proofs.

## Features

- **Privacy-preserving transactions**: Deposit and withdraw tokens without revealing transaction history
- **Zero-knowledge proofs**: Uses Groth16 zkSNARKs with BLS12-381 curve for cryptographic verification
- **Commitment scheme**: Cryptographic commitments using Poseidon hashing
- **Merkle tree inclusion**: Efficient proof of commitment inclusion in the pool state
- **Soroban integration**: Native integration with Stellar's smart contract platform

## Project Structure

```
.
├── circuits/                 # Circom circuits for zero-knowledge proofs
│   ├── commitment.circom     # Commitment hashing logic
│   ├── main.circom           # Main withdrawal verification circuit
│   ├── merkleProof.circom    # Merkle tree inclusion proof
│   ├── poseidon.circom       # Poseidon hash implementation
│   ├── dummy.circom          # Simplified circuit for testing
│   ├── build/                # Compiled circuit artifacts
│   ├── input/                # Test input files
│   └── output/               # Generated keys and proofs
├── contracts/                # Soroban smart contracts
│   └── privacy-pools/
│       ├── src/
│       │   ├── lib.rs        # Main contract logic
│       │   └── test.rs       # Contract tests
│       ├── Cargo.toml
│       └── Makefile
├── zk/                       # Zero-knowledge verification library
│   ├── src/
│   │   ├── lib.rs            # ZK proof verification logic
│   │   └── test.rs           # ZK verification tests
│   └── Cargo.toml
├── circom2soroban/           # Utility for converting circom artifacts
│   ├── src/main.rs           # Converts VK/proofs/public to Soroban format
│   └── Cargo.toml
├── coinutils/                # Coin generation and management utility
│   ├── src/main.rs           # CLI for generating coins and withdrawal inputs
│   └── Cargo.toml
├── Cargo.toml                # Workspace configuration
├── Makefile                  # Circuit compilation commands
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

# Generate circuit-specific setup for main circuit
snarkjs groth16 setup build/main.r1cs output/pot20_final.ptau output/main_0000.zkey

# Contribute to ceremony
snarkjs zkey contribute output/main_0000.zkey output/main_final.zkey --name="Your contribution" -e="random entropy"

# Export verification key
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

- **`commitment.circom`**: Implements the commitment scheme using Poseidon hashing
- **`main.circom`**: Full withdrawal circuit with Merkle tree inclusion proof
- **`merkleProof.circom`**: Lean Incremental Merkle Tree (LeanIMT) verification
- **`dummy.circom`**: Simplified circuit for testing without Merkle tree verification

### Smart Contract

The Soroban contract (`contracts/privacy-pools/`) implements:

- Deposit functionality with commitment generation
- Withdrawal with zero-knowledge proof verification using BLS12-381
- Nullifier tracking to prevent double-spending
- Integration with Soroban's native BLS12-381 curve operations

### Proof Generation and Conversion

```bash
# Generate witness
node build/main_js/generate_witness.js build/main_js/main.wasm input.json witness.wtns

# Generate proof
snarkjs groth16 prove output/main_final.zkey witness.wtns proof.json public.json

# Verify proof (snarkjs)
snarkjs groth16 verify output/main_verification_key.json public.json proof.json

# Convert verification key for Soroban
cargo run --bin circom2soroban vk output/main_verification_key.json

# Convert proof for Soroban
cargo run --bin circom2soroban proof proof.json

# Convert public outputs for Soroban
cargo run --bin circom2soroban public public.json
```

### coinutils Tool

The `coinutils` utility helps generate and manage privacy pool coins with proper cryptographic commitments:

```bash
# Generate a new coin for a privacy pool
cargo run --bin coinutils generate <scope> [output_file]

# Create withdrawal inputs from an existing coin (requires state file)
cargo run --bin coinutils withdraw <coin_file> <state_file> [output_file]
```

**Features:**
- **Coin Generation**: Creates new coins with random nullifiers and secrets
- **Commitment Calculation**: Implements the same commitment scheme as the circuits
- **Merkle Tree Integration**: Uses lean-imt for consistent merkle tree operations
- **Withdrawal Preparation**: Generates circuit inputs with merkle tree proofs
- **BLS12-381 Field Operations**: Uses arkworks for proper field arithmetic
- **Poseidon Hashing**: Matches the circuit's hash implementation

**Examples:**
```bash
# Generate a coin for "my_pool" scope
cargo run --bin coinutils generate my_pool coin.json

# Create state file with commitments
echo '{"commitments": ["commitment1", "commitment2", "..."], "scope": "my_pool"}' > state.json

# Create withdrawal from existing coin with state file
cargo run --bin coinutils withdraw coin.json state.json withdrawal.json
```

**Generated Coin Structure:**
```json
{
  "coin": {
    "value": "1000000000",         
    "nullifier": "12345...",     
    "secret": "67890...",       
    "label": "24680...",       
    "commitment": "13579..."
  },
  "commitment_hex": "0xabcd..."
}
```

**State File Structure:**
```json
{
  "commitments": [
    "commitment1_hash",
    "commitment2_hash",
    "commitment3_hash"
  ],
  "scope": "pool_scope"
}
```

**Withdrawal Input Structure:**
```json
{
  "withdrawnValue": "1000000000",
  "label": "24680...",
  "value": "1000000000",
  "nullifier": "12345...",
  "secret": "67890...",
  "stateRoot": "merkle_root_hash",
  "stateIndex": "1",
  "stateSiblings": [
    "sibling1_hash",
    "sibling2_hash",
    "0", "0", "0", "0", "0", "0", "0", "0",
    "0", "0", "0", "0", "0", "0", "0", "0", "0", "0"
  ]
}
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
4. **Generate test coins** using `coinutils generate`
5. **Create withdrawal inputs** using `coinutils withdraw`
6. **Generate new proofs** with circuit inputs
7. **Convert artifacts** using `circom2soroban`
8. **Update tests** with new verification keys/proofs/public outputs
9. **Run tests** to verify everything works

### Complete Example Workflow

```bash
# 1. Generate a test coin
cargo run --bin coinutils generate test_pool_scope test_coin.json

# 2. Create state file with commitments (including the generated coin's commitment)
COMMITMENT=$(cat test_coin.json | jq -r '.coin.commitment')
echo "{
  \"commitments\": [
    \"$COMMITMENT\"
  ],
  \"scope\": \"test_pool_scope\"
}" > state.json

# 3. Create withdrawal input from the coin with state file
cargo run --bin coinutils withdraw test_coin.json state.json withdrawal_input.json

# 4. Generate witness and proof
cd circuits
node build/main_js/generate_witness.js build/main_js/main.wasm ../withdrawal_input.json witness.wtns
snarkjs groth16 prove output/main_final.zkey witness.wtns proof.json public.json

# 5. Convert for Soroban
cd ..
cargo run --bin circom2soroban vk circuits/output/main_verification_key.json
cargo run --bin circom2soroban proof circuits/proof.json
cargo run --bin circom2soroban public circuits/public.json

# 6. Update test with new values and run
cargo test test_coin_ownership
```

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

## Building and Deploying the Privacy-Pools contract

```bash
# Generate a key for the deployer
soroban keys generate --global alice --network <NETWORK>

# Fund the deployer address
soroban keys fund alice --network <NETWORK>
```

In the workspace directory run

```bash
# Build the contract
cargo build --target wasm32v1-none --release -p privacy-pools

# Optimize the WASM for Soroban
soroban contract optimize --wasm target/wasm32v1-none/release/privacy_pools.wasm --wasm-out target/wasm32v1-none/release/privacy_pools.optimized.wasm

# Deploy the contract to the testnet passing a Hex-encoded verification key as an argument to the constructor
soroban contract deploy --wasm target/wasm32v1-none/release/privacy_pools.optimized.wasm --source alice --network <NETWORK> -- --vk_bytes <VK_BYTES_HEX>
```

**Note:** The constructor argument is passed as a Hex-encoding of a byte array, but without the `0x` prefix.

To deposit into the contract run

```bash
soroban contract invoke --id <CONTRACT_ID> --source alice --network <NETWORK> -- deposit --from alice --commitment <COMMITMENT_HEX>
```

and to withdraw

```bash
soroban contract invoke --id <CONTRACT_ID> --source alice --network <NETWORK> -- withdraw --to alice --proof_bytes <PROOF_BYTES_HEX> --pub_signals_bytes <PUBLIC_OUTPUT_HEX>
```

## Demo: Complete Privacy Pool Workflow

This demo walks through the complete lifecycle of a privacy pool transaction, from coin generation to withdrawal with zero-knowledge proofs.

### Prerequisites

Before running the demo, ensure you have:

1. **Compiled circuits** (see Quick Start section)
2. **Generated trusted setup** for the main circuit
3. **Soroban CLI** configured with a testnet account
4. **Built the contract** and utilities

```bash
# Ensure circuits are compiled
make .circuits

# Build all utilities
cargo build --release

# Set up Soroban testnet account
soroban keys generate --global demo_user --network testnet
soroban keys fund demo_user --network testnet
```

### Step 1: Deploy the Privacy-Pools Contract

First, we need to deploy the contract with the verification key:

```bash
# Build the contract
cargo build --target wasm32v1-none --release -p privacy-pools

# Optimize the WASM
soroban contract optimize --wasm target/wasm32v1-none/release/privacy_pools.wasm --wasm-out target/wasm32v1-none/release/privacy_pools.optimized.wasm

# Convert verification key to hex format and extract it
cargo run --bin circom2soroban vk circuits/output/main_verification_key.json > vk_hex.txt
VK_HEX=$(cat vk_hex.txt | grep -o '[0-9a-f]*$')

# Deploy the contract
soroban contract deploy --wasm target/wasm32v1-none/release/privacy_pools.optimized.wasm --source demo_user --network testnet --instructions 10000000 --fee 50000000 -- --vk_bytes $VK_HEX

# Save the contract ID for later use
export CONTRACT_ID=<CONTRACT_ID_FROM_DEPLOYMENT>
```

### Step 2: Generate a Coin

Create a new coin with a random nullifier and secret:

```bash
# Generate a coin for the demo pool
cargo run --bin coinutils generate demo_pool demo_coin.json

# View the generated coin
cat demo_coin.json
```

The generated coin contains:
- `value`: The coin's value (e.g., 1000000000 for 1 token)
- `nullifier`: Unique identifier to prevent double-spending
- `secret`: Random secret for commitment generation
- `label`: Additional random data
- `commitment`: Cryptographic commitment hash

### Step 3: Deposit the Coin into the Contract

Use the commitment from the generated coin to deposit:

```bash
# Extract the commitment hex from the coin file (remove 0x prefix)
COMMITMENT_HEX=$(cat demo_coin.json | jq -r '.commitment_hex' | sed 's/^0x//')

# Deposit the coin into the contract
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet --instructions 4000000 -- deposit --from demo_user --commitment $COMMITMENT_HEX
```

### Step 4: Check the Balance

Verify the deposit was successful by checking the contract state:

```bash
# Check the contract balance
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet -- get_balance

# Check the list of commitments
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet -- get_commitments
```

### Step 5: Create State File and Withdrawal Proof

First, create a state file with the commitment from the generated coin, then generate the withdrawal inputs and create a zero-knowledge proof:

```bash
# Create state file with the coin's commitment
COMMITMENT=$(cat demo_coin.json | jq -r '.coin.commitment')
echo "{
  \"commitments\": [
    \"$COMMITMENT\"
  ],
  \"scope\": \"demo_pool\"
}" > demo_state.json

# Create withdrawal inputs from the coin with state file
cargo run --bin coinutils withdraw demo_coin.json demo_state.json withdrawal_input.json

# Generate witness and proof using the main circuit
cd circuits
node build/main_js/generate_witness.js build/main_js/main.wasm ../withdrawal_input.json witness.wtns
snarkjs groth16 prove output/main_final.zkey witness.wtns proof.json public.json

# Convert proof and public signals for Soroban
cd ..
cargo run --bin circom2soroban proof circuits/proof.json > proof_hex.txt
cargo run --bin circom2soroban public circuits/public.json > public_hex.txt

# Extract the hex strings (without 0x prefix)
PROOF_HEX=$(sed -n '/^Proof Hex encoding:/{n;p;}' proof_hex.txt | tr -d '[:space:]' | sed -E 's/^0x//i')
PUBLIC_HEX=$(cat public_hex.txt | grep -o '[0-9a-f]*$')

echo "Generated proof: $PROOF_HEX"
echo "Public signals: $PUBLIC_HEX"
```

### Step 6: Withdraw from the Contract

Use the proof to withdraw the coin:

```bash
# Withdraw using the proof
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet --fee 1000000 --instructions 50000000 -- withdraw --to demo_user --proof_bytes $PROOF_HEX --pub_signals_bytes $PUBLIC_HEX

echo "Successfully withdrew coin"
```

### Step 7: Verify the Withdrawal

Check that the withdrawal was successful and the nullifier is recorded:

```bash
# Check the list of used nullifiers
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet -- get_nullifiers

# Check the updated contract balance
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet -- get_balance

# The nullifier should now appear in the list, indicating it has been spent
```

### Complete Demo Script

Here's a complete script that automates the entire demo:

```bash
#!/bin/bash
set -e

echo "🚀 Starting Privacy Pool Demo..."

# Check prerequisites
echo "🔍 Checking prerequisites..."
command -v jq >/dev/null 2>&1 || { echo "❌ Error: jq is required but not installed. Please install jq first."; exit 1; }
command -v soroban >/dev/null 2>&1 || { echo "❌ Error: soroban CLI is required but not installed."; exit 1; }

# Step 1: Deploy contract
echo "📦 Deploying contract..."
cargo build --target wasm32v1-none --release -p privacy-pools || { echo "❌ Error: Failed to build contract"; exit 1; }
soroban contract optimize --wasm target/wasm32v1-none/release/privacy_pools.wasm --wasm-out target/wasm32v1-none/release/privacy_pools.optimized.wasm || { echo "❌ Error: Failed to optimize WASM"; exit 1; }

# Convert verification key to hex format and extract it
echo "🔑 Converting verification key..."
cargo run --bin circom2soroban vk circuits/output/main_verification_key.json > vk_hex.txt || { echo "❌ Error: Failed to convert verification key"; exit 1; }
VK_HEX=$(cat vk_hex.txt | grep -o '[0-9a-f]*$')
if [ -z "$VK_HEX" ]; then
    echo "❌ Error: Failed to extract verification key hex"
    exit 1
fi

echo "🚀 Deploying contract to testnet..."
soroban contract deploy --wasm target/wasm32v1-none/release/privacy_pools.optimized.wasm --source demo_user --network testnet -- --vk_bytes $VK_HEX || { echo "❌ Error: Failed to deploy contract"; exit 1; }

# Save the contract ID for later use
echo ""
echo "📋 Please paste the contract ID from the deployment above:"
read CONTRACT_ID
if [ -z "$CONTRACT_ID" ]; then
    echo "❌ Error: No contract ID provided"
    exit 1
fi
echo "✅ Contract ID set to: $CONTRACT_ID"

# Step 2: Generate coin
echo "🪙 Generating coin..."
cargo run --bin coinutils generate demo_pool demo_coin.json || { echo "❌ Error: Failed to generate coin"; exit 1; }
COMMITMENT_HEX=$(cat demo_coin.json | jq -r '.commitment_hex' | sed 's/^0x//')
if [ -z "$COMMITMENT_HEX" ]; then
    echo "❌ Error: Failed to extract commitment hex"
    exit 1
fi
echo "Generated coin with commitment: $COMMITMENT_HEX"

# Step 3: Deposit
echo "💰 Depositing coin..."
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet -- deposit --from demo_user --commitment $COMMITMENT_HEX || { echo "❌ Error: Failed to deposit coin"; exit 1; }
echo "Deposit successful!"

# Step 4: Check balance
echo "📊 Checking balance..."
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet -- get_balance || { echo "❌ Error: Failed to get balance"; exit 1; }

# Step 5: Create state file and withdrawal proof
echo "📋 Creating state file..."
COMMITMENT=$(cat demo_coin.json | jq -r '.coin.commitment')
echo "{
  \"commitments\": [
    \"$COMMITMENT\"
  ],
  \"scope\": \"demo_pool\"
}" > demo_state.json

echo "🔐 Creating withdrawal proof..."
cargo run --bin coinutils withdraw demo_coin.json demo_state.json withdrawal_input.json || { echo "❌ Error: Failed to create withdrawal input"; exit 1; }

echo "📝 Generating witness and proof..."
cd circuits
node build/main_js/generate_witness.js build/main_js/main.wasm ../withdrawal_input.json witness.wtns || { echo "❌ Error: Failed to generate witness"; exit 1; }
snarkjs groth16 prove output/main_final.zkey witness.wtns proof.json public.json || { echo "❌ Error: Failed to generate proof"; exit 1; }
cd ..

echo "🔄 Converting proof for Soroban..."
cargo run --bin circom2soroban proof circuits/proof.json > proof_hex.txt || { echo "❌ Error: Failed to convert proof"; exit 1; }
cargo run --bin circom2soroban public circuits/public.json > public_hex.txt || { echo "❌ Error: Failed to convert public signals"; exit 1; }

PROOF_HEX=$(grep "Proof Hex encoding:" proof_hex.txt | sed 's/.*Proof Hex encoding://' | tr -d '[:space:]')
PUBLIC_HEX=$(grep "Public signals Hex encoding:" public_hex.txt | sed 's/.*Public signals Hex encoding://' | tr -d '[:space:]')

if [ -z "$PROOF_HEX" ] || [ -z "$PUBLIC_HEX" ]; then
    echo "❌ Error: Failed to extract proof or public signals"
    exit 1
fi

# Step 6: Withdraw
echo "💸 Withdrawing coin..."
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet -- withdraw --to demo_user --proof_bytes $PROOF_HEX --pub_signals_bytes $PUBLIC_HEX || { echo "❌ Error: Failed to withdraw coin"; exit 1; }
echo "Withdrawal successful!"

# Step 7: Verify
echo "✅ Verifying withdrawal..."
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet -- get_nullifiers || { echo "❌ Error: Failed to get nullifiers"; exit 1; }
soroban contract invoke --id $CONTRACT_ID --source demo_user --network testnet -- get_balance || { echo "❌ Error: Failed to get final balance"; exit 1; }

echo "🎉 Demo completed successfully!"
```

### Demo Output Example

When running the demo, you should see output similar to:

```bash
🚀 Starting Privacy Pool Demo...
📦 Deploying contract...
Contract deployed: CABC123...
🪙 Generating coin...
Generated coin with commitment: abcd1234...
💰 Depositing coin...
Deposit successful!
📊 Checking balance...
1000000000
🔐 Creating withdrawal proof...
💸 Withdrawing coin...
Withdrawal successful!
✅ Verifying withdrawal...
[nullifier_list]
0
🎉 Demo completed successfully!
```


## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is open source. Please see the license file for details.
