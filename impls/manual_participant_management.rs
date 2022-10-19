use ink_prelude::vec::Vec;
use openbrush::contracts::access_control::{access_control, RoleType};
use openbrush::traits::AccountId;
use openbrush::traits::Balance;
use openbrush::traits::Storage;

pub use crate::traits::participant_management::*;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
pub const PARTICIPANT_MANAGER: RoleType = ink_lang::selector_id!("PARTICIPANT_MANAGER");

#[derive(Default, Debug )]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    participants: Vec<(AccountId, u128, Balance)>,
}

impl<T> ParticipantManagement for T
    where
        T: Storage<Data>,
        T: Storage<access_control::Data>,
{

    #[openbrush::modifiers(access_control::only_role(PARTICIPANT_MANAGER))]
    default fn add_participant(&mut self, era: u128, participant: AccountId, value: Balance) -> Result<(), ParticipantManagementError> {
        self.data::<Data>().participants.push((participant, era, value)); // TODO test if is already there
        Ok(())
    }

    default fn _list_participants(&self, era: u128) -> Vec<(AccountId, Balance)> {
    	self.data::<Data>().participants.iter()
            .filter(|(_, e, _)| *e == era)
            .map(|(a, _, b)| (*a, *b))
            .collect()
    }

}
