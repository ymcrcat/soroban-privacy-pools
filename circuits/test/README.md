# MerkleProof Circuit Test Suite

This directory contains comprehensive tests for the `merkleProof.circom` circuit, which implements a Lean Incremental Merkle Tree proof verification system.

## Overview

The `merkleProof.circom` circuit verifies that a given leaf exists in a merkle tree by checking the provided merkle proof. It follows the LeanIMT design principles:

1. Every node with two children is the hash of its left and right nodes
2. Every node with one child has the same value as its child node
3. Tree is always built from leaves to root
4. Tree is always balanced by construction
5. Tree depth is dynamic and can increase with insertion of new leaves

## Test Files

### 1. `test/test_merkleProof.circom`
The main test circuit file that includes multiple test templates:
- `MerkleProofTest`: Full test with depth 4
- `MerkleProofSimpleTest`: Simple test with depth 2
- `MerkleProofEdgeCaseTest`: Edge case tests with depth 3

### 2. `test/test_merkleProof.js`
JavaScript test data generator that creates various test scenarios:
- Test 1: 2-leaf tree (depth 1)
- Test 2: 4-leaf tree (depth 2)
- Test 3: 8-leaf tree (depth 3)
- Test 4: Single leaf (depth 0)
- Test 5: Leftmost leaf in 8-leaf tree

### 3. `package.json`
Dependencies and scripts for running the tests.

## Test Scenarios

### Basic Functionality Tests
- **Valid Proofs**: Tests that correctly constructed merkle proofs are accepted
- **Different Tree Depths**: Tests trees of various depths (0, 1, 2, 3)
- **Leaf Positions**: Tests leaves at different positions in the tree

### Edge Cases
- **Single Leaf Trees**: Trees with only one leaf (depth 0)
- **Unbalanced Trees**: Trees where some levels have odd numbers of nodes
- **Empty Siblings**: Handles cases where sibling nodes are empty (value 0)

### Negative Test Cases (Should Fail)
- **Invalid Siblings**: Tests with incorrect sibling values that should cause witness generation to fail
- **Wrong Leaf Index**: Tests with incorrect leaf indices that should violate circuit constraints
- **Wrong Depth**: Tests with incorrect tree depth that should cause constraint violations
- **Wrong Expected Root**: Tests with incorrect expected root values that should fail verification

### Error Cases (Implicit)
- **Invalid Proofs**: The circuit will fail if provided with incorrect sibling values
- **Depth Mismatch**: The circuit enforces that actual depth ≤ max depth
- **Index Validation**: Leaf index must be valid for the given tree structure

## Getting Started

### Prerequisites
- Node.js (v14 or higher)
- circom compiler
- circomlib (for standard components)

### Installation
```bash
cd circuits
npm install
```

### Running Tests

#### 1. Generate Test Data
```bash
npm run test
```
This will generate test data and save it to JSON files.

#### 2. Compile Test Circuit
```bash
npm run compile
```
This compiles the test circuit to R1CS, WASM, and symbol files.

#### 3. Generate Witness
```bash
npm run generate-witness
```
This generates a witness file from the test inputs.

## Test Data Structure

Each test case contains:
```json
{
  "leaf": <leaf_value>,
  "leafIndex": <index_in_tree>,
  "siblings": [<sibling_values>],
  "actualDepth": <tree_depth>,
  "expectedRoot": <computed_root>
}
```

## Understanding the Tests

### Test 1: 2-leaf Tree
```
     Root
    /    \
  Hash   Hash
 /        \
1          2
```
- Tests basic hashing of two leaves
- Depth: 1
- Siblings: [2] (for leaf 1)

### Test 2: 4-leaf Tree
```
        Root
       /    \
    Hash    Hash
   /    \   /    \
  1     2  3      4
```
- Tests balanced tree with 4 leaves
- Depth: 2
- Siblings: [2, Hash(3,4)] (for leaf 1)

### Test 3: 8-leaf Tree
```
            Root
           /    \
      Hash      Hash
     /    \     /    \
   Hash  Hash Hash  Hash
  /  \   /  \ /  \   /  \
  1  2  3  4 5  6  7  8
```
- Tests larger tree with 8 leaves
- Depth: 3
- Complex sibling path calculations

## Circuit Verification

The test circuit verifies that:
1. The merkle proof is correctly computed
2. The output matches the expected root
3. All constraints are satisfied

## Negative Testing

The test suite includes negative test cases that verify the circuit properly rejects invalid proofs:

### How Negative Tests Work
1. **Generate Valid Proofs**: First, create mathematically correct merkle proofs
2. **Corrupt the Proofs**: Intentionally modify specific values to make proofs invalid
3. **Test Failure**: Verify that witness generation fails for invalid proofs
4. **Circuit Security**: Ensure the circuit cannot be tricked with incorrect data

### Types of Invalid Proofs Tested
- **Wrong Sibling Values**: Incorrect sibling hashes that don't match the tree structure
- **Wrong Leaf Index**: Leaf index that doesn't correspond to the provided siblings
- **Wrong Tree Depth**: Depth value that doesn't match the actual tree structure
- **Wrong Expected Root**: Expected root that doesn't match the computed root

