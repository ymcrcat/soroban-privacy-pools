pragma circom 2.2.0;

include "commitment.circom";
include "poseidon.circom";
include "comparators.circom";

// This circuit is for withdrawing by demonstrating only that the withdrawer
// knows the secret and nullifier, without prooving inclusion of the commitment in the Merkle tree.
// This is for testing purposes only, to enable testing depositing and withdrawing coins prior to 
// testing the full Merkle tree implementation.

template Dummy() {
    signal input withdrawnValue;
    signal input label;
    signal input existingValue;
    signal input existingNullifier;
    signal input existingSecret;
    signal input newNullifier;
    signal input newSecret;

    signal output existingNullifierHash;
    signal output newCommitmentHash;

    component commitmentHasher = CommitmentHasher();
    commitmentHasher.value <== withdrawnValue;
    commitmentHasher.label <== label;
    commitmentHasher.secret <== existingSecret;
    commitmentHasher.nullifier <== existingNullifier;

    signal existingCommitment <== commitmentHasher.commitment;
    existingNullifierHash <== commitmentHasher.nullifierHash;

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

component main = Dummy();