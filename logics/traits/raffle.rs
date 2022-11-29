use ink_prelude::vec::Vec;
use openbrush::contracts::access_control::AccessControlError;
use openbrush::traits::{AccountId, Balance};

use crate::helpers::helper::HelperError;

#[openbrush::trait_definition]
pub trait Raffle {

    /// Set the rate sharing by the winners
    /// First winner will receive : total_rewards * ratio[0] / total_ratio
    /// Second winner will receive : total_rewards * ratio[1] / total_ratio
    /// if ratio[n] equals to zero or is empty, tne winner n will receive nothing
    /// Sum(ratio[i]) <= total_ratio. Otherwise teh error IncorrectRatio is expected
    #[ink(message)]
    fn set_ratio_distribution(&mut self, ratio: Vec<Balance>, total_ratio: Balance) -> Result<(), RaffleError>;

    #[ink(message)]
    fn get_ratio_distribution(&self) -> Result<Vec<Balance>, RaffleError>;

    #[ink(message)]
    fn get_last_era_done(&self) -> u32;

    fn _run_raffle(
        &mut self, 
        era: u32,
        participants: Vec<(AccountId, Balance)>,
        total_rewards: Balance
    ) -> Result<Vec<(AccountId, Balance)>, RaffleError>;

}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RaffleError {
    RaffleAlreadyDone,
    NoReward,
    NoRatioSet,
    IncorrectRatio,
    NoParticipant,
    TooManyWinners,
    DivByZero,
    MulOverFlow,
    AddOverFlow,
    HelperError(HelperError),
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to RaffleError
impl From<AccessControlError> for RaffleError {
    fn from(error: AccessControlError) -> Self {
        RaffleError::AccessControlError(error)
    }
}

/// convertor from HelperError to RaffleError
impl From<HelperError> for RaffleError {
    fn from(error: HelperError) -> Self {
        RaffleError::HelperError(error)
    }
}


