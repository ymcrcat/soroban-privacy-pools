#![no_std]
use soroban_sdk::{
    contract, contractimpl, 
    vec, Env, String, Vec, Address, symbol_short, Symbol, Bytes, BytesN
};

use zk::{Groth16Verifier, VerificationKey, Proof, PublicSignals};

#[cfg(test)]
mod test;

// Error messages
pub const ERROR_NULLIFIER_USED: &str = "Nullifier already used";
pub const ERROR_INSUFFICIENT_BALANCE: &str = "Insufficient balance";
pub const ERROR_COIN_OWNERSHIP_PROOF: &str = "Couldn't verify coin ownership proof";
pub const ERROR_SUCCESS: &str = "Withdrawal successful";

// Storage keys
const COMMIT_KEY: Symbol = symbol_short!("commit");
const NULL_KEY: Symbol = symbol_short!("null");
const BALANCE_KEY: Symbol = symbol_short!("balance");
const VK_KEY: Symbol = symbol_short!("vk");

const FIXED_AMOUNT: i128 = 1000000000; // 1 XLM in stroops

#[contract]
pub struct PrivacyPoolsContract;

// This is a sample contract. Replace this placeholder with your own contract logic.
// A corresponding test example is available in `test.rs`.
//
// For comprehensive examples, visit <https://github.com/stellar/soroban-examples>.
// The repository includes use cases for the Stellar ecosystem, such as data storage on
// the blockchain, token swaps, liquidity pools, and more.
//
// Refer to the official documentation:
// <https://developers.stellar.org/docs/build/smart-contracts/overview>.
#[contractimpl]
impl PrivacyPoolsContract {
    pub fn __constructor(env: &Env, vk_bytes: Bytes) {
        // Only allow initialization if not already set
        if env.storage().instance().has(&VK_KEY) {
            panic!("Contract already initialized");
        }
        env.storage().instance().set(&VK_KEY, &vk_bytes);
    }

    pub fn deposit(env: &Env, from: Address, commitment: BytesN<32>) {
        from.require_auth();
        
        // Store the commitment
        let mut commitments: Vec<BytesN<32>> = env.storage().instance().get(&COMMIT_KEY)
            .unwrap_or(vec![&env]);
        commitments.push_back(commitment);
        env.storage().instance().set(&COMMIT_KEY, &commitments);

        // Update contract balance
        let current_balance = env.storage().instance().get(&BALANCE_KEY)
            .unwrap_or(0);
        env.storage().instance().set(&BALANCE_KEY, &(current_balance + FIXED_AMOUNT));
    }

    pub fn withdraw(env: &Env, 
            to: Address, 
            nullifier: BytesN<32>, 
            proof_bytes: Bytes, 
            pub_signals_bytes: Bytes) -> Vec<String> {
        to.require_auth();

        let vk_bytes: Bytes = env.storage().instance().get(&VK_KEY).unwrap();
        let vk = VerificationKey::from_bytes(env, &vk_bytes).unwrap();
        let proof = Proof::from_bytes(env, &proof_bytes);
        let pub_signals = PublicSignals::from_bytes(env, &pub_signals_bytes);

        // Check if nullifier has been used before
        let mut nullifiers: Vec<BytesN<32>> = env.storage().instance().get(&NULL_KEY)
            .unwrap_or(vec![env]);
        
        if nullifiers.contains(&nullifier) {
            return vec![env, String::from_str(env, ERROR_NULLIFIER_USED)]
        }

        // In a real implementation, we would verify the zero-knowledge proof here
        // For this example, we'll skip the actual proof verification
        let res = Groth16Verifier::verify_proof(env, vk, proof, &pub_signals.pub_signals);
        if res.is_err() || !res.unwrap() {
            return vec![env, String::from_str(env, ERROR_COIN_OWNERSHIP_PROOF)]
        }
        
        // Add nullifier to used nullifiers
        nullifiers.push_back(nullifier);
        env.storage().instance().set(&NULL_KEY, &nullifiers);

        // Check and update contract balance
        let current_balance = env.storage().instance().get(&BALANCE_KEY)
            .unwrap_or(0);
        if current_balance < FIXED_AMOUNT {
            return vec![env, String::from_str(env, ERROR_INSUFFICIENT_BALANCE)]
        }

        env.storage().instance().set(&BALANCE_KEY, &(current_balance - FIXED_AMOUNT));
        
        return vec![env, String::from_str(env, ERROR_SUCCESS)]
    }

    pub fn get_commitments(env: &Env) -> Vec<BytesN<32>> {
        env.storage().instance().get(&COMMIT_KEY)
            .unwrap_or(vec![env])
    }

    pub fn get_nullifiers(env: &Env) -> Vec<BytesN<32>> {
        env.storage().instance().get(&NULL_KEY)
            .unwrap_or(vec![env])
    }

    pub fn get_balance(env: &Env) -> i128 {
        env.storage().instance().get(&BALANCE_KEY)
            .unwrap_or(0)
    }
}
