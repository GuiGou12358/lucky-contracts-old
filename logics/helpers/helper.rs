use ink_env::debug_println;
use ink_prelude::vec::Vec;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

use crate::traits::random_generator::{
    RandomGenerator,
    RandomGeneratorError
};

/// Return the sum of weight for all participants given in parameters
fn total_weight(participants: &Vec<(AccountId, u128)>) -> Result<u128, HelperError>  {
    let mut total_weight = 0 ;
    for (_, weight) in participants {
        //total_weight += weight;
        total_weight = weight.checked_add(total_weight).ok_or(HelperError::AddOverFlow)?;
    }
    Ok(total_weight)
}


/// Iterate on the participants, sum the weights,
/// and return the participant if the sum is superior to the given weight
fn select_winner_matching_weight(
    participants: &Vec<(AccountId, u128)>, random_weight : u128
) -> Result<Option<AccountId>, HelperError> {
    let mut total_weight = 0;
    for (account, weight) in participants {
        total_weight = weight.checked_add(total_weight).ok_or(HelperError::AddOverFlow)?;
        if total_weight >= random_weight {
            return Ok(Some(*account));
        }
    }
    Ok(None)
}

pub fn select_winners(
    random_generator: &dyn RandomGenerator,
    participants: Vec<(AccountId, Balance)>,
    nb_winners: usize
) -> Result<Vec<AccountId>, HelperError>  {
    // initialize the empty list of lucky accounts
    let mut winners = Vec::with_capacity(nb_winners);
    if participants.len() > 0 {
        // compute the sum of weight of participants
        // TODO we can cap the weight by participant to avoid a whale wins always
        let total_weight = total_weight(&participants)?;

        // use the first account to further randomize
        let mut account = participants[0].0;

        let mut unsuccessful_choice = 0;
        loop {
            // generate the random number
            let random_weight = random_generator.get_random_number(0, total_weight, account)?;

            debug_println!("random_weight: {} - total_weight: {}", random_weight, total_weight);

            // select the lucky account
            let winner =  select_winner_matching_weight(&participants, random_weight)?;
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

                    if winners.len() == nb_winners {
                        // all winners have been chosen
                        break;
                    }
                }

            } else {
                // we should never go there
                // otherwise there is an issue in the method function select_winners
                break;
            }
        }
    }
    Ok(winners)
}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum HelperError {
    DivByZero,
    MulOverFlow,
    AddOverFlow,
    SubOverFlow,
    RandomGeneratorError(RandomGeneratorError),
}

/// convertor from RandomGeneratorError to HelperError
impl From<RandomGeneratorError> for HelperError {
    fn from(error: RandomGeneratorError) -> Self {
        HelperError::RandomGeneratorError(error)
    }
}