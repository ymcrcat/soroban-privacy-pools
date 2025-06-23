#![cfg(test)]
use super::*;
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String};
use soroban_sdk::testutils::Address as TestAddress;

#[test]
fn test_deposit_and_withdraw() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    // Create test addresses
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    
    let client = ContractClient::new(&env, &contract_id);

    // Test initial balance
    assert_eq!(client.get_balance(), 0);

    // Test deposit
    let commitment = BytesN::from_array(&env, &[1u8; 32]);
    
    // Mock authentication for alice
    env.mock_all_auths();
    client.deposit(&alice, &commitment);
    
    // Check balance after deposit
    assert_eq!(client.get_balance(), FIXED_AMOUNT);
    // Check commitments
    let commitments = client.get_commitments();
    assert_eq!(commitments.len(), 1);
    assert_eq!(commitments.get(0).unwrap(), commitment);

    // Test withdraw
    let nullifier = BytesN::from_array(&env, &[2u8; 32]);
    let proof = Bytes::from_slice(&env, &[0u8; 128]); // Dummy proof
    
    // Mock authentication for bob
    env.mock_all_auths();
    let result = client.withdraw(&bob, &nullifier, &proof);
    assert_eq!(
        result,
        vec![
            &env,
            String::from_str(&env, "Withdrawal successful")
        ]
    );

    // Check balance after withdrawal
    assert_eq!(client.get_balance(), 0);

    // Check nullifiers
    let nullifiers = client.get_nullifiers();
    assert_eq!(nullifiers.len(), 1);
    assert_eq!(nullifiers.get(0).unwrap(), nullifier);
}

#[test]
fn test_withdraw_insufficient_balance() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let bob = Address::generate(&env);
    let nullifier = BytesN::from_array(&env, &[3u8; 32]);
    let proof = Bytes::from_slice(&env, &[0u8; 128]); // Dummy proof
    // Attempt to withdraw with zero balance
    env.mock_all_auths();
    let result = client.withdraw(&bob, &nullifier, &proof);
    assert_eq!(
        result,
        vec![
            &env,
            String::from_str(&env, "Insufficient balance")
        ]
    );
}

#[test]
fn test_reuse_nullifier() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // First deposit
    let commitment = BytesN::from_array(&env, &[4u8; 32]);
    env.mock_all_auths();
    client.deposit(&alice, &commitment);

    // First withdraw
    let nullifier = BytesN::from_array(&env, &[5u8; 32]);
    let proof = Bytes::from_slice(&env, &[0u8; 128]); // Dummy proof
    env.mock_all_auths();
    client.withdraw(&bob, &nullifier, &proof);

    // Second deposit
    let commitment2 = BytesN::from_array(&env, &[6u8; 32]);
    env.mock_all_auths();
    client.deposit(&alice, &commitment2);
    // Attempt to reuse nullifier
    env.mock_all_auths();
    let result = client.withdraw(&bob, &nullifier, &proof);
    assert_eq!(
        result,
        vec![
            &env,
            String::from_str(&env, "Nullifier already used")
        ]
    );
}
