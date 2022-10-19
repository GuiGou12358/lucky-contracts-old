#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
#[openbrush::contract]
pub mod game {
    use ink_storage::traits::SpreadAllocate;
    use openbrush::contracts::access_control::{*, access_control, AccessControl};
    use openbrush::traits::Storage;

    use loto::impls::{
        *,
        game::*,
        manual_participant_management::*,
        reward::*,
        reward::psp22_reward::*,
    };
    use loto::traits::raffle::Raffle;

    #[ink(storage)]
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        participants_manager: manual_participant_management::Data,
        #[storage_field]
        rafle: raffle::Data,
        #[storage_field]
        game: game::Data,
        #[storage_field]
        reward: psp22_reward::Data,
        #[storage_field]
        access: access_control::Data,
    }

    impl Raffle for Contract {}
    impl Game for Contract {}
    impl Psp22Reward for Contract{}
    impl ParticipantManagement for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.participants_manager = manual_participant_management::Data::default();
                instance.rafle = raffle::Data::default();
                instance.game = game::Data::default();
                instance.reward = psp22_reward::Data::default();
                let caller = instance.env().caller();
                instance._init_with_admin(caller);
                instance.grant_role(PARTICIPANT_MANAGER, caller).expect("Should grant the role PARTICIPANT_MANAGER");
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
        use ink_env::debug_println;
        use ink_lang as ink;
        use openbrush::test_utils::accounts;

        use super::*;

        pub fn test(contract: &mut super::Contract, era: u128){
            let accounts = accounts();
            contract.set_total_rewards(era, 110).unwrap();
            contract.add_participant(era, accounts.alice, 100000).unwrap();
            contract.add_participant(era, accounts.bob, 100000).unwrap();
            contract.add_participant(era, accounts.charlie, 100000).unwrap();
            contract.add_participant(era, accounts.django, 100000).unwrap();
            contract.add_participant(era, accounts.eve, 100000).unwrap();
            contract.add_participant(era, accounts.frank, 100000).unwrap();
            contract._play(era).unwrap();

            debug_println!("winner era {} : {:?} ", era, contract.list_pending_rewards_from(Some(era), None));
        }


        #[ink::test]
        fn test_multi_users() {
            let mut contract = super::Contract::default();
            contract._set_max_winners_by_raffle(3);
            contract._set_ratio_distribution(vec![50, 30, 20]).unwrap();
            test(&mut contract, 1);
        }

    }
}