### Expected Behavior
- ✅ **Valid Proofs**: Should generate witnesses successfully
- ❌ **Invalid Proofs**: Should fail witness generation with constraint violations
- 🔒 **Security**: Circuit should be impossible to satisfy with incorrect data

## Troubleshooting

### Common Issues
1. **Compilation Errors**: Ensure circom and circomlib are properly installed
2. **Witness Generation Errors**: Check that input JSON matches the expected format
3. **Constraint Violations**: Verify that test data is mathematically correct

### Debugging
- Use the generated JSON files to verify test data manually
- Check that sibling values are correctly computed
- Verify tree depth calculations

## Extending the Tests

To add new test cases:
1. Add a new test method in `test_merkleProof.js`
2. Generate appropriate test data
3. Add the test to the `generateAllTests()` method
4. Update the main function to include the new test

## Performance Considerations

- Test with small tree depths first (≤ 4)
- Larger trees require more computation time
- Consider using different prime fields for testing vs production

## Security Notes

- These are test circuits, not production circuits
- Always verify proofs in production environments
- Test edge cases thoroughly before deployment

## Poseidon Compatibility Test

This directory also contains a compatibility test for the `poseidon255.circom` circuit, which verifies that the Rust `poseidon` crate produces identical hash outputs to the Circom implementation.

### Overview

The Poseidon compatibility test ensures consistency between:
1. **Rust Implementation**: `poseidon` crate with `Poseidon255` implementation
2. **Circom Implementation**: `poseidon255.circom` circuit

This is crucial for maintaining consistency between the Rust implementation (used in the main application) and the Circom implementation (used in zero-knowledge proofs).

### Test Files

#### 1. `test/test_poseidon.circom`
The main test circuit that includes:
- `TestPoseidon`: Tests both single and two-input hashing
- **Input signals**: `in1`, `in2`
- **Output signals**: `out1` (single input hash), `out2` (two input hash)
- **Components**: 
  - `Poseidon255(1)` for single input hashing
  - `Poseidon255(2)` for two input hashing

#### 2. `test/poseidon-test/`
Rust test implementation that:
- Uses the `poseidon` crate with `Poseidon255` implementation
- Takes JSON input with two values: `in1` and `in2`
- Computes the same hashes as the circuit

#### 3. `test/test_poseidon_input.json`
Test input data:
```json
{
    "in1": "123456789",
    "in2": "0"
}
```

### Running the Poseidon Compatibility Test

#### Prerequisites
- Rust toolchain (cargo)
- Node.js and npm
- circom compiler
- snarkjs

#### Step 1: Generate Witness from Rust Implementation
```bash
cd circuits/test
cat test_poseidon_input.json | cargo run --bin test_poseidon --manifest-path poseidon-test/Cargo.toml
```

**Expected Output:**
```
49771379518533783451081444171936304251693849153677701053778138403868110038125
2595333311380081774082696984545715941782212075692277571540746075566179600420
```

#### Step 2: Generate Witness from Circom Circuit
```bash
cd circuits/build/test_poseidon_js
node generate_witness.js test_poseidon.wasm ../../test/test_poseidon_input.json ../../test/test_poseidon_new.wtns
```

#### Step 3: Extract Witness Outputs
```bash
cd ../../test
snarkjs wtns export json test_poseidon_new.wtns
```

#### Step 4: Verify Compatibility
Compare the outputs from both implementations:

| Hash Type | Rust Output | Circom Output | Status |
|-----------|-------------|---------------|---------|
| Single Input (`in1`) | `49771379518533783451081444171936304251693849153677701053778138403868110038125` | `49771379518533783451081444171936304251693849153677701053778138403868110038125` | ✅ **MATCH** |
| Two Inputs (`in1`, `in2`) | `2595333311380081774082696984545715941782212075692277571540746075566179600420` | `2595333311380081774082696984545715941782212075692277571540746075566179600420` | ✅ **MATCH** |

### Test Circuit Structure

The `test_poseidon.circom` circuit:
- **Input signals**: `in1`, `in2`
- **Output signals**: `out1`, `out2`
- **Public signals**: All inputs and outputs are public for testing purposes
- **Hash functions**: Uses the same Poseidon255 parameters as the Rust implementation

### Key Benefits

1. **Cross-Implementation Verification**: Ensures Rust and Circom implementations produce identical results
2. **Cryptographic Consistency**: Validates that both implementations use the same parameters and algorithms
3. **Integration Testing**: Provides confidence that the two codebases can work together seamlessly
4. **Regression Prevention**: Catches any divergence between implementations during development

### Technical Details

- **Field**: Both implementations use the BLS12-381 scalar field
- **Hash Function**: Poseidon255 with identical constants and parameters
- **Input Processing**: Both handle the same input format and validation
- **Output Format**: Both produce field elements in the same representation

### Troubleshooting Poseidon Tests

#### Common Issues
1. **Compilation Errors**: Ensure circom and circomlib are properly installed
2. **Witness Generation Errors**: Check that input JSON matches the expected format
3. **Mismatched Outputs**: Verify that both implementations use the same Poseidon parameters

#### Debugging
- Use the generated witness files to verify outputs manually
- Check that both implementations use the same field arithmetic
- Verify Poseidon constants are identical between implementations
