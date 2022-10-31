use ink_prelude::vec::Vec;
use openbrush::contracts::access_control::{access_control, RoleType};
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Balance, Storage};

pub use crate::traits::reward::psp22_reward::{
    *,
    RewardError::*
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
pub const REWARD_MANAGER: RoleType = ink_lang::selector_id!("REWARD_MANAGER");
pub const REWARD_VIEWER: RoleType = ink_lang::selector_id!("REWARD_VIEWER");

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

impl<T> Psp22Reward for T
where
    T: Internal,
    T: Storage<Data>,
    T: Storage<access_control::Data>,
{

    default fn _set_ratio_distribution(&mut self, ratio: Vec<Balance>) -> Result<(), RewardError>{
        self.data::<Data>().ratio_distribution = ratio;
        let mut total = 0;
        for b in &self.data::<Data>().ratio_distribution {
            //total += b;
            total = b.checked_add(total).ok_or(AddOverFlow)?;
        }
        self.data::<Data>().total_ratio_distribution = total;
        Ok(())
    }

    #[openbrush::modifiers(access_control::only_role(REWARD_MANAGER))]
    default fn fund_rewards(&mut self, era: u128, amount: Balance) -> Result<(), RewardError> {

        let transferred_value = Self::env().transferred_value();
        let caller = Self::env().caller();
        ink_env::debug_println!("Thanks for the funding of {:?} from {:?}", transferred_value, caller);

        if transferred_value < amount {
            return Err(InsufficientTransferredBalance);
        }
        self.data::<Data>().remaining_rewards.insert(&era, &amount);
        Ok(())
    }

    default fn _add_winners(&mut self, era: u128, accounts: &Vec<AccountId>) -> Result<PendingReward, RewardError> {

        // get the remaining rewards for this era
        let era_reward = self.data::<Data>().remaining_rewards.get(&era).ok_or(NoReward)?;
        if era_reward <= 0{
            // no reward for era
            return Err(NoReward);
        }

        if self.data::<Data>().ratio_distribution.len() == 0 {
            // no ration set
            return Err(NoRatioSet);
        }

        let mut index = 0;
        let mut given_reward: Balance = 0;
        let mut nb_winners = 0;

        // iterate on the accounts (the winners)
        for account in accounts {
            if self.data::<Data>().ratio_distribution.len() < index {
                // no reward for this winner and for the next one
                break;
            }
            let ratio = self.data::<Data>().ratio_distribution.get(index).unwrap_or(&0);
            if *ratio == 0 {
                // no reward for this winner but maybe there will be some rewards for next one
                continue;
            }
            // compute the reward for this winner based on ratio
            let amount = era_reward
                .checked_mul(*ratio).ok_or(MulOverFlow)?
                .checked_div(self.data::<Data>().total_ratio_distribution).ok_or(DivByZero)?;
            // add the pending rewards for this account
            self.data::<Data>().pending_rewards.push((*account, era, amount));
            given_reward =  given_reward.checked_add(amount).ok_or(AddOverFlow)?;
            nb_winners += 1;
            index += 1;
        }

        // update the remaining rewards for this era
        if era_reward <= given_reward {
            self.data::<Data>().remaining_rewards.remove(&era);
        } else {
            self.data::<Data>().remaining_rewards.insert(&era, &(era_reward - given_reward));
        }

        Ok(PendingReward {era, given_reward, nb_winners})
    }

    default fn has_pending_rewards(&mut self) -> Result<bool, RewardError> {
        let from = Self::env().caller();
        self._has_pending_rewards_from(None, Some(from))
    }

    default fn _has_pending_rewards_from(&mut self, era: Option<u128>, from: Option<AccountId>) -> Result<bool, RewardError> {
        for (a, e, _) in &self.data::<Data>().pending_rewards {
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
        for (a, _, b) in &self.data::<Data>().pending_rewards {
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
            self.data::<Data>().pending_rewards.remove(*i);
        }

        // emit the events
        if pending_rewards > 0 {
            self._emit_reward_claimed_event(from, pending_rewards);
        }

        Ok(pending_rewards)
    }


    #[openbrush::modifiers(access_control::only_role(REWARD_VIEWER))]
    default fn list_pending_rewards_from(&mut self, era: Option<u128>, account: Option<AccountId>)
        -> Result<Vec<(AccountId, u128, Balance)>, RewardError> {
        let mut pending_rewards = Vec::new();
        for (a, e, b) in &self.data::<Data>().pending_rewards {
            let era_match = era.unwrap_or(*e) == *e;
            let account_match = account.unwrap_or(*a) == *a;
            if era_match && account_match{
                pending_rewards.push((*a, *e, *b))
            }
        }
        Ok(pending_rewards)
    }

}


