#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
#[openbrush::contract]
pub mod oracle {
    use openbrush::contracts::access_control::{*, access_control};
    use openbrush::traits::Storage;

    use lucky::impls::{
        *,
        oracle::*,
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        oracle_data: oracle::Data,
        #[storage_field]
        access: access_control::Data,
    }

    impl OracleDataConsumer for Contract {}
    impl OracleDataManager for Contract {}
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            instance.oracle_data = oracle::Data::default();
            let caller = instance.env().caller();
            instance._init_with_admin(caller);
            instance.grant_role(ORACLE_DATA_MANAGER, caller).expect("Should grant the role ORACLE_DATA_MANAGER");
            instance
        }

    }

    mod tests {
        use openbrush::test_utils::accounts;

        use super::*;

        #[ink::test]
        fn test_participants() {
            let mut contract = Contract::new();

            let accounts = accounts();
            let account_1 = accounts.alice;
            let account_2 = accounts.bob;
            let account_3 = accounts.charlie;

            contract.add_participants(1, vec![(account_1, 100), (account_2, 200)]).unwrap();
            contract.add_participants(2, vec![(account_3, 300)]).unwrap();

            let participants = contract.get_data(1).participants;

            assert_eq!(participants.len(), 2);
            assert_eq!(participants[0].0, account_1);
            assert_eq!(participants[0].1, 100);
            assert_eq!(participants[1].0, account_2);
            assert_eq!(participants[1].1, 200);

            let participants = contract.get_data(2).participants;
            assert_eq!(participants.len(), 1);
            assert_eq!(participants[0].0, account_3);
            assert_eq!(participants[0].1, 300);

            let participants = contract.get_data(3).participants;
            assert_eq!(participants.len(), 0);

        }

        #[ink::test]
        fn test_rewards() {
            let mut contract = Contract::new();

            contract.set_rewards(1, 1000).unwrap();
            contract.set_rewards(2, 500).unwrap();

            assert_eq!(contract.get_data(1).rewards, 1000);

            assert_eq!(contract.get_data(2).rewards, 500);
            assert_eq!(contract.get_data(3).rewards, 0);

        }

    }
}