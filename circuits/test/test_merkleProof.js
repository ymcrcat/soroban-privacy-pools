const { readFileSync, writeFileSync } = require("fs");
const { poseidon2 } = require("poseidon-lite");

/**
 * Test data generator for MerkleProof circuit
 */
class MerkleProofTester {
    constructor() {
        // No need for Poseidon initialization
    }

    /**
     * Hash two values using Poseidon (same as the circuit)
     */
    hash(left, right) {
        // Convert inputs to BigInt for Poseidon
        const leftBigInt = BigInt(left);
        const rightBigInt = BigInt(right);
        
        // Use Poseidon hash (same as circuit's Poseidon(2))
        return poseidon2([leftBigInt, rightBigInt]);
    }

    /**
     * Generate a simple merkle tree for testing
     */
    generateSimpleTree(leaves, depth) {
        const tree = [];
        tree[0] = leaves; // Level 0: leaves

        // Build tree levels
        for (let level = 1; level <= depth; level++) {
            tree[level] = [];
            const prevLevel = tree[level - 1];
            
            for (let i = 0; i < prevLevel.length; i += 2) {
                if (i + 1 < prevLevel.length) {
                    // Hash two children
                    tree[level].push(this.hash(prevLevel[i], prevLevel[i + 1]));
                } else {
                    // Propagate single child
                    tree[level].push(prevLevel[i]);
                }
            }
        }

        return tree;
    }

    /**
     * Generate merkle proof for a given leaf index using LeanIMT logic
     */
    generateProof(tree, leafIndex, depth) {
        const proof = {
            leaf: tree[0][leafIndex],
            leafIndex: leafIndex,
            siblings: [],
            actualDepth: depth,
            expectedRoot: depth === 0 ? tree[0][leafIndex] : tree[depth][0]
        };

        let currentIndex = leafIndex;
        
        for (let level = 0; level < depth; level++) {
            const levelNodes = tree[level];
            const siblingIndex = currentIndex % 2 === 0 ? currentIndex + 1 : currentIndex - 1;
            
            if (siblingIndex < levelNodes.length) {
                proof.siblings.push(levelNodes[siblingIndex]);
            } else {
                proof.siblings.push(0); // Empty sibling - will be handled by LeanIMT logic
            }
            
            currentIndex = Math.floor(currentIndex / 2);
        }

        return proof;
    }

    /**
     * Test case 1: Simple 2-leaf tree
     */
    generateTest1() {
        const leaves = [1, 2];
        const tree = this.generateSimpleTree(leaves, 1);
        return this.generateProof(tree, 0, 1);
    }

    /**
     * Test case 2: 4-leaf tree
     */
    generateTest2() {
        const leaves = [1, 2, 3, 4];
        const tree = this.generateSimpleTree(leaves, 2);
        return this.generateProof(tree, 1, 2);
    }

    /**
     * Test case 3: 8-leaf tree with depth 3
     */
    generateTest3() {
        const leaves = [10, 20, 30, 40, 50, 60, 70, 80];
        const tree = this.generateSimpleTree(leaves, 3);
        return this.generateProof(tree, 5, 3);
    }

    /**
     * Test case 4: Edge case - single leaf
     */
    generateTest4() {
        const leaves = [100];
        const tree = this.generateSimpleTree(leaves, 0);
        return this.generateProof(tree, 0, 0);
    }

    /**
     * Test case 5: Edge case - leaf at different positions
     */
    generateTest5() {
        const leaves = [1, 2, 3, 4, 5, 6, 7, 8];
        const tree = this.generateSimpleTree(leaves, 3);
        return this.generateProof(tree, 0, 3); // Leftmost leaf
    }

    /**
     * Negative Test 1: Invalid proof with wrong sibling values
     */
    generateNegativeTest1() {
        const leaves = [1, 2, 3, 4];
        const tree = this.generateSimpleTree(leaves, 2);
        const correctProof = this.generateProof(tree, 1, 2);
        
        // Corrupt the proof by changing sibling values
        const invalidProof = {
            ...correctProof,
            siblings: [999, 888] // Wrong sibling values
        };
        
        return {
            ...invalidProof,
            description: "Invalid proof with wrong sibling values - should fail witness generation"
        };
    }

    /**
     * Negative Test 2: Invalid proof with wrong leaf index
     */
    generateNegativeTest2() {
        const leaves = [1, 2, 3, 4];
        const tree = this.generateSimpleTree(leaves, 2);
        const correctProof = this.generateProof(tree, 1, 2);
        
        // Corrupt the proof by using wrong leaf index
        const invalidProof = {
            ...correctProof,
            leafIndex: 0, // Wrong index - should be 1
            description: "Invalid proof with wrong leaf index - should fail witness generation"
        };
        
        return invalidProof;
    }

    /**
     * Negative Test 3: Invalid proof with wrong actual depth
     */
    generateNegativeTest3() {
        const leaves = [1, 2, 3, 4];
        const tree = this.generateSimpleTree(leaves, 2);
        const correctProof = this.generateProof(tree, 1, 2);
        
        // Corrupt the proof by using wrong depth
        const invalidProof = {
            ...correctProof,
            actualDepth: 1, // Wrong depth - should be 2
            description: "Invalid proof with wrong actual depth - should fail witness generation"
        };
        
        return invalidProof;
    }


