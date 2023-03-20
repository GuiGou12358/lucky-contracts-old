use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::AccessControlError;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
pub struct Participant {
    pub account: AccountId,
    pub value: Balance,
}

#[openbrush::trait_definition]
pub trait ParticipantManager {

    #[ink(message)]
    fn get_nb_participants(&self) -> u16;

    #[ink(message)]
    fn get_participants(&self, page: u8) -> Result<Vec<Participant>, ParticipantManagerError>;

    #[ink(message)]
    fn get_total_value(&self) -> Balance;

    #[ink(message)]
    fn get_participant(&self, weight: Balance) -> Option<AccountId>;

    /// add participants in the raffle
    /// a participant with a weight higher than another participant will have normally more chance to be selected in the raffle
    /// weight can represent the number of raffle tickets for this participant.
    /// weight can also represent the amount staked in dAppStaking, ...
    #[ink(message)]
    fn add_participants(&mut self, participants: Vec<(AccountId, Balance)>) -> Result<(), ParticipantManagerError>;

    /// Clear the data (participants and rewards)
    #[ink(message)]
    fn clear_data(&mut self) -> Result<(), ParticipantManagerError>;

}


#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ParticipantManagerError {
    MaxSizeExceeded,
    PageNotFound,
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to ParticipantManagerError
impl From<AccessControlError> for ParticipantManagerError {
    fn from(error: AccessControlError) -> Self {
        ParticipantManagerError::AccessControlError(error)
    }
}