use ink_prelude::vec::Vec;
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Balance, Storage};

pub use crate::traits::reward::psp22_reward::{
    PendingReward,
    Psp22Reward,
    RewardError,
    RewardError::*
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    /// Extension to interact with `pallet-assets`
    //pub pallet_assets: AssetsExtension,
    pending_rewards: Vec<(AccountId, u128, Balance)>,
    remaining_rewards: Mapping<u128, Balance>,
    ratio_distribution: Vec<Balance>,
    total_ratio_distribution: Balance,
}


/*
pub trait Internal {
    fn _transfer_to(&mut self, to: AccountId, amount: Balance) -> Result<(), Error>;

    //fn _emit_event(&mut self, account: AccountId, amount: Balance);
}
 */

impl<T: Storage<Data>> Psp22Reward for T {

    default fn _set_ratio_distribution(&mut self, ratio: Vec<Balance>){
        self.data().ratio_distribution = ratio;
        let mut total = 0;
        for b in &self.data().ratio_distribution {
            total += *b;
        }
        self.data().total_ratio_distribution = total;
    }

    default fn set_total_rewards(&mut self, era: u128, amount: Balance){
        self.data().remaining_rewards.insert(&era, &amount);
    }

    default fn _add_winners(&mut self, era: u128, accounts: &Vec<AccountId>) -> Result<PendingReward, RewardError> {

        // get the remaining rewards for this era
        let era_reward = self.data().remaining_rewards.get(&era).ok_or(NoReward)?;
        if era_reward <= 0{
            // no reward for era
            return Err(NoReward);
        }

        if self.data().ratio_distribution.len() == 0 {
            // no ration set
            return Err(NoRatioSet);
        }

        let mut index = 0;
        let mut given_reward: Balance = 0;
        let mut nb_winners = 0;

        // iterate on the accounts (the winners)
        for account in accounts {
            if self.data().ratio_distribution.len() < index {
                // no reward for this winner and for the next one
                break;
            }
            let ratio = self.data().ratio_distribution.get(index).unwrap_or(&0);
            if *ratio == 0 {
                // no reward for this winner but maybe there will be some rewards for next one
                continue;
            }
            // compute the reward for this winner based on ratio
            let amount = era_reward
                .checked_mul(*ratio).ok_or(MulOverFlow)?
                .checked_div(self.data().total_ratio_distribution).ok_or(DivByZero)?;
            // add the pending rewards for this account
            self.data().pending_rewards.push((*account, era, amount));
            given_reward =  given_reward.checked_add(amount).ok_or(AddOverFlow)?;
            nb_winners += 1;
            index += 1;
        }

        // update the remaining rewards for this era
        if era_reward <= given_reward {
            self.data().remaining_rewards.remove(&era);
        } else {
            self.data().remaining_rewards.insert(&era, &(era_reward - given_reward));
        }

        Ok(PendingReward {era, given_reward, nb_winners})
    }

    default fn has_pending_rewards(&mut self) -> Result<bool, RewardError> {
        let from = Self::env().caller();
        self._has_pending_rewards_from(None, Some(from))
    }

    default fn _has_pending_rewards_from(&mut self, era: Option<u128>, from: Option<AccountId>) -> Result<bool, RewardError> {
        for (a, e, _) in &self.data().pending_rewards {
            let era_match = era.unwrap_or(*e) == *e;
            let account_match = from.unwrap_or(*a) == *a;
            if era_match && account_match {
                return Ok(true);
            }
        }
        Ok(false)
    }


    default fn claim(&mut self) -> Result<Balance, RewardError> {
        let from = Self::env().caller();
        self._claim_from(from)
    }

    default fn _claim_from(&mut self, from: AccountId) -> Result<Balance, RewardError>  {
        // get all pending rewards for this account
        let mut pending_rewards = Balance::default();
        let mut index_to_remove = Vec::new();
        let mut index = 0;
        for (a, _, b) in &self.data().pending_rewards {
            if from == *a {
                // aggregate the rewards
                pending_rewards = pending_rewards.checked_add(*b).ok_or(AddOverFlow)?;
                // remove this index
                index_to_remove.push(index);
            }
            index += 1;
        }
        // transfer the amount
        if Self::env().transfer(from, pending_rewards).is_err(){
            return Err(TransferError);
        }
        //self._transfer_to(account, pending_rewards)?;
        // remove the rewards for this account
        for i in index_to_remove.iter().rev() {
            self.data().pending_rewards.remove(*i);
        }
        Ok(pending_rewards)
    }


    default fn list_pending_rewards_from(&self, era: Option<u128>, account: Option<AccountId>) -> Vec<(AccountId, u128, Balance)>{
        let mut pending_rewards = Vec::new();
        for (a, e, b) in &self.data().pending_rewards {
            let era_match = era.unwrap_or(*e) == *e;
            let account_match = account.unwrap_or(*a) == *a;
            if era_match && account_match{
                pending_rewards.push((*a, *e, *b))
            }
        }
        pending_rewards
    }

}


