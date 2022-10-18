use crate::traits::reward::psp22_reward::{PendingReward, RewardError};

#[openbrush::trait_definition]
pub trait Game {

    /// Play the raffle for the given era
    /// Based on the configuration, a number of accounts are randomly selected among the participants
    /// and rewards are dispatched between the winners.
    ///
    /// Return the number of winners and the total of rewards dispatched between them
    fn _play(&mut self, era: u128) -> Result<PendingReward, RewardError> ;

}
