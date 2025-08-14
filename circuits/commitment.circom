pragma circom 2.2.0;

include "keccak256.circom";  // Uses standard field-compatible hashing for commitments

/**
 * @title CommitmentHasher template
 * @dev Template for generating privacy pool commitments and nullifier hashes
 * 
 * @notice HASH FUNCTION CHOICE:
 *   This template uses the standard field-compatible hash functions:
 *   - Keccak256FieldHash1(): For single field elements (254-bit safe)
 *   - Keccak256FieldHash2(): For two field elements (254-bit safe) 
 *   - Keccak256FieldHash3(): For three field elements (254-bit safe)
 * 
 *   These are DIFFERENT from the Merkle tree hashing which must use
 *   Keccak256FieldHash2_256() for Soroban compatibility. The commitment
 *   hashing doesn't need Soroban compatibility since commitments are
 *   generated and verified entirely within the circom circuit.
 * 
 * @notice COMMITMENT STRUCTURE:
 *   commitment = Keccak256(value, label, Keccak256(nullifier, secret))
 *   nullifierHash = Keccak256(nullifier)
 */
template CommitmentHasher() {
    
    // inputs
    signal input value;
    signal input label;              // keccak256(pool_scope, nonce) % SNARK_SCALAR_FIELD
    signal input secret;             // secret of commitment
    signal input nullifier;
    
    // outputs
    signal output commitment;
    signal output nullifierHash;

    component nullifierHasher = Keccak256FieldHash1();
    nullifierHasher.in <== nullifier;
    
    component precommitmentHasher = Keccak256FieldHash2();
    precommitmentHasher.in[0] <== nullifier;
    precommitmentHasher.in[1] <== secret;

    component commitmentHasher = Keccak256FieldHash3();
    commitmentHasher.in[0] <== value;
    commitmentHasher.in[1] <== label;
    commitmentHasher.in[2] <== precommitmentHasher.out;

    commitment <== commitmentHasher.out;
    nullifierHash <== nullifierHasher.out;
}
