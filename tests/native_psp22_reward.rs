#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
#[openbrush::contract]
pub mod native_psp22_reward {
    use ink_storage::traits::SpreadAllocate;
    use openbrush::contracts::access_control::{access_control, AccessControl, Internal};
    use openbrush::traits::Storage;

    use loto::impls::reward::psp22_reward;
    use loto::impls::reward::psp22_reward::*;

    #[ink(storage)]
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        rewards: psp22_reward::Data,
        #[storage_field]
        access: access_control::Data,
    }

    impl Psp22Reward for Contract {}

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
            contract._set_ratio_distribution(vec![1]).unwrap();
            // add the winner but no rewards has been set for this era
            assert!(contract._add_winners(era, &vec![account_1]).is_err()); // expect an error NOREWARD

            // no  reward => no winner
            assert_eq!(contract._has_pending_rewards_from(Some(era), Some(account_1)), Ok(false));
            assert_eq!(contract.list_pending_rewards_from(None, None).unwrap().len(), 0);
        }

        #[ink::test]
        fn test_no_ratio_distribution_no_winner() {
            let mut contract = Contract::default();

            let accounts = accounts();
            let account_1 = accounts.alice;
            let era = 1;

            // set the rewards for this era
            contract.set_total_rewards(era, 1000).unwrap();
            // add the winner but no ratio has been set


            assert!(contract._add_winners(era, &vec![account_1]).is_err());

            // no  reward => no winner
            assert_eq!(contract._has_pending_rewards_from(Some(era), Some(account_1)), Ok(false));
            assert_eq!(contract.list_pending_rewards_from(None, None).unwrap().len(), 0);
        }

        #[ink::test]
        fn test_no_reward_after_dispatching_them() {
            let mut contract = Contract::default();

            // the first winner will will all
            contract._set_ratio_distribution(vec![1]).unwrap();

            let accounts = accounts();
            let account_1 = accounts.alice;
            let account_2 = accounts.bob;
            let era = 1;

            // set the rewards for this era
            contract.set_total_rewards(era, 1000).unwrap();

            // first rafle, dispatch all rewards
            contract._add_winners(era, &vec![account_1]).unwrap();

            // second rafle for this era; no reward because all is already dispatched
            assert!(contract._add_winners(era, &vec![account_2]).is_err()); // expect an error

            assert_eq!(contract._has_pending_rewards_from(Some(era), Some(account_1)), Ok(true));
            assert_eq!(contract._has_pending_rewards_from(Some(2), Some(account_1)), Ok(false));
            assert_eq!(contract._has_pending_rewards_from(Some(era), Some(account_2)), Ok(false));
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).unwrap().len(), 1);
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).unwrap()[0].2, 1000);
            assert_eq!(contract.list_pending_rewards_from(Some(2), Some(account_1)).unwrap().len(), 0);
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_2)).unwrap().len(), 0);

        }


        #[ink::test]
        fn test_no_current_reward_after_claiming() {

            let mut contract = Contract::default();

            // the first winner will will all
            contract._set_ratio_distribution(vec![1]).unwrap();

            let accounts = accounts();
            let account_1 = accounts.alice;
            let account_2 = accounts.bob;
            let era = 1;

            // set the rewards for this era
            contract.set_total_rewards(era, 1000).unwrap();

            // first rafle, dispatch all rewards
            contract._add_winners(era, &vec![account_1]).unwrap();

            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).unwrap().len(), 1);
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).unwrap()[0].2, 1000);

            // bob claiming don't change erwards for alice
            contract._claim_from(account_2).unwrap();
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).unwrap().len(), 1);
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).unwrap()[0].2, 1000);

            // alice claim => alice doesn't have anymore rewards
            contract._claim_from(account_1).unwrap();
            match contract._has_pending_rewards_from(Some(era), Some(account_1)){
                Ok(x) => assert_eq!(x, false),
                _ => assert!(false), // ERROR
            }
            assert_eq!(contract.list_pending_rewards_from(Some(era), Some(account_1)).unwrap().len(), 0);

        }


    }
}