#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod random_generator {

    use openbrush::{
        modifiers,
        traits::Storage
    };
    use openbrush::contracts::access_control::{
        *,
        AccessControlError,
        AccessControl,
        RoleType
    };
    use lucky::impls::{
        random_generator,
        random_generator::*,
    };

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        AccessControlError(AccessControlError),
        UpgradeError,
    }

    /// convertor from AccessControlError to ContractError
    impl From<AccessControlError> for ContractError {
        fn from(error: AccessControlError) -> Self {
            ContractError::AccessControlError(error)
        }
    }

    /// Contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        random_generator_data: random_generator::Data,
        #[storage_field]
        access: access_control::Data,
    }

    /// implementations of the contracts
    impl RandomGenerator for Contract{}
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            instance._init_with_admin(caller);
            instance.grant_role(RANDOM_GENERATOR_CONSUMER, caller).expect("Should grant the role RANDOM_GENERATOR_CONSUMER");
            instance.grant_role(RANDOM_GENERATOR_MANAGER, caller).expect("Should grant the role RANDOM_GENERATOR_MANAGER");
            instance
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn upgrade_contract(&mut self, new_code_hash: [u8; 32]) -> Result<(), ContractError> {
            ink::env::set_code_hash(&new_code_hash).map_err(|_| ContractError::UpgradeError)?;
            Ok(())
        }

        #[ink(message)]
        pub fn get_role_random_generator_consumer(&self) -> RoleType {
            RANDOM_GENERATOR_CONSUMER
        }


        #[ink(message)]
        pub fn get_role_random_generator_manager(&self) -> RoleType {
            RANDOM_GENERATOR_MANAGER
        }

    }

}
