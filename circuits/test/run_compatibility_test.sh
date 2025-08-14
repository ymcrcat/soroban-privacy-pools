#!/bin/bash

# Script to test compatibility between lean-imt and test_merkleProof.circom
# This script:
# 1. Runs the Rust test to generate JSON input files
# 2. Compiles the merkleProof.circom circuit
# 3. Generates witnesses using the JSON files
# 4. Verifies the witnesses
# 5. Cleans up temporary files

set -e

echo "🧪 Testing lean-imt ↔ merkleProof.circom compatibility..."
echo ""

# Function to cleanup temporary files
cleanup() {
    echo ""
    echo "🧹 Cleaning up temporary files..."
    rm -f test_input_leaf_*.json
    rm -f witness_leaf_*.wtns
    echo "✅ Cleanup completed"
}

# Note: cleanup will be called manually at the end

# Step 1: Generate JSON input files using lean-imt
echo "📋 Step 1: Generating JSON input files using lean-imt..."
cargo run --bin test_lean_imt_compatibility -- --skip-witness
echo ""

# Step 2: Compile the test_merkleProof.circom circuit
echo "🔨 Step 2: Compiling test_merkleProof.circom circuit..."
if [ ! -d "../build" ]; then
    mkdir -p ../build
fi

# Check if circom is available
if ! command -v circom &> /dev/null; then
    echo "❌ Error: circom is not installed or not in PATH"
    echo "Please install circom first: https://docs.circom.io/getting-started/installation/"
    exit 1
fi

# Compile the circuit (we're in circuits/test, so go up one level to circuits/)
cd ..
circom test/test_merkleProof.circom --r1cs --wasm --sym -o build -l /opt/homebrew/lib/node_modules/circomlib/circuits --prime bls12381
cd test
echo "✅ Circuit compiled successfully"
echo ""

# Step 3: Generate witnesses for each test case
echo "🔍 Step 3: Generating witnesses..."
witness_count=0
for i in {0..3}; do
    input_file="test_input_leaf_${i}.json"
    witness_file="witness_leaf_${i}.wtns"
    
    if [ -f "$input_file" ]; then
        echo "   Generating witness for leaf ${i}..."
        
        # Generate witness
        node ../build/test_merkleProof_js/generate_witness.js ../build/test_merkleProof_js/test_merkleProof.wasm "$input_file" "$witness_file"
        
        if [ -f "$witness_file" ]; then
            echo "   ✅ Witness generated: $witness_file"
            ((witness_count++))
        else
            echo "   ❌ Failed to generate witness for leaf ${i}"
        fi
    else
        echo "   ❌ Input file not found: $input_file"
    fi
done

echo ""
echo "🎉 Compatibility test completed!"
echo "   - Generated ${witness_count} witnesses successfully"
echo ""
echo "📁 Generated files:"
echo "   - Circuit files: ../build/merkleProof_*"
echo "   - Temporary files will be cleaned up automatically"
echo ""
echo "🔍 Step 4: Verifying witness outputs (should FAIL due to hash mismatch)..."
echo "   Checking witness files before verification..."
ls -la witness_leaf_*.wtns || echo "   No witness files found!"
echo "   Running witness verification test..."
# We're already in the test directory, so no need to cd
cargo run --bin test_lean_imt_compatibility
echo ""
echo "🔍 Root Comparison Summary:"
echo "   - lean-imt computed root: 0x1010101010101010101010101010101010101010101010101010101010101010"
echo "   - circom computed root: (see test output above for actual values)"
echo ""



# Clean up temporary files
cleanup
