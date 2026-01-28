pragma circom 2.2.0;

include "poseidon255.circom";  // Uses Poseidon255 hashing for commitments

/**
 * @title CommitmentHasher template
 * @dev Template for generating privacy pool commitments and nullifier hashes
 * 
 * @notice HASH FUNCTION CHOICE:
 *   This template uses Poseidon hash functions:
 *   - Poseidon(1): For single field elements
 *   - Poseidon(2): For two field elements
 *   - Poseidon(3): For three field elements
 * 
 *   Poseidon is more efficient for SNARK circuits and provides better
 *   security properties for zero-knowledge applications.
 * 
 * @notice COMMITMENT STRUCTURE:
 *   commitment = Poseidon(value, label, Poseidon(nullifier, secret))
 *   nullifierHash = Poseidon(nullifier)
 */
template CommitmentHasher() {
    
    // inputs
    signal input value;
    signal input label;              // hash(pool_scope, nonce) % SNARK_SCALAR_FIELD
    signal input secret;             // secret of commitment
    signal input nullifier;
    
    // outputs
    signal output commitment;
    signal output nullifierHash;

    component nullifierHasher = Poseidon255(1);
    nullifierHasher.in[0] <== nullifier;

    component precommitmentHasher = Poseidon255(2);
    precommitmentHasher.in[0] <== nullifier;
    precommitmentHasher.in[1] <== secret;

    // Commitment = Poseidon(value, label, precommitment) with t=4 (3 inputs)
    // This matches the Rust implementation: poseidon_hash(env, &[value, label, precommitment])
    component commitmentHasher = Poseidon255(3);
    commitmentHasher.in[0] <== value;
    commitmentHasher.in[1] <== label;
    commitmentHasher.in[2] <== precommitmentHasher.out;

    commitment <== commitmentHasher.out;
    nullifierHash <== nullifierHasher.out;
}
