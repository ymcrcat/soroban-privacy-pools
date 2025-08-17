const fs = require('fs');
const { poseidon2 } = require('poseidon-lite');

/**
 * JavaScript implementation of the MerkleProof circuit logic
 * This computes the expected root using the same algorithm as the circuit
 */

/**
 * Compute the expected root following the circuit logic
 * @param {Object} proof - The merkle proof with leaf, leafIndex, siblings, actualDepth
 * @param {number} maxDepth - Maximum depth of the tree
 * @returns {BigInt} The computed root
 */
function computeExpectedRoot(proof, maxDepth = 4) {
    const { leaf, leafIndex, siblings, actualDepth } = proof;
    
    // Convert leafIndex to binary path (like the circuit does)
    const indices = [];
    let tempIndex = leafIndex;
    for (let i = 0; i < maxDepth; i++) {
        indices.push(tempIndex % 2);
        tempIndex = Math.floor(tempIndex / 2);
    }
    
    // Initialize nodes array (like the circuit)
    const nodes = new Array(maxDepth + 1);
    nodes[0] = BigInt(leaf); // Start with the leaf
    
    // Process each level (like the circuit's for loop)
    for (let i = 0; i < maxDepth; i++) {
        if (i >= actualDepth) {
            // If we're beyond the actual depth, propagate the previous value
            nodes[i + 1] = nodes[i];
        } else {
            // Get the sibling for this level
            const sibling = BigInt(siblings[i] || 0);
            
            // Determine the order based on the path index
            // If index[i] is 0, leaf is on the left; if 1, leaf is on the right
            let left, right;
            if (indices[i] === 0) {
                left = nodes[i];
                right = sibling;
            } else {
                left = sibling;
                right = nodes[i];
            }
            
            // Hash the pair using real Poseidon hash (like the circuit's Poseidon(2))
            nodes[i + 1] = poseidon2([left, right]);
        }
    }
    
    return nodes[maxDepth];
}

/**
 * Test the implementation with the test data
 */
function main() {
    const args = process.argv.slice(2);
    
    if (args.length < 1) {
        console.error("Usage: node compute_expected_root.js <test_file>");
        process.exit(1);
    }
    
    const testFile = args[0];
    
    try {
        const testData = JSON.parse(fs.readFileSync(testFile, 'utf8'));
        const computedRoot = computeExpectedRoot(testData);
        
        console.log(`Test file: ${testFile}`);
        console.log(`Input:`, testData);
        console.log(`Computed expected root: ${computedRoot}`);
        console.log(`Original expected root: ${testData.expectedRoot}`);
        
        // Check if they match (convert both to strings for comparison)
        if (computedRoot.toString() === testData.expectedRoot.toString()) {
            console.log("✅ Roots match!");
        } else {
            console.log("❌ Roots don't match!");
            console.log(`Computed: ${computedRoot.toString()}`);
            console.log(`Expected: ${testData.expectedRoot.toString()}`);
        }
        
    } catch (error) {
        console.error(`Error processing test file: ${error.message}`);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}

module.exports = { computeExpectedRoot };
