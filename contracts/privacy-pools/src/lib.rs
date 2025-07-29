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
pub const ERROR_WITHDRAW_SUCCESS: &str = "Withdrawal successful";

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

    /// Deposits funds into the privacy pool and stores a commitment.
    ///
    /// This function allows a user to deposit a fixed amount (1 XLM) into the privacy pool
    /// while providing a cryptographic commitment that will be used for zero-knowledge proof
    /// verification during withdrawal.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `from` - The address of the depositor (must be authenticated)
    /// * `commitment` - A 32-byte cryptographic commitment that will be used to prove
    ///                 ownership during withdrawal without revealing the actual coin details
    ///
    /// # Security
    ///
    /// * Requires authentication from the `from` address
    /// * The commitment is stored publicly but cannot be used to trace the actual deposit
    /// * Each deposit adds exactly `FIXED_AMOUNT` (1 XLM) to the contract balance
    ///
    /// # Storage
    ///
    /// * Updates the commitments list with the new commitment
    /// * Increases the contract balance by `FIXED_AMOUNT`
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

    /// Withdraws funds from the privacy pool using a zero-knowledge proof.
    ///
    /// This function allows a user to withdraw a fixed amount (1 XLM) from the privacy pool
    /// by providing a cryptographic proof that demonstrates ownership of a previously deposited
    /// commitment without revealing which specific commitment it corresponds to.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `to` - The address of the recipient (must be authenticated)
    /// * `nullifier` - A 32-byte unique identifier that prevents double-spending of the same
    ///                commitment. Each nullifier can only be used once.
    /// * `proof_bytes` - The serialized zero-knowledge proof demonstrating ownership of a
    ///                   commitment without revealing the commitment itself
    /// * `pub_signals_bytes` - The serialized public signals associated with the proof
    ///
    /// # Returns
    ///
    /// Returns a vector containing status messages:
    /// * `["Withdrawal successful"]` on successful withdrawal
    /// * `["Nullifier already used"]` if the nullifier has been used before
    /// * `["Couldn't verify coin ownership proof"]` if the zero-knowledge proof verification fails
    /// * `["Insufficient balance"]` if the contract doesn't have enough funds
    ///
    /// # Security
    ///
    /// * Requires authentication from the `to` address
    /// * Verifies that the nullifier hasn't been used before (prevents double-spending)
    /// * Validates the zero-knowledge proof using Groth16 verification
    /// * Each withdrawal deducts exactly `FIXED_AMOUNT` (1 XLM) from the contract balance
    ///
    /// # Storage
    ///
    /// * Adds the nullifier to the used nullifiers list to prevent reuse
    /// * Decreases the contract balance by `FIXED_AMOUNT`
    ///
    /// # Privacy
    ///
    /// * The withdrawal doesn't reveal which specific commitment is being spent
    /// * The nullifier ensures the same commitment cannot be spent twice
    /// * The zero-knowledge proof proves ownership without revealing the commitment details
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
        
        return vec![env, String::from_str(env, ERROR_WITHDRAW_SUCCESS)]
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
