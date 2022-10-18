#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use ink_lang as ink;

#[cfg(test)]
#[ink::contract]
pub mod game {
    use ink_storage::traits::SpreadAllocate;
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
    }

    impl Raffle for Contract{}


    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.participants_manager = manual_participant_management::Data::default();
                instance.rafle = raffle::Data::default();
                instance.game = game::Data::default();
                instance.reward = psp22_reward::Data::default();
            })
        }

        // required to satisfy #[ink::contract]
        #[ink(message)]
        pub fn test(&self){
        }

    }

    mod tests {
        use ink_lang as ink;
        use ink_env::debug_println;
        use openbrush::test_utils::accounts;

        use super::*;

        pub fn test(contract: &mut super::Contract, era: u128){
            let accounts = accounts();
            contract.set_total_rewards(era, 110);
            contract.add_participant(era, accounts.alice, 100000);
            contract.add_participant(era, accounts.bob, 100000);
            contract.add_participant(era, accounts.charlie, 100000);
            contract.add_participant(era, accounts.django, 100000);
            contract.add_participant(era, accounts.eve, 100000);
            contract.add_participant(era, accounts.frank, 100000);
            contract._play(era);

            debug_println!("winner era {} : {:?} ", era, contract.list_pending_rewards_from(Some(era), None));
        }


        #[ink::test]
        fn test_multi_users() {
            let mut contract = super::Contract::default();

            contract._set_max_winners_by_raffle(3);
            contract._set_ratio_distribution(vec![50, 30, 20]);

            for i in 0..10 {
                test(&mut contract, i);
            }
        }

    }
}