use openbrush::contracts::access_control::{access_control, RoleType};
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Balance, Storage};
use ink_prelude::vec::Vec; 

pub use crate::traits::reward::{
    psp22_reward,
    psp22_reward::*,
    psp22_reward::RewardError::*,
};


pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
pub const REWARD_MANAGER: RoleType = ink_lang::selector_id!("REWARD_MANAGER");
pub const REWARD_VIEWER: RoleType = ink_lang::selector_id!("REWARD_VIEWER");

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pending_rewards: Mapping<AccountId, Balance>,
}

impl<T> Psp22Reward for T
where
    T: psp22_reward::Internal,
    T: Storage<Data>,
    T: Storage<access_control::Data>,
{

    #[openbrush::modifiers(access_control::only_role(REWARD_MANAGER))]
    default fn fund_rewards_and_add_winners(&mut self, era: u32, accounts: Vec<(AccountId, Balance)>) -> Result<(), RewardError> {

        let transferred_value = Self::env().transferred_value();
        let mut total_rewards = Balance::default();

        // iterate on the accounts (the winners)
        for (account, reward) in accounts {
            
            total_rewards = total_rewards.checked_add(reward).ok_or(AddOverFlow)?;

            // compute the new rewards for this winner
            let new_reward = match self.data::<Data>().pending_rewards.get(&account){
                Some(existing_reward) => {existing_reward.checked_add(reward).ok_or(AddOverFlow)?}
                _ => {reward}
            };

            // add the pending rewards for this account
            self.data::<Data>().pending_rewards.insert(&account, &new_reward);

            self._emit_pending_reward_event(account, era, reward);
        }

        if transferred_value < total_rewards {
            return Err(InsufficientTransferredBalance);
        }

        Ok(())
    }

    default fn has_pending_rewards(&self) -> bool{
        let from = Self::env().caller();
        self._has_pending_rewards_from(from)
    }

    default fn _has_pending_rewards_from(&self, from: AccountId) -> bool{
        self.data::<Data>().pending_rewards.contains(&from)
    }


    #[openbrush::modifiers(access_control::only_role(REWARD_VIEWER))]
    default fn get_pending_rewards_from(&mut self, from: AccountId) -> Result<Option<Balance>, RewardError> {
        Ok(self.data::<Data>().pending_rewards.get(&from))
    }

    default fn claim(&mut self) -> Result<(), RewardError> {
        let from = Self::env().caller();
        self._claim_from(from)
    }

    default fn _claim_from(&mut self, from: AccountId) -> Result<(), RewardError>  {
        // get all pending rewards for this account
        match self.data::<Data>().pending_rewards.get(&from) {
            Some(pending_rewards) => {
                // transfer the amount
                Self::env().transfer(from, pending_rewards).map_err(|_| TransferError)?;
                // emmit the event
                self._emit_rewards_claimed_event(from, pending_rewards);
                // remove the pending rewards
                self.data::<Data>().pending_rewards.remove(&from);
                Ok(())
            }
            _ => Err(NoReward)
        }
    }

}