    /**
     * Generate all test cases
     */
    generateAllTests() {
        return {
            test1: this.generateTest1(),
            test2: this.generateTest2(),
            test3: this.generateTest3(),
            test4: this.generateTest4(),
            test5: this.generateTest5(),
            negativeTest1: this.generateNegativeTest1(),
            negativeTest2: this.generateNegativeTest2(),
            negativeTest3: this.generateNegativeTest3()
        };
    }

    /**
     * Save test data to JSON file
     */
    saveTestData(filename, testData) {
        // Convert BigInt values to strings for JSON serialization
        const jsonSafeData = JSON.parse(JSON.stringify(testData, (key, value) =>
            typeof value === 'bigint' ? value.toString() : value
        ));
        
        const jsonData = JSON.stringify(jsonSafeData, null, 2);
        writeFileSync(filename, jsonData);
        console.log(`Test data saved to ${filename}`);
    }

    /**
     * Convert test data to circuit input format
     */
    convertToCircuitFormat(testData) {
        // Convert values to field element format (numbers)
        const convertToFieldElement = (value) => {
            // If it's already a number, return it
            if (typeof value === 'number') {
                return value;
            }
            
            // If it's a string, try to convert to number
            if (typeof value === 'string') {
                const num = parseInt(value, 10);
                if (!isNaN(num)) {
                    return num;
                }
                // If it's a hex string, convert to number
                if (value.startsWith('0x')) {
                    return Number(BigInt(value));
                }
            }
            
            // If it's a BigInt, convert to number
            if (typeof value === 'bigint') {
                return Number(value);
            }
            
            // Default fallback
            return 0;
        };

        // Pad siblings array to match maxDepth (4)
        const maxDepth = 4;
        const paddedSiblings = [...testData.siblings];
        while (paddedSiblings.length < maxDepth) {
            paddedSiblings.push(0); // Use number 0
        }

        return {
            leaf: convertToFieldElement(testData.leaf),
            leafIndex: testData.leafIndex.toString(),
            siblings: paddedSiblings.map(sibling => convertToFieldElement(sibling)),
            actualDepth: testData.actualDepth.toString()
        };
    }

    /**
     * Print test data for manual verification
     */
    printTestData(testName, testData) {
        console.log(`\n=== ${testName} ===`);
        console.log(`Leaf: ${testData.leaf}`);
        console.log(`Leaf Index: ${testData.leafIndex}`);
        console.log(`Siblings: [${testData.siblings.join(', ')}]`);
        console.log(`Actual Depth: ${testData.actualDepth}`);
        console.log(`Expected Root: ${testData.expectedRoot}`);
    }
}

/**
 * Main test execution
 */
function main() {
    const tester = new MerkleProofTester();

    console.log("Generating MerkleProof test cases...");

    // Generate all test cases
    const allTests = tester.generateAllTests();

    // Print test data
    tester.printTestData("Test 1: 2-leaf tree", allTests.test1);
    tester.printTestData("Test 2: 4-leaf tree", allTests.test2);
    tester.printTestData("Test 3: 8-leaf tree", allTests.test3);
    tester.printTestData("Test 4: Single leaf", allTests.test4);
    tester.printTestData("Test 5: Leftmost leaf in 8-leaf tree", allTests.test5);

    // Print negative test data
    console.log("\n🧪 NEGATIVE TEST CASES (Should compute wrong roots):");
    tester.printTestData("Negative Test 1: Wrong siblings", allTests.negativeTest1);
    tester.printTestData("Negative Test 2: Wrong leaf index", allTests.negativeTest2);
    tester.printTestData("Negative Test 3: Wrong depth", allTests.negativeTest3);

    // Save test data to files
    tester.saveTestData("test_inputs.json", allTests);
    
    // Save individual test files for easier testing (excluding negative tests to avoid duplicates)
    Object.keys(allTests).forEach(testName => {
        if (!testName.startsWith('negative')) {
            tester.saveTestData(`test_${testName}.json`, allTests[testName]);
        }
    });
    
    // Save a circuit-compatible test input (using test4 - single leaf as it's simplest)
    const circuitInput = tester.convertToCircuitFormat(allTests.test4);
    tester.saveTestData("circuit_input.json", circuitInput);
    console.log("Circuit-compatible test data saved to circuit_input.json");

    // Save negative test inputs for testing failure cases
    console.log("\n💥 Saving negative test inputs for failure testing...");
    Object.keys(allTests).forEach(testName => {
        if (testName.startsWith('negative')) {
            const negativeInput = tester.convertToCircuitFormat(allTests[testName]);
            tester.saveTestData(`test_${testName}.json`, negativeInput);
            console.log(`Negative test input saved to test_${testName}.json`);
        }
    });

    console.log("\nTest generation complete!");
    console.log("\nTo test the circuit:");
    console.log("1. Compile the test circuit: circom test_merkleProof.circom --r1cs --wasm --sym -o ../build");
    console.log("2. Generate witness: node ../build/test_merkleProof_js/generate_witness.js ../build/test_merkleProof_js/test_merkleProof.wasm test_inputs.json witness.wtns");
    console.log("3. Verify the proof using your preferred ZK proof system");
}

// Run if this file is executed directly
if (require.main === module) {
    main();
}

module.exports = MerkleProofTester;
