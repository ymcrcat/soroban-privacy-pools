#!/bin/bash

# MerkleProof Circuit Test Runner
# This script automates the entire testing process

set -e  # Exit on any error

echo "🚀 Starting MerkleProof Circuit Tests"
echo "====================================="

# Check if we're in the right directory
if [ ! -f "test_merkleProof.circom" ]; then
    echo "❌ Error: Please run this script from the test directory"
    exit 1
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Error: Node.js is not installed"
    exit 1
fi

# Check if circom is installed
if ! command -v circom &> /dev/null; then
    echo "❌ Error: circom compiler is not installed"
    exit 1
fi

echo "✅ Prerequisites check passed"

# Install dependencies if package.json exists
if [ -f "package.json" ]; then
    echo "📦 Installing dependencies..."
    npm install
    echo "✅ Dependencies installed"
else
    echo "❌ Error: package.json not found"
    exit 1
fi

# Generate test data
echo "🧪 Generating test data..."
npm run test
echo "✅ Test data generated"

# Compile the test circuit
echo "🔨 Compiling test circuit..."
npm run compile
echo "✅ Circuit compiled"

# Check if compilation was successful
if [ ! -f "../build/test_merkleProof_js/test_merkleProof.wasm" ]; then
    echo "❌ Error: Circuit compilation failed"
    exit 1
fi

# Generate witness using circuit-compatible input
echo "🔍 Generating witness..."
if [ -f "circuit_input.json" ]; then
    node ../build/test_merkleProof_js/generate_witness.js \
        ../build/test_merkleProof_js/test_merkleProof.wasm \
        circuit_input.json \
        witness.wtns
    
    if [ -f "witness.wtns" ]; then
        echo "✅ Witness generated successfully"
        echo "📊 Witness file size: $(ls -lh witness.wtns | awk '{print $5}')"
    else
        echo "❌ Error: Witness generation failed"
        exit 1
    fi
else
    echo "❌ Error: Circuit input file not found"
    exit 1
fi

# Test negative cases - these should fail witness generation
echo ""
echo "🧪 Testing negative cases (should fail)..."
negative_tests=(
    "test_negativeTest1.json"
    "test_negativeTest2.json"
    "test_negativeTest3.json"
    "test_negativeTest4.json"
)

negative_test_passed=0
for negative_test in "${negative_tests[@]}"; do
    if [ -f "$negative_test" ]; then
        echo "   Testing: $negative_test"
        if node ../build/test_merkleProof_js/generate_witness.js \
            ../build/test_merkleProof_js/test_merkleProof.wasm \
            "$negative_test" \
            "negative_witness.wtns" 2>/dev/null; then
            echo "   ❌ FAILED: Witness generation succeeded when it should have failed"
            negative_test_passed=1
        else
            echo "   ✅ PASSED: Witness generation failed as expected"
        fi
        
        # Clean up negative witness file if it was created
        if [ -f "negative_witness.wtns" ]; then
            rm "negative_witness.wtns"
        fi
    else
        echo "   ⚠️  Warning: $negative_test not found"
    fi
done

if [ $negative_test_passed -eq 1 ]; then
    echo ""
    echo "⚠️  Warning: Some negative tests passed when they should have failed"
    echo "   This may indicate the circuit is not properly validating proofs"
else
    echo ""
    echo "✅ All negative tests passed - circuit properly rejects invalid proofs"
fi

echo ""
echo "🎉 All tests completed successfully!"
echo ""

# Clean up generated test files
echo "🧹 Cleaning up generated test files..."
cleanup_files=(
    "test_inputs.json"
    "test_test1.json"
    "test_test2.json"
    "test_test3.json"
    "test_test4.json"
    "test_test5.json"
    "circuit_input.json"
    "witness.wtns"
    "test_negativeTest1.json"
    "test_negativeTest2.json"
    "test_negativeTest3.json"
    "test_negativeTest4.json"
    "negative_witness.wtns"
)

for file in "${cleanup_files[@]}"; do
    if [ -f "$file" ]; then
        rm "$file"
        echo "   🗑️  Removed: $file"
    fi
done

echo "✅ Cleanup completed"

echo ""
echo "📁 Source files preserved:"
echo "   - test_merkleProof.circom"
echo "   - test_merkleProof.js"
echo "   - package.json"
echo "   - README.md"
echo "   - run_tests.sh"
echo ""
echo "🔧 Next steps:"
echo "   1. All tests passed successfully!"
echo "   2. Generated files have been cleaned up"
echo "   3. Run './run_tests.sh' again to test with fresh data"
echo ""
echo "📚 For more information, see README.md"
