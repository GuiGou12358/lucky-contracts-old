use ink_env::Error;
use ink_prelude::vec::Vec;
//use ink_prelude::collections::HashMap;
//use ink_storage::traits::SpreadLayout;
//use openbrush::traits::OccupyStorage;
//use openbrush::traits::StorageAsRef;
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Balance, Storage};

pub use Internal as _;

pub use crate::traits::reward::psp22_reward::Psp22Reward;
pub use crate::traits::reward::reward::{
    Reward,
    RewardError,
    RewardError::*
};
pub use crate::traits::reward::reward::PendindReward;

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

impl<T: Storage<Data> + Internal> Reward for T {

    default fn _add_winners(&mut self, era: u128, accounts: &Vec<AccountId>) -> Result<PendindReward, RewardError> {

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
        let mut given_reward = 0;
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
            let amount = era_reward * ratio / self.data().total_ratio_distribution;
            // add the pending rewards for this account
            self.data().pending_rewards.push((*account, era, amount));
            given_reward +=  amount;
            nb_winners += 1;
            index += 1;
        }

        // update the remaining rewards for this era
        if era_reward <= given_reward {
            self.data().remaining_rewards.remove(&era);
        } else {
            self.data().remaining_rewards.insert(&era, &(era_reward - given_reward));
        }

        Ok(PendindReward {era, given_reward, nb_winners})
    }


    default fn _has_pending_rewards_from(&self, era: Option<u128>, account: Option<AccountId>) -> bool{
        for (a, e, _) in &self.data().pending_rewards {
            let era_match = era.unwrap_or(*e) == *e;
            let account_match = account.unwrap_or(*a) == *a;
            if era_match && account_match{
                return true;
            }
        }
        false
    }

    default fn _claim_from(&mut self, account: AccountId) -> Result<Balance, Error>  {
        // get all pending rewards for this account
        let mut pending_rewards = Balance::default();
        let mut index_to_remove = Vec::new();
        let mut index = 0;
        for (a, _, b) in &self.data().pending_rewards {
            if account == *a {
                // aggregate the rewards
                pending_rewards += *b;
                // remove this index
                index_to_remove.push(index);
            }
            index += 1;
        }
        // transfer the amount
        self._transfer_to(account, pending_rewards)?;
        // remove the rewards for this account
        for i in index_to_remove.iter().rev() {
            self.data().pending_rewards.remove(*i);
        }
        Ok(pending_rewards)
    }

}

pub trait Internal {
    fn _transfer_to(&mut self, to: AccountId, amount: Balance) -> Result<(), Error>;

    //fn _emit_event(&mut self, account: AccountId, amount: Balance);
}

impl<T: Storage<Data>> Psp22Reward for T {

    default fn _set_ratio_distribution(&mut self, ratio: Vec<Balance>){
        self.data().ratio_distribution = ratio;
        let mut total = 0;
        for b in &self.data().ratio_distribution {
            total += *b;
        }
        self.data().total_ratio_distribution = total;
    }

    default fn _set_total_rewards(&mut self, era: u128, amount: Balance){
        self.data().remaining_rewards.insert(&era, &amount);
        /*
        let position = self.data().total_rewards.iter().position(|(e, _)| *e == era);
        if position.is_some(){
            self.data().total_rewards.remove(position.unwrap());
        }
        self.data().total_rewards.push((era, amount));
        */
    }

    default fn _list_pending_rewards_from(&self, era: Option<u128>, account: Option<AccountId>) -> Vec<(AccountId, u128, Balance)>{
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


