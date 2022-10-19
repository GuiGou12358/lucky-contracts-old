#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
#[openbrush::contract]
pub mod manual_participant_management {
    use ink_storage::traits::SpreadAllocate;
    use openbrush::contracts::access_control::{access_control, AccessControl, Internal};
    use openbrush::traits::Storage;

    use loto::impls::{
        *,
        manual_participant_management::*,
    };

    #[ink(storage)]
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        participants_manager: manual_participant_management::Data,
        #[storage_field]
        access: access_control::Data,
    }

    impl ParticipantManagement for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.participants_manager = manual_participant_management::Data::default();
                let caller = instance.env().caller();
                instance._init_with_admin(caller);
                instance.grant_role(PARTICIPANT_MANAGER, caller).expect("Should grant the role PARTICIPANT_MANAGER");
            })
        }

    }

    mod tests {
        use ink_lang as ink;
        use openbrush::test_utils::accounts;

        use super::*;

        #[ink::test]
        fn test() {
            let mut contract = Contract::default();

            let accounts = accounts();
            let account_1 = accounts.alice;
            let account_2 = accounts.bob;
            let account_3 = accounts.charlie;

            contract.add_participant(1, account_1, 100).unwrap();
            contract.add_participant(1, account_2, 200).unwrap();
            contract.add_participant(2, account_3, 300).unwrap();

            assert_eq!(contract._list_participants(1).len(), 2);
            assert_eq!(contract._list_participants(1)[0].0, account_1);
            assert_eq!(contract._list_participants(1)[0].1, 100);
            assert_eq!(contract._list_participants(1)[1].0, account_2);
            assert_eq!(contract._list_participants(1)[1].1, 200);

            assert_eq!(contract._list_participants(2).len(), 1);
            assert_eq!(contract._list_participants(2)[0].0, account_3);
            assert_eq!(contract._list_participants(2)[0].1, 300);

            assert_eq!(contract._list_participants(3).len(), 0);
        }

    }
}