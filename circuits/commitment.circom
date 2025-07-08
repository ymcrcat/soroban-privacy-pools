pragma circom 2.2.0;

include "keccak256.circom";

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
