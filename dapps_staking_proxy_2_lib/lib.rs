#![cfg_attr(not(feature = "std"), no_std)]

use scale::{Decode, Encode, HasCompact};

//use frame_support::pallet_prelude::MaxEncodedLen;
use ink_env::{AccountId, DefaultEnvironment, Environment};
pub type Balance = <DefaultEnvironment as Environment>::Balance;


pub struct DAppsStaking;

impl DAppsStaking {

    pub fn read_current_era() -> u32 {
        ink_env::chain_extension::ChainExtensionMethod::build(0801u32) // 0801 in Swanky node - 3401 in Shiden/Astar
            .input::<()>()
            .output::<u32>()
            .ignore_error_code()
            .call(&())
    }

    pub fn read_era_info(era: u32) -> Result<EraInfo<Balance>, DSError> {
        ink_env::chain_extension::ChainExtensionMethod::build(0802u32) // 0802 in Swanky node - 3402 in Shiden/Astar
            .input::<u32>()
            .output::<Result<EraInfo<Balance>, DSError>>()
            .handle_error_code::<DSError>()
            .call(&era)?
    }


    pub fn bond_and_stake(account_id: AccountId, value: Balance) -> Result<(), DSError> {
        let input = BondStakeInput {account_id, value};
        ink_env::chain_extension::ChainExtensionMethod::build(0803u32) // 0803 in Swanky node - 3403 in Shiden/Astar
            .input::<BondStakeInput>()
            .output::<Result<(), DSError>>()
            .handle_error_code::<DSError>()
            .call(&input)?
    }
}

/// A record of rewards allocated for stakers and dapps
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode)]
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
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode)]
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


#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
pub struct BondStakeInput {
    account_id: AccountId,
    value: Balance,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum DSError {
    Failed,
}

/// convertor from RewardError to ContractError
impl From<scale::Error> for DSError {
    fn from(_: scale::Error) -> Self {
        panic!("Encountered unexpected error!")
    }
}

impl ink_env::chain_extension::FromStatusCode for DSError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::Failed),
            _ => panic!("Encountered unknown status code"),
        }
    }
}