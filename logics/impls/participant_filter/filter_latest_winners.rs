use openbrush::contracts::access_control::{access_control, RoleType};
use openbrush::traits::{AccountId, Storage};
use ink::prelude::vec::Vec;
use ink::prelude::collections::vec_deque::VecDeque;
use crate::traits::participant_filter::participant_filter::ParticipantFilterError;

pub use crate::traits::participant_filter::filter_latest_winners::*;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
pub const PARTICIPANT_FILTER_MANAGER: RoleType = ink::selector_id!("PARTICIPANT_FILTER_MANAGER");

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    nb_filtered_winners: u16,
    /// last winners to exclude
    last_winners: VecDeque<AccountId>
}

impl<T> FilterLatestWinners for T
    where
        T: Storage<Data>,
        T: Storage<access_control::Data>,
{

    #[openbrush::modifiers(access_control::only_role(PARTICIPANT_FILTER_MANAGER))]
    default fn set_nb_winners_filtered(&mut self, nb_filtered_winners: u16) -> Result<(), ParticipantFilterError> {
        self.data::<Data>().nb_filtered_winners = nb_filtered_winners;
        Ok(())
    }

    default fn get_nb_winners_filtered(&self) -> u16 {
        self.data::<Data>().nb_filtered_winners
    }

    default fn _add_winner(&mut self, winner: AccountId) {
        // add the last winner in the back
        self.data::<Data>().last_winners.push_back(winner);
        if self.data::<Data>().last_winners.len() > self.data::<Data>().nb_filtered_winners as usize {
            // remove the oldest winner (from the front)
            self.data::<Data>().last_winners.pop_front();
        }
    }

    default fn get_last_winners(&self) -> Vec<AccountId> {
        Vec::from(self.data::<Data>().last_winners.clone())
    }

    default fn _is_in_last_winners(&self, participant: &AccountId) -> bool {
        self.data::<Data>().last_winners.contains(participant)
    }

}


