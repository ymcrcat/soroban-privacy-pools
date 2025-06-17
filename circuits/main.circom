pragma circom 2.2.0;

include "commitment.circom";
include "merkleProof.circom";
include "poseidon.circom";

template Withdraw(maxTreeDepth) {
    // PUBLIC SIGNALS
    signal input withdrawnValue;

    // signals for merkle tree inclusion proofs
    signal input stateRoot;             // a known state root
    signal input stateTreeDepth;        // current state tree depth
    
    // PRIVATE SIGNALS

    // signals to compute commitments
    signal input label;                 // keccak256(scope, nonce) % SNARK_SCALAR_FIELD
    signal input existingValue;         // value of the existing commitment
    signal input existingNullifier;     // nullifier of the existing commitment
    signal input existingSecret;        // Secret of the existing commitment
    signal input newNullifier;          // nullifier of the new commitment
    signal input newSecret;             // secret of the new commitment

    // signals for merkle tree inclusion proofs
    signal input stateSiblings[maxTreeDepth];   // siblings of the state tree
    signal input stateIndex;                     // indices for the state tree

    // OUTPUT SIGNALS
    signal output newCommitmentHash;    // hash of new commitment
    signal output existingNullifierHash; // hash of existing commitment nullifier

    // IMPLEMENTATION

    // compute existing commitment
    component existingCommitmentHasher = CommitmentHasher();
    existingCommitmentHasher.label <== label;
    existingCommitmentHasher.value <== existingValue;
    existingCommitmentHasher.secret <== existingSecret;
    existingCommitmentHasher.nullifier <== existingNullifier;
    signal existingCommitment <== existingCommitmentHasher.commitment;

    // output existing nullifier hash
    existingNullifierHash <== existingCommitmentHasher.nullifierHash;

    // verify existing commitment is in the state tree
    component stateRootChecker = MerkleProof(maxTreeDepth);
    stateRootChecker.leaf <== existingCommitment;
    stateRootChecker.leafIndex <== stateIndex;
    stateRootChecker.siblings <== stateSiblings;
    stateRootChecker.actualDepth <== stateTreeDepth;
    
    stateRoot === stateRootChecker.out;

    // check the withdrawn value is valid
    signal remainingValue <== existingValue - withdrawnValue;
    component remainingValueRangeCheck = Num2Bits(128);
    remainingValueRangeCheck.in <== remainingValue;
    _ <== remainingValueRangeCheck.out;

    component withdrawnValueRangeCheck = Num2Bits(128);
    withdrawnValueRangeCheck.in <== withdrawnValue;
    _ <== withdrawnValueRangeCheck.out;

    // check existing and new nullifiers are different
    component nullifierEqualityCheck = IsEqual();
    nullifierEqualityCheck.in[0] <== existingNullifier;
    nullifierEqualityCheck.in[1] <== newNullifier;
    nullifierEqualityCheck.out === 0;

    // compute new commitment
    component newCommitmentHasher = CommitmentHasher();
    newCommitmentHasher.label <== label;
    newCommitmentHasher.value <== remainingValue;
    newCommitmentHasher.nullifier <== newNullifier;
    newCommitmentHasher.secret <== newSecret;

    // output new commitment hash
    newCommitmentHash <== newCommitmentHasher.commitment;
    _ <== newCommitmentHasher.nullifierHash;
}

component main = Withdraw(20);  // 20 levels = 1,048,576-leaf tree
