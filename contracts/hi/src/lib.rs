#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, token};

// Define the contract state structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowState {
    pub client: Address,
    pub freelancer: Address,
    pub token: Address,
    pub amount: i128,
    pub is_released: bool,
}

#[contract]
pub struct KaziPayContract;

#[contractimpl]
impl KaziPayContract {
    /// Initializes the escrow by locking USDC from the client into the contract.
    /// Maps to MVP: Client deposits USDC into the contract.
    pub fn init(
        env: Env,
        client: Address,
        freelancer: Address,
        token: Address,
        amount: i128,
    ) {
        // Ensure the client has authorized this transaction
        client.require_auth();

        // Prevent double initialization for the MVP demo contract
        if env.storage().instance().has(&symbol_short!("STATE")) {
            panic!("Escrow already initialized");
        }

        // Transfer funds from the client to this contract's address
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&client, &env.current_contract_address(), &amount);

        // Save the escrow state
        let state = EscrowState {
            client,
            freelancer,
            token,
            amount,
            is_released: false,
        };
        env.storage().instance().set(&symbol_short!("STATE"), &state);
    }

    /// Releases the locked funds to the freelancer.
    /// Maps to MVP: Client approves the work and releases the funds.
    pub fn release(env: Env) {
        // Load the current state
        let mut state: EscrowState = env
            .storage()
            .instance()
            .get(&symbol_short!("STATE"))
            .expect("Escrow not initialized");

        // ONLY the client can authorize the release of funds
        state.client.require_auth();

        if state.is_released {
            panic!("Funds already released");
        }

        // Transfer funds from this contract to the freelancer
        let token_client = token::Client::new(&env, &state.token);
        token_client.transfer(&env.current_contract_address(), &state.freelancer, &state.amount);

        // Update state to prevent double-releases
        state.is_released = true;
        env.storage().instance().set(&symbol_short!("STATE"), &state);
    }

    /// Helper function to view the current state of the escrow
    pub fn get_state(env: Env) -> EscrowState {
        env.storage()
            .instance()
            .get(&symbol_short!("STATE"))
            .expect("Escrow not initialized")
    }
}