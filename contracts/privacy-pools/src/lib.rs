#![no_std]

use soroban_sdk::{
    contract, contractimpl, 
    vec, Env, String, Vec, Address, symbol_short, Symbol, Bytes, BytesN, U256
};
use soroban_sdk::crypto::bls12_381::Fr;
use ark_ff::{BigInteger, PrimeField};

use zk::{Groth16Verifier, VerificationKey, Proof, PublicSignals};
use lean_imt::{LeanIMT, TREE_ROOT_KEY, TREE_DEPTH_KEY, TREE_LEAVES_KEY};

#[cfg(test)]
mod test;

// Error messages
pub const ERROR_NULLIFIER_USED: &str = "Nullifier already used";
pub const ERROR_INSUFFICIENT_BALANCE: &str = "Insufficient balance";
pub const ERROR_COIN_OWNERSHIP_PROOF: &str = "Couldn't verify coin ownership proof";
pub const ERROR_WITHDRAW_SUCCESS: &str = "Withdrawal successful";

const TREE_DEPTH: u32 = 2;

// Storage keys
const NULL_KEY: Symbol = symbol_short!("null");
const BALANCE_KEY: Symbol = symbol_short!("balance");
const VK_KEY: Symbol = symbol_short!("vk");

const FIXED_AMOUNT: i128 = 1000000000; // 1 XLM in stroops

#[contract]
pub struct PrivacyPoolsContract;

#[contractimpl]
impl PrivacyPoolsContract {
    pub fn __constructor(env: &Env, vk_bytes: Bytes) {
        env.storage().instance().set(&VK_KEY, &vk_bytes);
        
        // Initialize empty merkle tree with fixed depth
        let tree = LeanIMT::new(env, TREE_DEPTH);
        let (leaves, depth, root) = tree.to_storage();
        env.storage().instance().set(&TREE_LEAVES_KEY, &leaves);
        env.storage().instance().set(&TREE_DEPTH_KEY, &depth);
        env.storage().instance().set(&TREE_ROOT_KEY, &root);
    }

    /// Stores a commitment in the merkle tree and updates the tree state
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `commitment` - The commitment to store
    /// 
    /// # Returns
    /// * A tuple of (updated_merkle_root, leaf_index) after insertion
    fn store_commitment(env: &Env, commitment: BytesN<32>) -> (BytesN<32>, u32) {
        // Load current tree state
        let leaves: Vec<BytesN<32>> = env.storage().instance().get(&TREE_LEAVES_KEY)
            .unwrap_or(vec![&env]);
        let depth: u32 = env.storage().instance().get(&TREE_DEPTH_KEY)
            .unwrap_or(0);
        let root: BytesN<32> = env.storage().instance().get(&TREE_ROOT_KEY)
            .unwrap_or(BytesN::from_array(&env, &[0u8; 32]));
        
        // Create tree and insert new commitment
        let mut tree = LeanIMT::from_storage(env, leaves, depth, root);
        tree.insert(commitment);
        
        // Get the leaf index (it's the last leaf in the tree)
        let leaf_index = tree.get_leaf_count() - 1;
        
        // Store updated tree state
        let (new_leaves, new_depth, new_root) = tree.to_storage();
        env.storage().instance().set(&TREE_LEAVES_KEY, &new_leaves);
        env.storage().instance().set(&TREE_DEPTH_KEY, &new_depth);
        env.storage().instance().set(&TREE_ROOT_KEY, &new_root);

        (new_root, leaf_index)
    }

