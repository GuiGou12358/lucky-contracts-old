use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::AccessControlError;
use openbrush::traits::{AccountId, Balance};

#[openbrush::wrapper]
pub type Psp22RewardRef = dyn Psp22Reward;

#[openbrush::trait_definition]
pub trait Psp22Reward {

    /// Add the accounts in the list of winners for a given era
    /// accounts contains the list of winners and the rewards by account
    #[ink(message, payable, selector = 0xc218e5ba)]
    fn fund_rewards_and_add_winners(&mut self, era: u32, accounts: Vec<(AccountId, Balance)>) -> Result<(), RewardError>;

    /// return true if the current account has pending rewards
    #[ink(message)]
    fn has_pending_rewards(&self) -> bool;

    fn _has_pending_rewards_from(&self, from: AccountId) -> bool;

    /// return the pending rewards for a given account.
    #[ink(message)]
    fn get_pending_rewards_from(&mut self, from: AccountId) -> Result<Option<Balance>, RewardError>;

    /// claim all pending rewards for the current account
    /// After claiming, there is not anymore pending rewards for this account
    #[ink(message)]
    fn claim(&mut self) -> Result<(), RewardError> ;

    /// claim all pending rewards for the given account
    /// After claiming, there is not anymore pending rewards for this account
    fn _claim_from(&mut self, from: AccountId) -> Result<(), RewardError> ;

}

#[openbrush::trait_definition]
pub trait Internal {
    fn _emit_pending_reward_event(&self, account: AccountId, era: u32, amount: Balance);
    fn _emit_rewards_claimed_event(&self, account: AccountId, amount: Balance);
}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RewardError {
    InsufficientTransferredBalance,
    TransferError,
    AddOverFlow,
    NoReward,
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to RewardError
impl From<AccessControlError> for RewardError {
    fn from(error: AccessControlError) -> Self {
        RewardError::AccessControlError(error)
    }
}


