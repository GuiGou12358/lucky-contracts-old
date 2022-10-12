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
        reward::psp22::*,
        reward::psp22::psp22_reward::*,
        manual_participant_management::*,
        game::*,
    };

    #[ink(storage)]
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        participants_manager: manual_participant_management::Data,
        #[storage_field]
        rafle: rafle::Data,
        #[storage_field]
        game: game::Data,
        #[storage_field]
        reward: psp22_reward::Data,
    }


    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.participants_manager = manual_participant_management::Data::default();
                instance.rafle = rafle::Data::default();
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
            contract._set_total_rewards(era, 110);
            contract._add_participant(era, accounts.alice, 100000);
            contract._add_participant(era, accounts.bob, 100000);
            contract._add_participant(era, accounts.charlie, 100000);
            contract._add_participant(era, accounts.django, 100000);
            contract._add_participant(era, accounts.eve, 100000);
            contract._add_participant(era, accounts.frank, 100000);
            contract._play(era);

            debug_println!("winner era {} : {:?} ", era, contract._list_pending_rewards_from(Some(era), None));
        }


        #[ink::test]
        fn test_multi_users() {
            let mut contract = super::Contract::default();

            contract.rafle.set_nb_winners_by_rafle(3);
            contract._set_ratio_distribution(vec![50, 30, 20]);

            for i in 0..10 {
                test(&mut contract, i);
            }
        }

    }
}