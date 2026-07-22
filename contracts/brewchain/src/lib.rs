#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
pub enum DataKey {
    Owner,
    Name,
    Symbol,
    Decimals,
    Balance(Address), // Stores balance per wallet
}

#[contract]
pub struct BrewChain;

#[contractimpl]
impl BrewChain {
    /// Initializes the contract. Only callable once.
    pub fn initialize(env: Env, owner: Address, name: String, symbol: String) {
        if !env.storage().instance().has(&DataKey::Owner) {
            env.storage().instance().set(&DataKey::Owner, &owner);
            env.storage().instance().set(&DataKey::Name, &name);
            env.storage().instance().set(&DataKey::Symbol, &symbol);
            env.storage().instance().set(&DataKey::Decimals, &7u32); // 10^7 for micro-coffee precision
        }
    }

    // --- View Functions ---
    pub fn name(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Name)
            .unwrap_or_else(|| String::from_str(&env, "BrewChain"))
    }

    pub fn symbol(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Symbol)
            .unwrap_or_else(|| String::from_str(&env, "BREW"))
    }

    pub fn decimals(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Decimals)
            .unwrap_or(7)
    }

    /// Owner-only: Mints new BREW tokens to a customer's address.
    pub fn mint(env: Env, to: Address, amount: i128) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth(); // Ensures only the coffee shop owner can mint

        let mut balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
            
        balance += amount;
        
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &balance);
    }

    /// User-facing: Burns the caller's own BREW tokens to redeem a free coffee.
    pub fn burn(env: Env, from: Address, amount: i128) {
        // Require authorization from the target account
        from.require_auth();

        let mut balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        if balance < amount {
            panic!("insufficient balance");
        }
        
        balance -= amount;
        
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &balance);
    }

    /// Checks the BREW balance of any address.
    pub fn balance(env: Env, address: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(address))
            .unwrap_or(0)
    }
}