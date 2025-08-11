pragma circom 2.2.0;

include "../merkleProof.circom";
include "poseidon.circom";

/**
 * @title MerkleProof Test Template
 * @dev Single test template that can handle different depths dynamically
 * @notice Tests various scenarios for merkle proof verification
 * @param maxDepth Maximum depth for testing (handles depths 0 to maxDepth)
 */
template MerkleProofTest(maxDepth) {
    // Test inputs
    signal input testLeaf;              // Leaf value to test
    signal input testLeafIndex;         // Index of the leaf
    signal input testSiblings[maxDepth]; // Sibling values (padded to maxDepth)
    signal input testActualDepth;       // Actual tree depth (0 to maxDepth)
    
    // Expected output for verification
    signal input expectedRoot;          // Expected merkle root
    
    // Circuit instance
    component merkleProof = MerkleProof(maxDepth);
    
    // Connect inputs to the circuit
    merkleProof.leaf <== testLeaf;
    merkleProof.leafIndex <== testLeafIndex;
    merkleProof.siblings <== testSiblings;
    merkleProof.actualDepth <== testActualDepth;
    
    // Verify the output matches expected root
    merkleProof.out === expectedRoot;
}

// Main component with configurable max depth for testing
component main = MerkleProofTest(4); // Can be changed to test different depths
