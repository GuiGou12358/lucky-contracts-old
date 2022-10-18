use openbrush::traits::Storage;

pub use crate::traits::game::Game;
use crate::traits::game::GameError;
use crate::traits::participant_management::ParticipantManagement;
use crate::traits::raffle::Raffle;
use crate::traits::reward::psp22_reward::{PendingReward, Psp22Reward};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug )]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    _reserved: Option<()>,
}

impl<T: Storage<Data> + ParticipantManagement + Raffle + Psp22Reward> Game for T {

    default fn _play(&mut self, era: u128) -> Result<PendingReward, GameError> {
        let participants = self._list_participants(era);
        let winners = self._run(era, participants)?;
        match self._add_winners(era, &winners){
            Ok(pr) => Ok(pr),
            Err(e) => Err(GameError::RewardError(e)),
        }
    }

}


