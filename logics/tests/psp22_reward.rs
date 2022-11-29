#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
#[openbrush::contract]
pub mod psp22_reward {
    use ink_storage::traits::SpreadAllocate;
    use openbrush::contracts::access_control::{*, access_control};
    use openbrush::traits::Storage;

    use lucky::impls::reward::psp22_reward;
    use lucky::impls::reward::psp22_reward::*;

    #[ink(storage)]
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        rewards: psp22_reward::Data,
        #[storage_field]
        access: access_control::Data,
    }

    impl Psp22Reward for Contract {}
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.rewards = psp22_reward::Data::default();
                let caller = instance.env().caller();
                instance._init_with_admin(caller);
                instance.grant_role(REWARD_MANAGER, caller).expect("Should grant the role REWARD_MANAGER");
                instance.grant_role(REWARD_VIEWER, caller).expect("Should grant the role REWARD_VIEWER");
            })
        }

    }

    impl psp22_reward::Internal for Contract {
        fn _emit_rewards_claimed_event(&self, _account: AccountId, _amount: Balance){
            // no event for the tests
        }
        fn _emit_pending_reward_event(&self, _account: AccountId, _era: u32, _amount: Balance) {
            // no event for the tests
        }
    }

    mod tests {
        use ink_lang as ink;
        use ink_env::debug_println;
        use openbrush::test_utils::accounts;

        use super::*;


        #[ink::test]
        fn test_fund_rewards_and_add_winners_insufficient_transferred_value() {

            let mut contract = Contract::default();

            let accounts = accounts();
            let era = 1;

            // 600 > 100 => error
            let result = ink_env::pay_with_call!(contract.fund_rewards_and_add_winners(
                era, [(accounts.alice, 600)].to_vec()), 100
            );     
            
            match result {
                Err(InsufficientTransferredBalance) => debug_println!("Insufficient Transferred Balance as expected"), 
                _ => panic!("Error 1!"),
            };
            
            // 600 > 100 => ok
            ink_env::pay_with_call!(contract.fund_rewards_and_add_winners(
                era, [(accounts.alice, 600)].to_vec()), 600
            ).unwrap();
        }


        #[ink::test]
        fn test_fund_rewards_and_add_winners() {

            let mut contract = Contract::default();

            let accounts = accounts();

            // set the rewards for era 1
            ink_env::pay_with_call!(contract.fund_rewards_and_add_winners(
                1,
                [(accounts.alice, 600)].to_vec()
            ), 1000).unwrap();

            match contract.get_pending_rewards_from(accounts.alice) {
                Ok(Some(v)) => assert_eq!(v, 600),
                _ => panic!("Error 1!"),
            };
            match contract.get_pending_rewards_from(accounts.bob) {
                Ok(None) => debug_println!("No Reward as expected"), 
                _ => panic!("Error 2!"),
            };

            // set the rewards for era 2
            ink_env::pay_with_call!(contract.fund_rewards_and_add_winners(
                2,
                [(accounts.bob, 400)].to_vec()
            ), 1000).unwrap();

            match contract.get_pending_rewards_from(accounts.alice) {
                Ok(Some(v)) => assert_eq!(v, 600),
                _ => panic!("Rewards for Alice should be 600"),
            }
            match contract.get_pending_rewards_from(accounts.bob) {
                Ok(Some(v)) => assert_eq!(v, 400),
                _ => panic!("Rewards for Bob should be 400"),
            }

            // set the rewards for era 3
            ink_env::pay_with_call!(contract.fund_rewards_and_add_winners(
                3,
                [(accounts.alice, 600), (accounts.django, 200)].to_vec()
            ), 1000).unwrap();


            match contract.get_pending_rewards_from(accounts.alice) {
                Ok(Some(v)) => assert_eq!(v, 1200), // 600 + 600
                _ => panic!("Rewards for Alice should be 1200"),
            }
            match contract.get_pending_rewards_from(accounts.bob) {
                Ok(Some(v)) => assert_eq!(v, 400),
                _ => panic!("Rewards for Bob should be 400"),
            }
            match contract.get_pending_rewards_from(accounts.django) {
                Ok(Some(v)) => assert_eq!(v, 200),
                _ => panic!("Rewards for Django should be 200"),
            }
        
        }

        #[ink::test]
        fn test_no_current_reward_after_claiming() {

            let mut contract = Contract::default();

            let accounts = accounts();
            let era = 1;

            // set the rewards for this era
            ink_env::pay_with_call!(contract.fund_rewards_and_add_winners(
                era,
                [(accounts.alice, 600), (accounts.bob, 400)].to_vec()
            ), 1000).unwrap();


            match contract.get_pending_rewards_from(accounts.alice) {
                Ok(Some(v)) => assert_eq!(v, 600),
                _ => panic!("Rewards for Alice should be 600"),
            }

            match contract.get_pending_rewards_from(accounts.bob) {
                Ok(Some(v)) => assert_eq!(v, 400),
                _ => panic!("Rewards for Bob should be 400"),
            }

            // alice claims => alice doesn't have anymore rewards
            contract.claim_from(accounts.alice).unwrap();
            match contract.get_pending_rewards_from(accounts.alice) {
                Ok(None) => debug_println!("no rewards for Alice"),
                _ => panic!("Alice should have no rewards"),
            }
            // bob still have rewards
            match contract.get_pending_rewards_from(accounts.bob) {
                Ok(Some(v)) => assert_eq!(v, 400),
                _ => panic!("Rewards for Bob should be 400"),
            }
            
            // bob claims => bob doesn't have anymore rewards
            contract.claim_from(accounts.bob).unwrap();
            match contract.get_pending_rewards_from(accounts.bob) {
                Ok(None) => debug_println!("no rewards for bob"),
                _ => panic!("Bob should have no rewards"),
            }

            // claims no rewards => Error
            match contract.claim_from(accounts.bob) {
                Err(NoReward) => debug_println!("no rewards for bob"),
                _ => panic!("Error 1"),
            }

        }


    }
}