    /// Deposits funds into the privacy pool and stores a commitment in the merkle tree.
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
    /// # Returns
    ///
    /// * The leaf index where the commitment was stored in the merkle tree
    ///
    /// # Security
    ///
    /// * Requires authentication from the `from` address
    /// * The commitment is stored in a merkle tree for efficient inclusion proofs
    /// * Each deposit adds exactly `FIXED_AMOUNT` (1 XLM) to the contract balance
    ///
    /// # Storage
    ///
    /// * Updates the merkle tree with the new commitment
    /// * Increases the contract balance by `FIXED_AMOUNT`
    pub fn deposit(env: &Env, from: Address, commitment: BytesN<32>) -> u32 {
        from.require_auth();
        
        // Store the commitment in the merkle tree
        let (_, leaf_index) = Self::store_commitment(env, commitment);

        // Update contract balance
        let current_balance = env.storage().instance().get(&BALANCE_KEY)
            .unwrap_or(0);
        env.storage().instance().set(&BALANCE_KEY, &(current_balance + FIXED_AMOUNT));

        leaf_index
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
            proof_bytes: Bytes, 
            pub_signals_bytes: Bytes) -> Vec<String> {
        to.require_auth();

        // Check contract balance before updating state
        let current_balance = env.storage().instance().get(&BALANCE_KEY)
            .unwrap_or(0);
        if current_balance < FIXED_AMOUNT {
            return vec![env, String::from_str(env, ERROR_INSUFFICIENT_BALANCE)]
        }

        let vk_bytes: Bytes = env.storage().instance().get(&VK_KEY).unwrap();
        let vk = VerificationKey::from_bytes(env, &vk_bytes).unwrap();
        let proof = Proof::from_bytes(env, &proof_bytes);
        let pub_signals = PublicSignals::from_bytes(env, &pub_signals_bytes);

        // Extract public signals: [nullifierHash, withdrawnValue, stateRoot]
        let nullifier_hash = &pub_signals.pub_signals.get(0).unwrap();
        let _withdrawn_value = &pub_signals.pub_signals.get(1).unwrap();
        let state_root = &pub_signals.pub_signals.get(2).unwrap();

        // Validate state root matches current LeanIMT root
        let leaves: Vec<BytesN<32>> = env.storage().instance().get(&TREE_LEAVES_KEY)
            .unwrap_or(vec![&env]);
        let depth: u32 = env.storage().instance().get(&TREE_DEPTH_KEY)
            .unwrap_or(0);
        let root: BytesN<32> = env.storage().instance().get(&TREE_ROOT_KEY)
            .unwrap_or(BytesN::from_array(&env, &[0u8; 32]));
        
        let tree = LeanIMT::from_storage(env, leaves, depth, root);
        let current_root_scalar = tree.get_root_scalar();
        let current_root_bytes = current_root_scalar.into_bigint().to_bytes_be();
        let mut padded_bytes = [0u8; 32];
        let offset = 32 - current_root_bytes.len();
        padded_bytes[offset..].copy_from_slice(&current_root_bytes);
        let current_root_u256 = U256::from_be_bytes(env, &Bytes::from_array(env, &padded_bytes));
        let current_root_fr = Fr::from_u256(current_root_u256);
        if state_root != &current_root_fr {
            return vec![env, String::from_str(env, ERROR_COIN_OWNERSHIP_PROOF)]
        }

        // Check if nullifier has been used before
        let mut nullifiers: Vec<BytesN<32>> = env.storage().instance().get(&NULL_KEY)
            .unwrap_or(vec![env]);

        let nullifier = nullifier_hash.to_bytes();
        
        if nullifiers.contains(&nullifier) {
            return vec![env, String::from_str(env, ERROR_NULLIFIER_USED)]
        }
        
        let res = Groth16Verifier::verify_proof(env, vk, proof, &pub_signals.pub_signals);
        if res.is_err() || !res.unwrap() {
            return vec![env, String::from_str(env, ERROR_COIN_OWNERSHIP_PROOF)]
        }

        // Add nullifier to used nullifiers only after all checks pass
        nullifiers.push_back(nullifier);
        env.storage().instance().set(&NULL_KEY, &nullifiers);

        // Update contract balance
        env.storage().instance().set(&BALANCE_KEY, &(current_balance - FIXED_AMOUNT));

        return vec![env, String::from_str(env, ERROR_WITHDRAW_SUCCESS)]
    }

    /// Gets the current merkle root of the commitment tree
    pub fn get_merkle_root(env: &Env) -> BytesN<32> {
        env.storage().instance().get(&TREE_ROOT_KEY)
            .unwrap_or(BytesN::from_array(&env, &[0u8; 32]))
    }

    /// Gets the current depth of the merkle tree
    pub fn get_merkle_depth(env: &Env) -> u32 {
        env.storage().instance().get(&TREE_DEPTH_KEY)
            .unwrap_or(0)
    }

    /// Gets the number of commitments (leaves) in the merkle tree
    pub fn get_commitment_count(env: &Env) -> u32 {
        let leaves: Vec<BytesN<32>> = env.storage().instance().get(&TREE_LEAVES_KEY)
            .unwrap_or(vec![&env]);
        leaves.len() as u32
    }

    /// Gets all commitments (leaves) in the merkle tree
    pub fn get_commitments(env: &Env) -> Vec<BytesN<32>> {
        env.storage().instance().get(&TREE_LEAVES_KEY)
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
