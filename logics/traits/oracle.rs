use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::AccessControlError;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

#[openbrush::wrapper]
pub type OracleDataConsumerRef = dyn OracleDataConsumer;

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct OracleData {
    /// list all participants for a given era
    pub participants: Vec<(AccountId, Balance)>,
    /// rewards for a given era
    pub rewards: Balance,
}

#[openbrush::trait_definition]
pub trait OracleDataConsumer {

    /// return the data for a given era
    #[ink(message)]
    fn get_data(&self, era: u32) -> OracleData;

}

#[openbrush::trait_definition]
pub trait OracleDataManager {

    
    /// add a participant in the raffle for a given era
    /// a participant with a weight higher than another participant will have normally more chance to be selected in the raffle
    /// weight can represent the number of raffle tickets for this participant.
    /// weight can also represent the amount staked in dAppStaking, ...
    #[ink(message)]
    fn add_participant(&mut self, era: u32, participant: AccountId, weight: Balance) -> Result<(), OracleManagementError>;

    /// add participants in the raffle for a given era
    /// a participant with a weight higher than another participant will have normally more chance to be selected in the raffle
    /// weight can represent the number of raffle tickets for this participant.
    /// weight can also represent the amount staked in dAppStaking, ...
    #[ink(message)]
    fn add_participants(&mut self, era: u32, participants: Vec<(AccountId, Balance)>) -> Result<(), OracleManagementError>;

    /// Set the rewards to share between participants
    #[ink(message)]
    fn set_rewards(&mut self, era: u32, reward: Balance) -> Result<(), OracleManagementError>;

    /// Clear the data (participants and rewards) stored for a given era
    #[ink(message)]
    fn clear_data(&mut self, era: u32) -> Result<(), OracleManagementError>;

}


#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum OracleManagementError {
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to ParticipantManagementError
impl From<AccessControlError> for OracleManagementError {
    fn from(error: AccessControlError) -> Self {
        OracleManagementError::AccessControlError(error)
    }
}