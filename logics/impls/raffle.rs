use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::{access_control, RoleType};
use openbrush::traits::{AccountId, Balance, Storage};

pub use crate::traits::raffle::{
    *,
    RaffleError::*,
};

pub use crate::traits::random::{Random, RandomError};


use crate::impls::participant_manager::ParticipantManager;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
pub const RAFFLE_MANAGER: RoleType = ink::selector_id!("RAFFLE_MANAGER");

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    ratio_distribution: Vec<Balance>,
    total_ratio_distribution: Balance,
    last_era_done: u32,
}

impl<T> Raffle for T
where
    T: Storage<Data>,
    T: Storage<access_control::Data>,
    T: Random,
    T: ParticipantManager,
{

    #[openbrush::modifiers(access_control::only_role(RAFFLE_MANAGER))]
    default fn set_ratio_distribution(&mut self, ratio: Vec<Balance>, total_ratio: Balance) -> Result<(), RaffleError> {

        let mut total = 0;
        for r in &ratio {
            total = r.checked_add(total).ok_or(AddOverFlow)?;
        }
        if total > total_ratio {
            return Err(IncorrectRatio);
        }

        self.data::<Data>().ratio_distribution = ratio;
        self.data::<Data>().total_ratio_distribution = total_ratio;
        Ok(())
    }

    default fn get_ratio_distribution(&self) -> Vec<Balance> {
        let ratio = &self.data::<Data>().ratio_distribution;
        ratio.to_vec()
    }

    default fn get_total_ratio_distribution(&self) -> Balance {
        self.data::<Data>().total_ratio_distribution
    }

    default fn get_last_era_done(&self) -> u32 {
        self.data::<Data>().last_era_done
    }

    #[openbrush::modifiers(access_control::only_role(RAFFLE_MANAGER))]
    default fn _run_raffle(
        &mut self,
        era: u32,
        total_rewards: Balance
    ) -> Result<Vec<(AccountId, Balance)>, RaffleError> {

        // check if the raffle has not been done
        if self.get_last_era_done() >= era {
            return Err(RaffleAlreadyDone);
        }

        let nb_winners = self.data::<Data>().ratio_distribution.len();

        if nb_winners == 0 {
            // no ration set
            return Err(NoRatioSet);
        }

        if total_rewards <= 0 {
            // no reward 
            return Err(NoReward);
        }

        if self.get_nb_participants() == 0 {
            // no participant
            return Err(NoParticipant);
        }

        // total value locked by all participants
        let total_value = self.get_total_value();
        // initialize the empty list of randomly selected values
        let mut winner_and_reward = Vec::with_capacity(nb_winners);

        for i in 0..nb_winners {
            // generate the random value
            let random_value = self.get_random_number(0, total_value)?;
            // select the participant matching with this value
            let winner = self.get_participant(random_value).ok_or(NoSelectedParticipant)?;

            // select the erwards ratio
            let ratio = self.data::<Data>().ratio_distribution.get(i).unwrap_or(&0);
            if *ratio != 0 {
                // compute the reward for this winner based on the ratio
                let amount = total_rewards
                    .checked_mul(*ratio).ok_or(MulOverFlow)?
                    .checked_div(self.data::<Data>().total_ratio_distribution).ok_or(DivByZero)?;
                // add the pending rewards for this account
                winner_and_reward.push((winner, amount));
            }
        }

        // set the raffle is done
        self.data::<Data>().last_era_done = era;

        Ok(winner_and_reward)
    
    }



}


