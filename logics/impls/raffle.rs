use ink_prelude::vec::Vec;
use openbrush::contracts::access_control::{access_control, RoleType};
use openbrush::traits::{AccountId, Balance, Storage};

pub use crate::traits::raffle::{
    *,
    RaffleError::*,
};

use crate::traits::random_generator::RandomGenerator;

use crate::helpers::helper;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
pub const RAFFLE_MANAGER: RoleType = ink_lang::selector_id!("RAFFLE_MANAGER");

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
    T: RandomGenerator,
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

    default fn get_ratio_distribution(&self) -> Result<Vec<Balance>, RaffleError> {
        let ratio = &self.data::<Data>().ratio_distribution;
        Ok(ratio.to_vec())
    }    

    default fn get_last_era_done(&self) -> u32 {
        self.data::<Data>().last_era_done
    }

    #[openbrush::modifiers(access_control::only_role(RAFFLE_MANAGER))]
    default fn _run_raffle(
        &mut self,
        era: u32,
        participants: Vec<(AccountId, Balance)>,
        total_rewards: Balance
    ) -> Result<Vec<(AccountId, Balance)>, RaffleError> {

        // check if the raffle has not been done
        if self.get_last_era_done() >= era{
            return Err(RaffleAlreadyDone);
        }

        let nb_winners = self.data::<Data>().ratio_distribution.len();

        if nb_winners == 0 {
            // no ration set
            return Err(NoRatioSet);
        }

        if total_rewards <= 0{
            // no reward 
            return Err(NoReward);
        }

        if participants.len() == 0{
            // no participant
            return Err(NoParticipant);
        }

        // select the participants
        let winners = helper::select_winners(self, participants, nb_winners)?; 

        if winners.len() > nb_winners {
            // too many winners!
            return Err(TooManyWinners);
        }


        let mut winner_and_reward = Vec::<(AccountId, Balance)>::with_capacity(winners.len() as usize);

        let mut index = 0;
    
        // iterate on the winners
        for winner in winners {

            let ratio = self.data::<Data>().ratio_distribution.get(index).unwrap_or(&0);
            if *ratio != 0 {
                // compute the reward for this winner based on ratio
                let amount = total_rewards
                    .checked_mul(*ratio).ok_or(MulOverFlow)?
                    .checked_div(self.data::<Data>().total_ratio_distribution).ok_or(DivByZero)?;
                // add the pending rewards for this account
                winner_and_reward.push((winner, amount));
            }
            index += 1;
        }

        // set the raffle is done
        self.data::<Data>().last_era_done = era;

        Ok(winner_and_reward)
    
    }



}


