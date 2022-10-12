use ink_prelude::vec::Vec;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;
use crate::traits::reward::reward::Reward;


pub trait Psp22Reward : Reward {

    /// Set the rate sharing by teh winners
    /// First winner will receive (total_rewards * ratio[0]) / sum(ratio)
    /// Second winner will receive (total_rewards * ratio[1]) / sum(ratio)
    /// if ration[n] equals to zero or is empty, tne winner n will receice nothing
    fn _set_ratio_distribution(&mut self, ratio: Vec<Balance>);

    /// Set the total rewards shared by all wiiners for a given era
    fn _set_total_rewards(&mut self, era: u128, amount: Balance);

    /// return the pending rewards for a given era and a given account.
    /// If the era is None, teh funnction returns the pending rewards for all era
    /// If the account is None, teh funnction returns the pending rewards for all accounts
    fn _list_pending_rewards_from(&self, era: Option<u128>, account: Option<AccountId>) -> Vec<(AccountId, u128, Balance)>;

}


