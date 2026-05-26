#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};
use soroban_sdk::token::{AdminClient, Client as TokenClient};

// Helper function to setup the environment, token, and contract
fn setup(env: &Env) -> (Address, Address, Address, TokenClient, KaziPayContractClient) {
    let client = Address::generate(env);
    let freelancer = Address::generate(env);
    let admin = Address::generate(env);
    
    // Setup dummy token
    let token_contract = env.register_stellar_asset_contract(admin.clone());
    let token_admin_client = AdminClient::new(env, &token_contract);
    let token_client = TokenClient::new(env, &token_contract);
    
    // Mint tokens to the client
    token_admin_client.mint(&client, &1000);

    // Setup KaziPay Contract
    let contract_id = env.register_contract(None, KaziPayContract);
    let contract_client = KaziPayContractClient::new(env, &contract_id);

    (client, freelancer, token_contract, token_client, contract_client)
}

#[test]
fn test_1_happy_path_init_and_release() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, freelancer, token, token_client, contract_client) = setup(&env);

    // Init MVP Transaction
    contract_client.init(&client, &freelancer, &token, &100);
    assert_eq!(token_client.balance(&client), 900);
    assert_eq!(token_client.balance(&contract_client.address), 100);

    // Release MVP Transaction
    contract_client.release();
    assert_eq!(token_client.balance(&freelancer), 100);
    assert_eq!(token_client.balance(&contract_client.address), 0);
}

#[test]
#[should_panic(expected = "Escrow already initialized")]
fn test_2_edge_case_duplicate_init() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, freelancer, token, _, contract_client) = setup(&env);

    contract_client.init(&client, &freelancer, &token, &100);
    // Should panic on second initialization
    contract_client.init(&client, &freelancer, &token, &100); 
}

#[test]
fn test_3_state_verification() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, freelancer, token, _, contract_client) = setup(&env);

    contract_client.init(&client, &freelancer, &token, &100);
    
    let state = contract_client.get_state();
    assert_eq!(state.client, client);
    assert_eq!(state.freelancer, freelancer);
    assert_eq!(state.token, token);
    assert_eq!(state.amount, 100);
    assert_eq!(state.is_released, false);

    contract_client.release();
    
    let updated_state = contract_client.get_state();
    assert_eq!(updated_state.is_released, true);
}

#[test]
#[should_panic]
fn test_4_edge_case_unauthorized_caller() {
    let env = Env::default();
    // Do NOT mock all auths to test authorization failure
    let client = Address::generate(&env);
    let freelancer = Address::generate(&env);
    let admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(admin);
    
    let contract_id = env.register_contract(None, KaziPayContract);
    let contract_client = KaziPayContractClient::new(&env, &contract_id);

    // This will panic because the client has not authorized the initialization
    contract_client.init(&client, &freelancer, &token, &100);
}

#[test]
#[should_panic(expected = "Funds already released")]
fn test_5_edge_case_double_release() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, freelancer, token, _, contract_client) = setup(&env);

    contract_client.init(&client, &freelancer, &token, &100);
    contract_client.release();
    
    // Should panic because funds are already released
    contract_client.release();
}