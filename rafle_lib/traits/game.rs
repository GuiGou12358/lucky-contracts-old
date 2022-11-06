use crate::traits::raffle::RaffleError;
use crate::traits::reward::psp22_reward::{PendingReward, RewardError};

#[openbrush::trait_definition]
pub trait Game {

    /// Play the raffle for the given era
    /// Based on the configuration, a number of accounts are randomly selected among the participants
    /// and rewards are dispatched between the winners.
    ///
    /// Return the number of winners and the total of rewards dispatched between them
    fn _play(&mut self, era: u32) -> Result<PendingReward, GameError> ;

}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum GameError {
    RewardError(RewardError),
    RaffleError(RaffleError),
}

/// convertor from RewardError to GameError
impl From<RewardError> for GameError {
    fn from(error: RewardError) -> Self {
        GameError::RewardError(error)
    }
}

/// convertor from RaffleError to GameError
impl From<RaffleError> for GameError {
    fn from(error: RaffleError) -> Self {
        GameError::RaffleError(error)
    }
}
