use ink::prelude::vec::Vec;
use openbrush::traits::AccountId;
use crate::traits::participant_filter::participant_filter::ParticipantFilterError;

#[openbrush::trait_definition]
pub trait FilterLatestWinners {

    #[ink(message)]
    fn set_nb_winners_filtered(&mut self, nb_filtered_winners: u16) -> Result<(), ParticipantFilterError>;

    #[ink(message)]
    fn get_nb_winners_filtered(&self) -> u16;

    fn _add_winner(&mut self, winner: AccountId);

    #[ink(message)]
    fn get_last_winners(&self) -> Vec<AccountId>;

    fn _is_in_last_winners(&self, participant: &AccountId) -> bool;

}

