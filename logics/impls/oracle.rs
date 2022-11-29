use ink_prelude::vec::Vec;
use openbrush::contracts::access_control::{access_control, RoleType};
use openbrush::traits::AccountId;
use openbrush::traits::Balance;
use openbrush::traits::Storage;
use openbrush::storage::Mapping;

pub use crate::traits::oracle::*;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
pub const ORACLE_DATA_MANAGER: RoleType = ink_lang::selector_id!("ORACLE_DATA_MANAGER");

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    participants: Vec<(AccountId, u32, Balance)>,
    rewards: Mapping<u32, Balance>,
}

impl<T> OracleDataConsumer for T
    where
        T: Storage<Data>,
        T: Storage<access_control::Data>,
{

    default fn get_data(&self, era: u32) -> OracleData {
    	let participants = self.data::<Data>().participants.iter()
            .filter(|(_, e, _)| *e == era)
            .map(|(a, _, b)| (*a, *b))
            .collect();
        let rewards = self.data::<Data>().rewards.get(&era).unwrap_or(0);

        OracleData {participants, rewards}
    }

}

impl<T> OracleDataManager for T
    where
        T: Storage<Data>,
        T: Storage<access_control::Data>,
{

    #[openbrush::modifiers(access_control::only_role(ORACLE_DATA_MANAGER))]
    default fn add_participant(&mut self, era: u32, participant: AccountId, value: Balance) -> Result<(), OracleManagementError> {
        self.data::<Data>().participants.push((participant, era, value));
        Ok(())
    }

    #[openbrush::modifiers(access_control::only_role(ORACLE_DATA_MANAGER))]
    default fn add_participants(&mut self, era: u32, participants: Vec<(AccountId, Balance)>) -> Result<(), OracleManagementError> {
        for (participant, value) in participants {
            self.add_participant(era, participant, value)?;
        }
        Ok(())
    }

    #[openbrush::modifiers(access_control::only_role(ORACLE_DATA_MANAGER))]
    default fn set_rewards(&mut self, era: u32, value: Balance) -> Result<(), OracleManagementError> {
        self.data::<Data>().rewards.insert(&era, &value);
        Ok(())
    }

}
