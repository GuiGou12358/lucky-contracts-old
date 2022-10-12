use ink_env::Error;
use ink_prelude::vec::Vec;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

#[openbrush::trait_definition]
pub trait Reward {

    /// add the accounts in the list of winners for the given era
    fn _add_winners(&mut self, era: u128, accounts: &Vec<AccountId>) -> Result<PendindReward, RewardError>;

    /// Return true if the the given account has pending rewards
    fn _has_pending_rewards_from(&self, era: Option<u128>, account: Option<AccountId>) -> bool;

    /// claim all pending rewards
    /// After claiming, there is not anymore pending rewards for this account
    fn _claim_from(&mut self, account: AccountId) -> Result<Balance, Error> ;

}

pub struct PendindReward {
    pub era: u128,
    pub given_reward: Balance,
    pub nb_winners: u8
}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RewardError {
    NoReward,
    NoRatioSet,
}
