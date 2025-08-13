pragma circom 2.2.0;

include "merkleProof.circom";

/**
 * @title TestMerkleProof template
 * @dev Simple test circuit that instantiates the MerkleProof template
 * @notice This is used to test compatibility between lean-imt and merkleProof.circom
 * @param maxDepth The maximum depth of the Merkle tree (set to 10 for testing)
 */
template TestMerkleProof(maxDepth) {
    // inputs 
    signal input leaf;                  // leaf value to prove inclusion of
    signal input leafIndex;             // index of leaf in the Merkle tree
    signal input siblings[maxDepth];    // sibling values along the path to the root
    signal input actualDepth;           // current tree depth

    // outputs
    signal output out;
    
    // Instantiate the MerkleProof template
    component merkleProof = MerkleProof(maxDepth);
    merkleProof.leaf <== leaf;
    merkleProof.leafIndex <== leafIndex;
    merkleProof.siblings <== siblings;
    merkleProof.actualDepth <== actualDepth;
    
    // Output the computed root
    out <== merkleProof.out;
}

// Main component for testing
component main {public [leaf, leafIndex, actualDepth]} = TestMerkleProof(10);  // 10 levels = 1,024-leaf tree
