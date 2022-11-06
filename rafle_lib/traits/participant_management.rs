use ink_prelude::vec::Vec;
use openbrush::contracts::access_control::AccessControlError;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

#[openbrush::wrapper]
pub type ParticipantManagementRef = dyn ParticipantManagement;

#[openbrush::trait_definition]
pub trait ParticipantManagement {

    /// add a participant in the raffle for a given era
    /// a participant with a weight higher than another participant will have normally more chance to be selected in the raffle
    /// weight can represent the number of raffle tickets for this participant.
    /// weight can also represent the amount staked in dAppStaking, ...
    #[ink(message)]
    fn add_participant(&mut self, era: u32, participant: AccountId, weight: Balance) -> Result<(), ParticipantManagementError>;

    /// list all participant for a given era
    fn _list_participants(&self, era: u32) -> Vec<(AccountId, Balance)>;

}


#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ParticipantManagementError {
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to ParticipantManagementError
impl From<AccessControlError> for ParticipantManagementError {
    fn from(error: AccessControlError) -> Self {
        ParticipantManagementError::AccessControlError(error)
    }
}