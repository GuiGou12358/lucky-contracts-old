#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod dapps_staking_proxy {

    use dapps_staking_proxy_2_lib::*;


    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        DSError(DSError),
    }

    /// convertor from RewardError to ContractError
    impl From<DSError> for ContractError {
        fn from(error: DSError) -> Self {
            ContractError::DSError(error)
        }
    }

    /// Contract storage
    #[ink(storage)]
    pub struct Contract {
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self { Self {} }

        #[ink(message)]
        pub fn read_current_era(&self) -> u32 {
            DAppsStaking::read_current_era()
        }

        #[ink(message)]
        pub fn read_era_info(&mut self, era: u32) -> Result<EraInfo<Balance>, ContractError> {
            let era_info = DAppsStaking::read_era_info(era)?;
            Ok(era_info)
        }

        #[ink(message)]
        pub fn bond_and_stake(&mut self, value: Balance) -> Result<(), ContractError> {
            let contract = self.env().account_id();
            DAppsStaking::bond_and_stake(contract, value)?;
            Ok(())
        }

    }

}
