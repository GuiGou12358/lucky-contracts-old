use ink_prelude::vec::Vec;
use openbrush::traits::{DefaultEnv, Storage};
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::RngCore;
use rand_chacha::rand_core::SeedableRng;

pub use crate::traits::rafle::Rafle;
pub use crate::traits::rafle::RandomGenerator;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    nb_winners_by_rafle: u8,
}

impl Data {

    /// set the max number of winners selected by rafle
    /// This max number is set for all era
    pub fn set_nb_winners_by_rafle(&mut self, max_number: u8){
        self.nb_winners_by_rafle = max_number;
    }

    /// Return the sum of weight for all participants given in parameters
    fn total_weight(participants: &Vec<(AccountId, u128)>) -> u128 {
        let mut total_weight = 0 ;
        for (_, weight) in participants {
            total_weight += weight;
        }
        total_weight
    }

    /// Iterate on the participants, sum the weights,
    /// and return the participant if the sum is superior to the given weight
    fn select_winners(participants: &Vec<(AccountId, u128)>, random_weight : u128) -> Option<AccountId> {
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


impl<T: DefaultEnv> RandomGenerator for T {

    default fn _get_random_number(&self, min: u128, max: u128, account: AccountId) -> u128 {
        let random_seed = Self::env().random(account.as_ref());
        let mut seed_converted: [u8; 32] = Default::default();
        seed_converted.copy_from_slice(random_seed.0.as_ref());
        let mut rng = ChaChaRng::from_seed(seed_converted);
        let a = rng.next_u64();
        (a  as u128) / (u128::MAX) * (max - min) + min
    }

}



impl<T: Storage<Data> + RandomGenerator> Rafle for T {

    default fn _run(&mut self, _era: u128, participants: Vec<(AccountId, Balance)>) -> Vec<AccountId> {
        // initialize the empty list of lucky accounts
        let max = self.data().nb_winners_by_rafle as usize;
        let mut winners = Vec::with_capacity(max);
        if participants.len() > 0 {
            // cumpute the sum of weight of participants
            // TODO we can cap the weight by participant to avoid a whale wins always
            let total_weight = Data::total_weight(&participants);
            let random_generator : &mut dyn RandomGenerator = self;

            // use the first account to further randomize
            let mut account = participants[0].0;

            let mut unsuccessful_choice = 0;
            for _i in 0..max {
                // generate teh random number
                let random_weight = random_generator._get_random_number(0, total_weight, account);
                // select the lucky account
                let winner =  Data::select_winners(&participants, random_weight);
                if winner.is_some(){
                    let winner = winner.unwrap();
                    if winners.contains(&winner) {
                        // this winners already win => choose another one
                        unsuccessful_choice += 1;
                        // avoid infinite loop
                        if unsuccessful_choice > 10 {
                            break;
                        }
                        // no other account can be choosen
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
                    }

                } else {
                    // we should never go there
                    // otherwhise there is an issue in the method function Data::select_winners
                    break;
                }
            }
        }
        winners
    }

}



