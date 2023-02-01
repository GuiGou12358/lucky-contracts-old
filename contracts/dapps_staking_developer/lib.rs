#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

/*
pub use self::dapps_staking_developer::{
    Contract,
    ContractRef,
};
*/

#[openbrush::contract]
pub mod dapps_staking_developer {

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

    const WHITELISTED_ADDRESS: RoleType = ink::selector_id!("WHITELISTED_ADDRESS");

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        AccessControlError(AccessControlError),
        TransferError,
        UpgradeError,
    }

    /// convertor from AccessControlError to ContractError
    impl From<access_control::AccessControlError> for ContractError {
        fn from(error: AccessControlError) -> Self {
            ContractError::AccessControlError(error)
        }
    }

    /// Contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        access: access_control::Data,
    }

    /// implementations of the contracts
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            instance._init_with_admin(caller);
            instance.grant_role(WHITELISTED_ADDRESS, caller).expect("Should grant the role WHITELISTED_ADDRESS");
            instance
        }

        #[ink(message, selector = 0x410fcc9d)]
        #[openbrush::modifiers(only_role(WHITELISTED_ADDRESS))]
        pub fn withdraw(&mut self, value: Balance) -> Result<(), ContractError>{
            let caller = Self::env().caller();
            Self::env().transfer(caller, value).map_err(|_| ContractError::TransferError)?;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn upgrade_contract(&mut self, new_code_hash: [u8; 32]) -> Result<(), ContractError> {
            ink::env::set_code_hash(&new_code_hash).map_err(|_| ContractError::UpgradeError)?;
            Ok(())
        }

        #[ink(message)]
        pub fn get_role_whitelisted_address(&self) -> RoleType {
            WHITELISTED_ADDRESS
        }

    }

}
