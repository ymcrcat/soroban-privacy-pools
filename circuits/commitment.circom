pragma circom 2.2.0;

include "poseidon.circom";

template CommitmentHasher() {
    
    // inputs
    signal input value;
    signal input label;              // keccak256(pool_scope, nonce) % SNARK_SCALAR_FIELD
    signal input secret;             // secret of commitment
    signal input nullifier;
    
    // outputs
    signal output commitment;
    signal output nullifierHash;

    component nullifierHasher = Poseidon(1);
    nullifierHasher.inputs[0] <== nullifier;
    
    component precommitmentHasher = Poseidon(2);
    precommitmentHasher.inputs[0] <== nullifier;
    precommitmentHasher.inputs[1] <== secret;

    component commitmentHasher = Poseidon(2);
    commitmentHasher.inputs[0] <== value;
    commitmentHasher.inputs[1] <== precommitmentHasher.out;

    commitment <== commitmentHasher.out;
    nullifierHash <== nullifierHasher.out;
}
