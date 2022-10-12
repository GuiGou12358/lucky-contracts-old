use ink_prelude::vec::Vec;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

#[openbrush::trait_definition]
pub trait Rafle {

    /// Run the rafle and return the list of winners
    fn _run(&mut self, era: u128, participants: Vec<(AccountId, Balance)>) -> Vec<AccountId>;

}

#[openbrush::trait_definition]
pub trait RandomGenerator {

    /// generate a random number between min and max values.
    /// The subject can be used to further randomize the number.
    fn _get_random_number(&self, min: u128, max: u128, subject: AccountId) -> u128;

}

