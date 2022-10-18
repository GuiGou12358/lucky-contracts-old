use ink_prelude::vec::Vec;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;
use openbrush::traits::Storage;
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::RngCore;
use rand_chacha::rand_core::SeedableRng;

pub use crate::traits::raffle::Raffle;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    max_winners_by_raffle: u8,
}

impl Data {

    /// Return the sum of weight for all participants given in parameters
    fn _total_weight(participants: &Vec<(AccountId, u128)>) -> u128 {
        let mut total_weight = 0 ;
        for (_, weight) in participants {
            total_weight += weight;
        }
        total_weight
    }

    /// Iterate on the participants, sum the weights,
    /// and return the participant if the sum is superior to the given weight
    fn _select_winners(participants: &Vec<(AccountId, u128)>, random_weight : u128) -> Option<AccountId> {
        let mut total_weight = 0;
        for (account, weight) in participants {
            total_weight += weight;
            if total_weight >= random_weight {
                return Some(*account);
            }
        }
        None
    }
}


impl<T: Storage<Data>> Raffle for T {

    default fn _set_max_winners_by_raffle(&mut self, max_number: u8){
        self.data().max_winners_by_raffle = max_number;
    }

    default fn _run(&mut self, _era: u128, participants: Vec<(AccountId, Balance)>) -> Vec<AccountId> {
        // initialize the empty list of lucky accounts
        let max_winners = self.data().max_winners_by_raffle as usize;
        let mut winners = Vec::with_capacity(max_winners);
        if participants.len() > 0 {
            // compute the sum of weight of participants
            // TODO we can cap the weight by participant to avoid a whale wins always
            let total_weight = Data::_total_weight(&participants);

            // use the first account to further randomize
            let mut account = participants[0].0;

            let mut unsuccessful_choice = 0;
            loop {
                // generate the random number
                let random_weight = self._get_random_number(0, total_weight, account);
                // select the lucky account
                let winner =  Data::_select_winners(&participants, random_weight);
                if winner.is_some(){
                    let winner = winner.unwrap();
                    if winners.contains(&winner) {
                        // this winners already win => choose another one
                        unsuccessful_choice += 1;
                        if unsuccessful_choice > 10 {
                            // avoid infinite loop
                            break;
                        }
                        // no other account can be chosen
                        if unsuccessful_choice >= participants.len() {
                            break
                        }
                        // change the account to further randomize
                        account = participants[unsuccessful_choice].0;
                    } else {
                        // a lucky has been selected
                        winners.push(winner);
                        // this account will be used to further randomize
                        account = winner;
                        // reset
                        unsuccessful_choice = 0;

                        if winners.len() == max_winners {
                            // all winners have been chosen
                            break;
                        }
                    }

                } else {
                    // we should never go there
                    // otherwise there is an issue in the method function Data::_select_winners
                    break;
                }
            }
        }
        winners
    }


    default fn _get_random_number(&self, min: u128, max: u128, account: AccountId) -> u128 {
        let random_seed = Self::env().random(account.as_ref());
        let mut seed_converted: [u8; 32] = Default::default();
        seed_converted.copy_from_slice(random_seed.0.as_ref());
        let mut rng = ChaChaRng::from_seed(seed_converted);
        let a = rng.next_u64();
        (a  as u128) / (u128::MAX) * (max - min) + min
    }

}
