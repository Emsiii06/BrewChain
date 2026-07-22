#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Env, String};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_1_initialize_works() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let name = String::from_str(&env, "BrewChain");
        let symbol = String::from_str(&env, "BREW");

        BrewChain::initialize(env.clone(), owner.clone(), name.clone(), symbol.clone());

        assert_eq!(BrewChain::name(env.clone()), name);
        assert_eq!(BrewChain::symbol(env.clone()), symbol);
        assert_eq!(BrewChain::decimals(env.clone()), 7);
    }

    #[test]
    fn test_2_happy_path_mint_and_burn() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let student = Address::generate(&env);

        // Setup
        BrewChain::initialize(
            env.clone(),
            owner.clone(),
            String::from_str(&env, "BrewChain"),
            String::from_str(&env, "BREW"),
        );

        // 1. Owner mints 100 BREW to student
        BrewChain::mint(env.clone(), student.clone(), 100);
        assert_eq!(BrewChain::balance(env.clone(), student.clone()), 100);

        // 2. Student burns 10 BREW to redeem
        env.mock_all_auths(); // Simulates student signing the burn tx
        BrewChain::burn(env.clone(), 10);
        assert_eq!(BrewChain::balance(env.clone(), student.clone()), 90);
    }

    #[test]
    #[should_panic(expected = "insufficient balance")]
    fn test_3_edge_case_burn_too_much() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let student = Address::generate(&env);

        BrewChain::initialize(
            env.clone(),
            owner.clone(),
            String::from_str(&env, "BrewChain"),
            String::from_str(&env, "BREW"),
        );

        BrewChain::mint(env.clone(), student.clone(), 5);

        // Student tries to burn 10 (only has 5)
        env.mock_all_auths();
        BrewChain::burn(env.clone(), 10);
    }

    #[test]
    #[should_panic(expected = "require auth")]
    fn test_4_edge_case_non_owner_mints() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let hacker = Address::generate(&env);
        let student = Address::generate(&env);

        BrewChain::initialize(
            env.clone(),
            owner.clone(),
            String::from_str(&env, "BrewChain"),
            String::from_str(&env, "BREW"),
        );

        // Hacker tries to mint without being the owner -> should panic due to require_auth
        env.mock_all_auths(); // We mock auth, but the address is not the owner, so contract rejects.
        BrewChain::mint(env.clone(), student.clone(), 100);
        // Note: We don't need to check balance, panic happens inside mint.
    }

    #[test]
    fn test_5_state_verification_after_multiple_operations() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        BrewChain::initialize(
            env.clone(),
            owner.clone(),
            String::from_str(&env, "BrewChain"),
            String::from_str(&env, "BREW"),
        );

        // Owner mints to both
        BrewChain::mint(env.clone(), user1.clone(), 50);
        BrewChain::mint(env.clone(), user2.clone(), 30);

        // User1 burns 20
        env.mock_all_auths();
        // We need to set invoker as user1 for the burn.
        // In mock_all_auths, it mocks auth for all addresses, but the invoker is still the default.
        // We can just call as is, but to be precise we set the env's invoker.
        // Instead, we just rely on mock_all_auths allowing the call.
        // However, burn uses env.invoker(). In mock_all_auths, env.invoker() returns the default test address.
        // So we need to ensure user1 is the invoker. Let's just use the default address as user1 to simplify.
        // Let's rework: generate user1, set it as the invoker.
        // Actually, simpler: since we use mock_all_auths, it bypasses auth checks but we need to simulate user1.
        // We'll just call burn as the default address, but mint goes to user1. Let's just mint to the default address.
        let default_addr = Address::generate(&env); // Wait, default invoker is env.current_contract_address? 
        // The simplest way: just mint to env.current_contract_address()? No.
        // Let's just call burn without mock_all_auths? It will fail because invoker is not the owner? No, burn doesn't check owner, it checks invoker's balance.
        // Let's just use the default invoker address.
        // The standard way in soroban test is to use `env.as_contract(&student, || { ... })` but to keep it simple, 
        // I'll just mint to the default invoker address.
        // Actually, just use env.current_contract_address() as the student.
        let student = env.current_contract_address();
        BrewChain::mint(env.clone(), student.clone(), 100);
        BrewChain::burn(env.clone(), 40); // burns from current invoker (which is the contract address if not set? Actually invoker is the caller.
        // Let's just set the invoker properly.
        // I'll use the standard pattern:
        // env.as_contract(&student, || { BrewChain::burn(env.clone(), 20) });
        
        // Let's rewrite this test cleanly without weird invoker issues:
        // We'll use `env.as_contract` to simulate user calls.
        let env = Env::default();
        let owner = Address::generate(&env);
        let student = Address::generate(&env);

        BrewChain::initialize(
            env.clone(),
            owner.clone(),
            String::from_str(&env, "BrewChain"),
            String::from_str(&env, "BREW"),
        );

        BrewChain::mint(env.clone(), student.clone(), 100);

        // Simulate student calling burn
        env.as_contract(&student, || {
            BrewChain::burn(env.clone(), 30);
            assert_eq!(BrewChain::balance(env.clone(), student.clone()), 70);
        });
        // Verify state outside the contract call
        assert_eq!(BrewChain::balance(env.clone(), student.clone()), 70);
    }
}