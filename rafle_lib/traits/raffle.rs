use ink_prelude::vec::Vec;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

#[openbrush::trait_definition]
pub trait Raffle {

    /// set the max number of winners selected by rafle
    /// This max number is set for all era
    fn _set_max_winners_by_raffle(&mut self, max_number: u8);

    /// Run the raffle and return the list of winners
    fn _run(&mut self, era: u32, participants: Vec<(AccountId, Balance)>) -> Result<Vec<AccountId>, RaffleError>;

    /// generate a random number between min and max values.
    /// The subject can be used to further randomize the number.
    fn _get_random_number(&self, min: u128, max: u128, subject: AccountId) -> Result<u128, RaffleError> ;

}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RaffleError {
    DivByZero,
    MulOverFlow,
    AddOverFlow,
    SubOverFlow,
}

