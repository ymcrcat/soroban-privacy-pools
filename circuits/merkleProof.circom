pragma circom 2.2.0;

include "keccak256.circom";
include "comparators.circom";
include "mux1.circom";

/**
 * @title MerkleProof template
 * @dev Template for generating and verifying inclusion proofs in a Lean Incremental Merkle Tree
 * @notice This circuit follows the LeanIMT design where:
 *   1. Every node with two children is the hash of its left and right nodes
 *   2. Every node with one child has the same value as its child node
 *   3. Tree is always built from leaves to root
 *   4. Tree is always balanced by construction
 *   5. Tree depth is dynamic and can increase with insertion of new leaves
 * @param maxDepth The maximum depth of the Merkle tree
 */
template MerkleProof(maxDepth) {
    // inputs 
    signal input leaf;                  // leaf value to prove inclusion of
    signal input leafIndex;             // index of leaf in the Merkle tree
    signal input siblings[maxDepth];    // sibling values along the path to the root
    signal input actualDepth;           // current tree depth

    // outputs
    signal output out;
    
    // internal signals
    signal nodes[maxDepth + 1]; // stores computed node values at each level
    signal indices[maxDepth];   // stores path indices for each level

    // components
    component siblingIsEmpty[maxDepth]; // checks if sibling node is empty
    component hashInCorrectOrder[maxDepth]; // orders node pairs for hashing
    component latestValidHash[maxDepth]; // selects between hash and propagation
    component hashes[maxDepth]; // Hash components (can use any hash function)

    // implmenentation
    component depthCheck = LessEqThan(6);
    depthCheck.in[0] <== actualDepth;
    depthCheck.in[1] <== maxDepth;
    depthCheck.out === 1;

    component indexToPath = Num2Bits(maxDepth);
    indexToPath.in <== leafIndex;
    indices <== indexToPath.out;

    // Init leaf with value
    nodes[0] <== leaf;

    for (var i = 0; i < maxDepth; i++) {
        // prepare pairs for both possible orderings
        var childrenToSort[2][2] = [ [nodes[i], siblings[i]], [siblings[i], nodes[i]] ];
        hashInCorrectOrder[i] = MultiMux1(2);
        hashInCorrectOrder[i].c <== childrenToSort;
        hashInCorrectOrder[i].s <== indices[i];
        
        // hash the nodes using the specified hash function
        hashes[i] = Keccak256FieldHash2();
        hashes[i].in[0] <== hashInCorrectOrder[i].out[0];
        hashes[i].in[1] <== hashInCorrectOrder[i].out[1];
        
        // check if sibling is empty
        siblingIsEmpty[i] = IsZero();
        siblingIsEmpty[i].in <== siblings[i];

        // either keep the previous hash or the new one
        nodes[i + 1] <== (nodes[i] - hashes[i].out) * siblingIsEmpty[i].out + hashes[i].out;
    }

    out <== nodes[maxDepth];
}
