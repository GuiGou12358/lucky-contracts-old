#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
#[openbrush::contract]
pub mod participant_manager {
    use openbrush::contracts::access_control::{*, access_control};
    use openbrush::traits::Storage;

    use lucky::impls::{
        *,
        participant_manager::*,
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        participant_manager: participant_manager::Data,
        #[storage_field]
        access: access_control::Data,
    }

    impl ParticipantManager for Contract {}
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            instance._init_with_admin(caller);
            instance.grant_role(PARTICIPANT_MANAGER, caller).expect("Should grant the role ORACLE_DATA_MANAGER");
            instance
        }

    }

    mod tests {
        use openbrush::test_utils::accounts;
        use ink::env::debug_println;
        use crate::participant_manager::MAX_PART;

        use super::*;

        #[ink::test]
        fn test_0_participants() {
            let contract = Contract::new();

            assert_eq!(contract.get_nb_participants(), 0);
            assert_eq!(contract.get_total_value(), 0);

            assert_eq!(contract.get_participants(0).unwrap().len(), 0);
            assert_eq!(contract.get_participants(1).unwrap().len(), 0);
            assert_eq!(contract.get_participants(2).unwrap().len(), 0);
            assert_eq!(contract.get_participants(3).unwrap().len(), 0);
            assert_eq!(contract.get_participants(4).unwrap().len(), 0);
            assert_eq!(contract.get_participants(5).unwrap().len(), 0);
            assert_eq!(contract.get_participants(6).unwrap().len(), 0);
        }


        #[ink::test]
        fn test_participants() {
            let mut contract = Contract::new();

            let accounts = accounts();
            let account_1 = accounts.alice;
            let account_2 = accounts.bob;
            let account_3 = accounts.charlie;
            let account_4 = accounts.django;

            contract.add_participants(
                vec![(account_1, 100), (account_2, 200),
                     (account_3, 300), (account_4, 400)]
            ).unwrap();

            assert_eq!(contract.get_nb_participants(), 4);
            assert_eq!(contract.get_total_value(), 100 + 200 + 300 + 400);

            let participants = contract.get_participants(1).unwrap();
            assert_eq!(participants.len(), 4);
            assert_eq!(participants[0].account, account_1);
            assert_eq!(participants[0].value, 100);
            assert_eq!(participants[1].account, account_2);
            assert_eq!(participants[1].value, 200);
            assert_eq!(participants[2].account, account_3);
            assert_eq!(participants[2].value, 300);
            assert_eq!(participants[3].account, account_4);
            assert_eq!(participants[3].value, 400);

            assert_eq!(contract.get_participant(0).unwrap(), account_1);
            assert_eq!(contract.get_participant(50).unwrap(), account_1);
            assert_eq!(contract.get_participant(100).unwrap(), account_1);
            assert_eq!(contract.get_participant(101).unwrap(), account_2);
            assert_eq!(contract.get_participant(200).unwrap(), account_2);
            assert_eq!(contract.get_participant(300).unwrap(), account_2);
            assert_eq!(contract.get_participant(301).unwrap(), account_3);
            assert_eq!(contract.get_participant(400).unwrap(), account_3);
            assert_eq!(contract.get_participant(600).unwrap(), account_3);
            assert_eq!(contract.get_participant(601).unwrap(), account_4);
            assert_eq!(contract.get_participant(999).unwrap(), account_4);
            assert_eq!(contract.get_participant(1000).unwrap(), account_4);

            match contract.get_participant(1001) {
                None => debug_println!("As expected, no participant found when weight is too much"),
                _ => panic!("We should not find participant"),
            }
        }

        #[ink::test]
        fn test_many_participants() {
            let mut contract = Contract::new();

            let accounts = accounts();
            let account_1 = accounts.alice;
            let account_2 = accounts.bob;
            let account_3 = accounts.charlie;
            let account_4 = accounts.django;
            for _i in 0..100 {
                contract.add_participants(
                    vec![(account_1, 100), (account_2, 200),
                         (account_3, 300), (account_4, 400)]
                ).unwrap();
            }
            //assert_eq!(contract.get_nb_participants(), 1200);

            for _i in 0..1 {
                //assert_eq!(contract.get_participant(0 * i).unwrap(), account_1);
                assert_eq!(contract.get_participant(50).unwrap(), account_1);
                //assert_eq!(contract.get_participant(100 * i).unwrap(), account_1);
                //assert_eq!(contract.get_participant(101 * i).unwrap(), account_2);
                //assert_eq!(contract.get_participant(200 * i).unwrap(), account_2);
                //assert_eq!(contract.get_participant(300 * i).unwrap(), account_2);
                //assert_eq!(contract.get_participant(301 * i).unwrap(), account_3);
                //assert_eq!(contract.get_participant(400 * i).unwrap(), account_3);
                //assert_eq!(contract.get_participant(600* i).unwrap(), account_3);
                //assert_eq!(contract.get_participant(601* i).unwrap(), account_4);
                //assert_eq!(contract.get_participant(999 * i).unwrap(), account_4);
                //assert_eq!(contract.get_participant(1000 * i).unwrap(), account_4);
            }

        }

        #[ink::test]
        fn test_too_many_participants() {
            let mut contract = Contract::new();

            let accounts = accounts();
            let nb_iter = MAX_PART / 4;
            for _i in 0..nb_iter {
                contract.add_participants(
                    vec![
                        (accounts.alice, 100),
                        (accounts.bob, 200),
                        (accounts.charlie, 200),
                        (accounts.django, 300),
                    ]
                ).unwrap();
            }
            assert_eq!(contract.get_nb_participants() as usize, MAX_PART);

            match contract.add_participants(vec![(accounts.alice, 100)]) {
                Err(ParticipantManagerError::MaxSizeExceeded) => debug_println!("Max size exceeded"),
                _ => panic!("We should exceed the max size limit"),
            }
        }

        #[ink::test]
        fn test_clear_data() {
            let mut contract = Contract::new();

            let accounts = accounts();

            let nb_iter = MAX_PART / 4;
            for _i in 0..nb_iter {
                contract.add_participants(
                    vec![
                        (accounts.alice, 100),
                        (accounts.bob, 200),
                        (accounts.charlie, 200),
                        (accounts.django, 300),
                    ]
                ).unwrap();
            }
            assert_eq!(contract.get_nb_participants() as usize, MAX_PART);
            assert_ne!(contract.get_total_value(), 0);

            contract.clear_data().unwrap();
            assert_eq!(contract.get_nb_participants(), 0);
            assert_eq!(contract.get_total_value(), 0);

            // test idempotency
            contract.clear_data().unwrap();
            assert_eq!(contract.get_nb_participants(), 0);
            assert_eq!(contract.get_total_value(), 0);

        }

    }
}