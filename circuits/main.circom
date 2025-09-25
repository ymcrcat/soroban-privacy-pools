pragma circom 2.2.0;

include "commitment.circom";
include "merkleProof.circom";
include "poseidon.circom";

template Withdraw(treeDepth) {
    // PUBLIC SIGNALS
    signal input withdrawnValue;

    // PRIVATE SIGNALS
    // signals for merkle tree inclusion proofs
    signal input stateRoot;             // a known state root

    // signals to compute commitments
    signal input label;                 // hash(scope, nonce) % SNARK_SCALAR_FIELD
    signal input value;                 // value of the commitment
    signal input nullifier;             // nullifier of the commitment
    signal input secret;                // Secret of the commitment

    // signals for merkle tree inclusion proofs
    signal input stateSiblings[treeDepth];   // siblings of the state tree
    signal input stateIndex;                    // index of the commitment in the state tree

    // OUTPUT SIGNALS
    signal output nullifierHash;        // hash of commitment nullifier

    // IMPLEMENTATION

    // compute commitment
    component commitmentHasher = CommitmentHasher();
    commitmentHasher.label <== label;
    commitmentHasher.value <== value;
    commitmentHasher.secret <== secret;
    commitmentHasher.nullifier <== nullifier;
    signal commitment <== commitmentHasher.commitment;

    // output nullifier hash
    nullifierHash <== commitmentHasher.nullifierHash;

    // verify commitment is in the state tree
    component stateRootChecker = MerkleProof(treeDepth);
    stateRootChecker.leaf <== commitment;
    stateRootChecker.leafIndex <== stateIndex;
    stateRootChecker.siblings <== stateSiblings;
    
    stateRoot === stateRootChecker.out;

    // check the withdrawn value is valid (must not exceed commitment value)
    signal remainingValue <== value - withdrawnValue;
    component remainingValueRangeCheck = Num2Bits(128);
    remainingValueRangeCheck.in <== remainingValue;
    _ <== remainingValueRangeCheck.out;

    component withdrawnValueRangeCheck = Num2Bits(128);
    withdrawnValueRangeCheck.in <== withdrawnValue;
    _ <== withdrawnValueRangeCheck.out;

    // ensure withdrawn value doesn't exceed commitment value
    // (this is enforced by the remainingValue being non-negative through range check)
}

component main {public [withdrawnValue, stateRoot]} = Withdraw(2);  // 20 levels = 1,048,576-leaf tree
