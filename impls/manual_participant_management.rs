use ink_prelude::vec::Vec;
use openbrush::traits::Storage;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

pub use crate::traits::participant_management::ParticipantManagement;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug )]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    participants: Vec<(AccountId, u128, Balance)>,
}

impl<T: Storage<Data>> ParticipantManagement for T {

    default fn add_participant(&mut self, era: u128, participant: AccountId, value: Balance) {
        self.data().participants.push((participant, era, value)); // TODO test if is already there
    }

    default fn _list_participants(&self, era: u128) -> Vec<(AccountId, Balance)> {
    	self.data().participants.iter()
            .filter(|(_, e, _)| *e == era)
            .map(|(a, _, b)| (*a, *b))
            .collect()
    }

}
