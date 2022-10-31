#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

use scale::{Decode, Encode, HasCompact};
use ink_env::{AccountId, DefaultEnvironment, Environment};
pub type Balance = <DefaultEnvironment as Environment>::Balance;

#[ink::chain_extension]
pub trait DappsStakingExt {

    type ErrorCode = DSErrorCode;

    #[ink(extension = 0801u32, returns_result = false, handle_status = false)]
    fn read_current_era() -> u32;

    #[ink(extension = 0802u32)]
    fn read_era_info(
        era: u32,
    ) -> Result<EraInfo<<ink_env::DefaultEnvironment as Environment>::Balance>, DSError>;

    #[ink(extension = 0803u32)]
    fn bond_and_stake(
        account_id: <ink_env::DefaultEnvironment as Environment>::AccountId,
        value: <ink_env::DefaultEnvironment as Environment>::Balance,
    ) -> Result<(), DSError>;
}


/// A record of rewards allocated for stakers and dapps
#[derive(PartialEq, Eq, Clone, Default, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RewardInfo<Balance: HasCompact> {
    /// Total amount of rewards for stakers in an era
    #[codec(compact)]
    pub stakers: Balance,
    /// Total amount of rewards for dapps in an era
    #[codec(compact)]
    pub dapps: Balance,
}

/// A record for total rewards and total amount staked for an era
#[derive(PartialEq, Eq, Clone, Default, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct EraInfo<Balance: HasCompact> {
    /// Total amount of earned rewards for an era
    pub rewards: RewardInfo<Balance>,
    /// Total staked amount in an era
    #[codec(compact)]
    pub staked: Balance,
    /// Total locked amount in an era
    #[codec(compact)]
    pub locked: Balance,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Encode, Decode)]
pub struct BondStakeInput<Balance: HasCompact> {
    contract_id: AccountId,
    #[codec(compact)]
    value: Balance,
}


#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum DSErrorCode {
    Failed,
}

#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum DSError {
    ErrorCode(DSErrorCode),
}

impl From<DSErrorCode> for DSError {
    fn from(error_code: DSErrorCode) -> Self { Self::ErrorCode(error_code) }
}

impl From<scale::Error> for DSError {
    fn from(_: scale::Error) -> Self {
        panic!("Encountered unexpected error!")
    }
}

impl ink_env::chain_extension::FromStatusCode for DSErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::Failed),
            _ => panic!("Encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = DappsStakingExt;

}




#[ink::contract(env = crate::CustomEnvironment)]
mod dapps_staking_proxy {

    use super::{DSError, EraInfo};

    #[ink(storage)]
    pub struct Contract {
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self { Self {} }

        #[ink(message)]
        pub fn read_current_era(&self) -> u32 {
            self.env().extension().read_current_era()
        }

        #[ink(message)]
        pub fn read_era_info(&self, era: u32) -> Result<EraInfo<Balance>, DSError> {
            self.env().extension().read_era_info(era)
        }

        #[ink(message)]
        pub fn bond_and_stake(&mut self, contract: AccountId, value: Balance) -> Result<(), DSError> {
            //let contract = self.env().account_id();
            self.env().extension().bond_and_stake(contract, value)
        }

    }

}
