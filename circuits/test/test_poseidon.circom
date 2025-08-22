pragma circom 2.2.0;

include "../poseidon255.circom";

/**
 * @title PoseidonSingle template
 * @dev Template for computing poseidon hash over a single input element
 * @notice This circuit uses the poseidon hash function from circomlib
 */
template PoseidonSingle() {
    // inputs 
    signal input in;                    // input value to hash
    
    // outputs
    signal output out;                  // poseidon hash output
    
    // components
    component hasher = Poseidon255(1);    // poseidon hash for 1 input
    
    // implementation
    hasher.in[0] <== in;
    out <== hasher.out;
}

component main { public [in] } = PoseidonSingle();
