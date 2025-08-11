CIRCOMLIB=/opt/homebrew/lib/node_modules/circomlib/circuits
CIRCUITS=circuits/main.circom circuits/commitment.circom circuits/merkleProof.circom

.circuits: $(CIRCUITS)
	@cd circuits && circom main.circom --r1cs --wasm --sym -o build -l $(CIRCOMLIB) --prime bls12381
	@cd circuits && circom dummy.circom --r1cs --wasm --sym -o build -l $(CIRCOMLIB) --prime bls12381
	@ls -l circuits/build/main.r1cs circuits/build/main.sym circuits/build/main_js/main.wasm

test_circuits: .circuits
	@cd circuits/test && ./run_tests.sh