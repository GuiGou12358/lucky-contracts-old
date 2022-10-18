#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use ink_lang as ink;

#[cfg(test)]
#[ink::contract]
pub mod native_psp22_reward {
    use ink_storage::traits::SpreadAllocate;
    use openbrush::traits::Storage;

    use loto::impls::reward::psp22_reward;
    use loto::impls::reward::psp22_reward::*;

    #[ink(storage)]
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        rewards: psp22_reward::Data,
    }

    //impl Reward for Contract {}
    //impl Psp22Reward for Contract {}
    //impl native_psp22_reward::_Internal for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.rewards = psp22_reward::Data::default();
            })
        }

        // required to satisfy #[ink::contract]
        #[ink(message)]
        pub fn test(&self){}
    }

    impl psp22_reward::Internal for Contract {
        fn _emit_reward_claimed_event(&self, _account: AccountId, _amount: Balance){
            // no event for the tests
        }
    }

    mod tests {
        use ink_lang as ink;
        use openbrush::test_utils::accounts;

        use super::*;

        #[ink::test]
        fn test_no_reward_no_winner() {
            let mut contract = Contract::default();

            let accounts = accounts();
            let account_1 = accounts.alice;
            let era = 1;

            // the first winner will will all
            contract._set_ratio_distribution(vec![1]);
            // add the winner but no rewards has been set for this era
            contract._add_winners(era, &vec![account_1]);

            // no  reward => no winner
            assert_eq!(contract._has_pending_rewards_from(Some(era), Some(account_1)), Ok(false));
            assert_eq!(contract.list_pending_rewards_from(None, None).len(), 0);
        }

        #[ink::test]
        fn test_no_ratio_distribution_no_winner() {
            let mut contract = Contract::default();

            let accounts = accounts();
            let account_1 = accounts.alice;
            let era = 1;

            // set the rewards for this era
            contract.set_total_rewards(era, 1000);
            // add the winner but no ratio has been set
            contract._add_winners(era, &vec![account_1]);

            // no  reward => no winner
            assert_eq!(contract._has_pending_rewards_from(Some(era), Some(account_1)), Ok(false));
            assert_eq!(contract.list_pending_rewards_from(None, None).len(), 0);
        }

        #[ink::test]
        fn test_no_reward_after_dispatching_them() {
            let mut contract = Contract::default();

            // the first winner will will all
            contract._set_ratio_distribution(vec![1]);

            let accounts = accounts();
            let account_1 = accounts.alice;
            let account_2 = accounts.bob;
            let era = 1;

            // set the rewards for this era
            contract.set_total_rewards(era, 1000);

            // first rafle, dispatch all rewards
            contract._add_winners(era, &vec![account_1]);

            // second rafle for this era; no reward because all is already dispatched
            contract._add_winners(era, &vec![account_2]);

            assert_eq!(contract._has_pending_rewards_from(Some(era), Some(account_1)), Ok(true));
            assert_eq!(contract._has_pending_rewards_from(Some(2), Some(account_1)), Ok(false));
            assert_eq!(contract._has_pending_rewards_from(Some(era), Some(account_2)), Ok(false));
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).len(), 1);
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1))[0].2, 1000);
            assert_eq!(contract.list_pending_rewards_from(Some(2), Some(account_1)).len(), 0);
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_2)).len(), 0);

        }


        #[ink::test]
        fn test_no_current_reward_after_claiming() {

            let mut contract = Contract::default();

            // the first winner will will all
            contract._set_ratio_distribution(vec![1]);

            let accounts = accounts();
            let account_1 = accounts.alice;
            let account_2 = accounts.bob;
            let era = 1;

            // set the rewards for this era
            contract.set_total_rewards(era, 1000);

            // first rafle, dispatch all rewards
            contract._add_winners(era, &vec![account_1]);

            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).len(), 1);
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1))[0].2, 1000);

            // bob claiming don't change erwards for alice
            contract._claim_from(account_2);
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).len(), 1);
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1))[0].2, 1000);

            // alice claim => alice doesn't have anymore rewards
            contract._claim_from(account_1);
            match contract._has_pending_rewards_from(Some(era), Some(account_1)){
                Ok(x) => assert_eq!(x, false),
                _ => assert!(false), // ERROR
            }
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).len(), 0);

        }


    }
}